//! Windowless OpenGL ES rendering through ANGLE's D3D11 backend.

use std::{
    convert::TryFrom,
    ffi::{c_void, CStr, CString},
    ptr::null_mut,
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use crate::{
    conf::{Conf, WindowsEgl},
    native::{egl::*, gl, module::Module, Clipboard, NativeDisplayData, Request},
    EventHandler,
};

const EGL_OPENGL_ES_API: u32 = 0x30A0;
const EGL_OPENGL_ES3_BIT: EGLint = 0x0040;
const EGL_PLATFORM_ANGLE_ANGLE: u32 = 0x3202;
const EGL_PLATFORM_ANGLE_TYPE_ANGLE: EGLint = 0x3203;
const EGL_PLATFORM_ANGLE_TYPE_D3D11_ANGLE: EGLint = 0x3208;
const EGL_PLATFORM_ANGLE_DEVICE_TYPE_ANGLE: EGLint = 0x3209;
const EGL_PLATFORM_ANGLE_DEVICE_TYPE_HARDWARE_ANGLE: EGLint = 0x320A;
const EGL_PLATFORM_ANGLE_DEVICE_TYPE_D3D_WARP_ANGLE: EGLint = 0x320B;

type GetPlatformDisplay = unsafe extern "system" fn(u32, *mut c_void, *const EGLint) -> EGLDisplay;

struct AngleContext {
    egl: LibEgl,
    // Keep both DLLs loaded until after the context and application resources are destroyed.
    gles: Module,
    display: EGLDisplay,
    config: EGLConfig,
    context: EGLContext,
    surface: EGLSurface,
}

impl AngleContext {
    fn new(conf: &Conf) -> Result<Self, String> {
        let gles = Module::load("libGLESv2.dll").map_err(|err| {
            format!(
                "load ANGLE libGLESv2.dll: {err}. Place matching ANGLE DLLs beside the executable"
            )
        })?;
        let egl = LibEgl::try_load().map_err(|err| {
            format!("load ANGLE libEGL.dll: {err}. Place matching ANGLE DLLs beside the executable")
        })?;
        let mut result = Self {
            egl,
            gles,
            display: null_mut(),
            config: null_mut(),
            context: null_mut(),
            surface: null_mut(),
        };

        unsafe {
            let extensions = (result.egl.eglQueryString)(null_mut(), EGL_EXTENSIONS as _);
            if extensions.is_null() {
                return Err(result.error("query ANGLE client extensions"));
            }
            let extensions = CStr::from_ptr(extensions).to_string_lossy();
            for required in [
                "EGL_EXT_platform_base",
                "EGL_ANGLE_platform_angle",
                "EGL_ANGLE_platform_angle_d3d",
            ] {
                if !extensions.split_whitespace().any(|item| item == required) {
                    return Err(format!(
                        "ANGLE is missing {required}; a D3D11-enabled ANGLE build is required"
                    ));
                }
            }
            let address =
                (result.egl.eglGetProcAddress)(b"eglGetPlatformDisplayEXT\0".as_ptr() as _)
                    .ok_or_else(|| "ANGLE is missing eglGetPlatformDisplayEXT".to_owned())?;
            let get_display =
                std::mem::transmute::<unsafe extern "system" fn(), GetPlatformDisplay>(address);
            match conf.platform.windows_egl {
                WindowsEgl::D3D11 => result.initialize(
                    conf,
                    get_display,
                    EGL_PLATFORM_ANGLE_DEVICE_TYPE_HARDWARE_ANGLE,
                )?,
                WindowsEgl::D3D11Warp => result.initialize(
                    conf,
                    get_display,
                    EGL_PLATFORM_ANGLE_DEVICE_TYPE_D3D_WARP_ANGLE,
                )?,
                WindowsEgl::D3D11WithWarpFallback => {
                    if let Err(hardware_error) = result.initialize(
                        conf,
                        get_display,
                        EGL_PLATFORM_ANGLE_DEVICE_TYPE_HARDWARE_ANGLE,
                    ) {
                        result.release();
                        eprintln!("ANGLE D3D11 hardware initialization failed: {hardware_error}. Trying WARP");
                        result
                            .initialize(
                                conf,
                                get_display,
                                EGL_PLATFORM_ANGLE_DEVICE_TYPE_D3D_WARP_ANGLE,
                            )
                            .map_err(|warp_error| {
                                format!("D3D11: {hardware_error}; WARP: {warp_error}")
                            })?;
                    }
                }
            }
        }
        Ok(result)
    }

    fn error(&self, operation: &str) -> String {
        format!("{operation} failed (EGL error 0x{:04x})", unsafe {
            (self.egl.eglGetError)()
        })
    }

    unsafe fn initialize(
        &mut self,
        conf: &Conf,
        get_display: GetPlatformDisplay,
        device: EGLint,
    ) -> Result<(), String> {
        let attributes = [
            EGL_PLATFORM_ANGLE_TYPE_ANGLE,
            EGL_PLATFORM_ANGLE_TYPE_D3D11_ANGLE,
            EGL_PLATFORM_ANGLE_DEVICE_TYPE_ANGLE,
            device,
            EGL_NONE as _,
        ];
        self.display = get_display(EGL_PLATFORM_ANGLE_ANGLE, null_mut(), attributes.as_ptr());
        if self.display.is_null() {
            return Err(self.error("eglGetPlatformDisplayEXT"));
        }
        if (self.egl.eglInitialize)(self.display, null_mut(), null_mut()) == 0 {
            return Err(self.error("eglInitialize"));
        }
        if (self.egl.eglBindAPI)(EGL_OPENGL_ES_API) == 0 {
            return Err(self.error("eglBindAPI(OpenGL ES)"));
        }

        // ES 3 supplies the VAO and instanced draw entry points used by miniquad.
        let samples = if conf.sample_count > 1 {
            conf.sample_count
        } else {
            0
        };
        let alpha = conf.platform.framebuffer_alpha;
        self.config = match self.choose_config(alpha, samples) {
            Ok(config) => config,
            Err(error) if samples > 0 => {
                eprintln!("{error}; falling back to a config without multisampling");
                self.choose_config(alpha, 0)?
            }
            Err(error) => return Err(error),
        };
        self.context = self.create_context();
        if self.context.is_null() {
            return Err(self.error("eglCreateContext(OpenGL ES)"));
        }
        self.resize(conf.window_width.max(1), conf.window_height.max(1))
    }

    /// Creates the newest OpenGL ES context the driver can provide.
    ///
    /// ANGLE always exposes OpenGL ES 3.0, newer builds also support 3.1/3.2.
    /// Requesting the version explicitly keeps the highest available feature
    /// set, while the fallback chain works with every ANGLE build. Shaders are
    /// adapted to whatever version was actually returned.
    unsafe fn create_context(&mut self) -> EGLContext {
        if has_extension(&self.egl, self.display, "EGL_KHR_create_context") {
            for (major, minor) in [(3u32, 2u32), (3, 1), (3, 0)] {
                let attributes = [
                    EGL_CONTEXT_MAJOR_VERSION_KHR as EGLint,
                    major as EGLint,
                    EGL_CONTEXT_MINOR_VERSION_KHR as EGLint,
                    minor as EGLint,
                    EGL_NONE as EGLint,
                ];
                // Drop the error of a failed attempt so the final message is
                // about the last try only.
                (self.egl.eglGetError)();
                let context = (self.egl.eglCreateContext)(
                    self.display,
                    self.config,
                    null_mut(),
                    attributes.as_ptr(),
                );
                if !context.is_null() {
                    return context;
                }
            }
        }
        // Some EGL implementations only honor EGL_CONTEXT_CLIENT_VERSION.
        let attributes = [EGL_CONTEXT_CLIENT_VERSION as EGLint, 3, EGL_NONE as EGLint];
        (self.egl.eglCreateContext)(self.display, self.config, null_mut(), attributes.as_ptr())
    }

    unsafe fn choose_config(&self, alpha: bool, samples: EGLint) -> Result<EGLConfig, String> {
        let attributes = [
            EGL_SURFACE_TYPE as _,
            EGL_PBUFFER_BIT as _,
            EGL_RENDERABLE_TYPE as _,
            EGL_OPENGL_ES3_BIT,
            EGL_RED_SIZE as _,
            8,
            EGL_GREEN_SIZE as _,
            8,
            EGL_BLUE_SIZE as _,
            8,
            EGL_ALPHA_SIZE as _,
            if alpha { 8 } else { 0 },
            EGL_DEPTH_SIZE as _,
            16,
            EGL_STENCIL_SIZE as _,
            8,
            EGL_SAMPLE_BUFFERS as _,
            if samples > 0 { 1 } else { 0 },
            EGL_SAMPLES as _,
            samples,
            EGL_NONE as _,
        ];
        let mut config = null_mut();
        let mut count = 0;
        if (self.egl.eglChooseConfig)(
            self.display,
            attributes.as_ptr(),
            &mut config,
            1,
            &mut count,
        ) == 0
        {
            return Err(self.error("eglChooseConfig"));
        }
        if count == 0 {
            return Err(format!(
                "ANGLE has no GLES 3 pbuffer config with {samples} samples"
            ));
        }
        Ok(config)
    }

    unsafe fn resize(&mut self, width: i32, height: i32) -> Result<(), String> {
        let attributes = [
            EGL_WIDTH as _,
            width,
            EGL_HEIGHT as _,
            height,
            EGL_NONE as _,
        ];
        let surface =
            (self.egl.eglCreatePbufferSurface)(self.display, self.config, attributes.as_ptr());
        if surface.is_null() {
            return Err(self.error("eglCreatePbufferSurface"));
        }
        if (self.egl.eglMakeCurrent)(self.display, surface, surface, self.context) == 0 {
            let error = self.error("eglMakeCurrent");
            (self.egl.eglDestroySurface)(self.display, surface);
            return Err(error);
        }
        if !self.surface.is_null() {
            (self.egl.eglDestroySurface)(self.display, self.surface);
        }
        self.surface = surface;
        Ok(())
    }

    fn get_proc_address(&self, name: &str) -> Option<unsafe extern "system" fn()> {
        let cname = CString::new(name).unwrap();
        unsafe {
            if let Some(address) = (self.egl.eglGetProcAddress)(cname.as_ptr()) {
                return Some(address);
            }
            self.gles
                .get_symbol::<unsafe extern "system" fn()>(name)
                .ok()
        }
    }

    unsafe fn release(&mut self) {
        if !self.display.is_null() {
            (self.egl.eglMakeCurrent)(self.display, null_mut(), null_mut(), null_mut());
            if !self.surface.is_null() {
                (self.egl.eglDestroySurface)(self.display, self.surface);
            }
            if !self.context.is_null() {
                (self.egl.eglDestroyContext)(self.display, self.context);
            }
            (self.egl.eglTerminate)(self.display);
        }
        self.display = null_mut();
        self.config = null_mut();
        self.context = null_mut();
        self.surface = null_mut();
    }
}

impl Drop for AngleContext {
    fn drop(&mut self) {
        unsafe { self.release() }
    }
}

struct HeadlessClipboard;

impl Clipboard for HeadlessClipboard {
    fn get(&mut self) -> Option<String> {
        None
    }
    fn set(&mut self, _string: &str) {}
}

fn process_request(
    request: Request,
    context: &mut AngleContext,
    handler: &mut dyn EventHandler,
    update: &mut bool,
) -> Result<(), String> {
    match request {
        Request::ScheduleUpdate => *update = true,
        Request::SetWindowSize {
            new_width,
            new_height,
        } => {
            let width = i32::try_from(new_width)
                .map_err(|_| "pbuffer width exceeds i32::MAX")?
                .max(1);
            let height = i32::try_from(new_height)
                .map_err(|_| "pbuffer height exceeds i32::MAX")?
                .max(1);
            let changed = {
                let display = crate::native_display().lock().unwrap();
                display.screen_width != width || display.screen_height != height
            };
            if changed {
                unsafe {
                    context.resize(width, height)?;
                }
                {
                    let mut display = crate::native_display().lock().unwrap();
                    display.screen_width = width;
                    display.screen_height = height;
                }
                handler.resize_event(width as _, height as _);
            }
            *update = true;
        }
        Request::SetCursorGrab(_)
        | Request::ShowMouse(_)
        | Request::SetMouseCursor(_)
        | Request::SetWindowPosition { .. }
        | Request::SetFullscreen(_)
        | Request::ShowKeyboard(_)
        | Request::SetImePosition { .. }
        | Request::SetImeEnabled(_)
        | Request::UpdateTextInputState { .. } => {}
    }
    Ok(())
}

fn quit_requested(handler: &mut dyn EventHandler) -> bool {
    let (requested, ordered) = {
        let display = crate::native_display().lock().unwrap();
        (display.quit_requested, display.quit_ordered)
    };
    if ordered {
        return true;
    }
    if requested {
        handler.quit_requested_event();
        let mut display = crate::native_display().lock().unwrap();
        if display.quit_requested {
            display.quit_ordered = true;
        }
        return display.quit_ordered;
    }
    false
}

fn main_loop(
    conf: &Conf,
    context: &mut AngleContext,
    handler: &mut dyn EventHandler,
    rx: Receiver<Request>,
) -> Result<(), String> {
    let mut update = true;
    let rx_timeout = conf
        .platform
        .sleep_interval_ms
        .map(|sleep| Duration::from_millis(sleep as u64));
    loop {
        if quit_requested(handler) {
            break;
        }
        while let Ok(request) = rx.try_recv() {
            process_request(request, context, handler, &mut update)?;
        }
        if quit_requested(handler) {
            break;
        }
        if !conf.platform.blocking_event_loop || update {
            update = false;
            handler.update();
            if quit_requested(handler) {
                break;
            }
            handler.draw();
            // A pbuffer has no presentation step. Flush commands even if the application
            // does not read pixels this frame; readback performs its own synchronization.
            unsafe {
                gl::glFlush();
            }
        } else {
            match crate::native::rx_recv(&rx, rx_timeout) {
                Ok(request) => process_request(request, context, handler, &mut update)?,
                // Timeout so time to do a periodic update().
                Err(RecvTimeoutError::Timeout) => update = true,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }
    Ok(())
}

pub fn run<F>(conf: &Conf, f: F) -> Result<(), String>
where
    F: 'static + FnOnce() -> Box<dyn EventHandler>,
{
    let mut context = AngleContext::new(conf)?;
    for name in [
        "glGetString",
        "glGenVertexArrays",
        "glDrawElementsInstanced",
        "glReadPixels",
        "glFlush",
    ] {
        if context.get_proc_address(name).is_none() {
            return Err(format!("ANGLE is missing required GLES 3 function {name}"));
        }
    }
    gl::load_gl_funcs_system(|name| context.get_proc_address(name));
    unsafe {
        let version = gl::glGetString(gl::GL_VERSION);
        if version.is_null()
            || !CStr::from_ptr(version as _)
                .to_bytes()
                .starts_with(b"OpenGL ES 3")
        {
            return Err("ANGLE did not create a current OpenGL ES 3 context".to_owned());
        }
        let renderer = gl::glGetString(gl::GL_RENDERER);
        if !renderer.is_null() {
            eprintln!(
                "Headless renderer: {}",
                CStr::from_ptr(renderer as _).to_string_lossy()
            );
        }
    }
    let (tx, rx) = std::sync::mpsc::channel();
    crate::set_display(NativeDisplayData {
        blocking_event_loop: conf.platform.blocking_event_loop,
        ..NativeDisplayData::new(
            conf.window_width.max(1),
            conf.window_height.max(1),
            tx,
            Box::new(HeadlessClipboard),
        )
    });
    // The handler (and its GPU resources) must drop while the context is current.
    let mut handler = f();
    main_loop(conf, &mut context, &mut *handler, rx)
}
