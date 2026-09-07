#![allow(non_camel_case_types, non_snake_case, dead_code)]

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
pub type EGLNativeDisplayType = *mut crate::native::linux_x11::libx11::Display;
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
pub type EGLNativePixmapType = crate::native::linux_x11::libx11::Pixmap;
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
pub type EGLNativeWindowType = crate::native::linux_x11::libx11::Window;

#[cfg(any(target_os = "android", target_env = "ohos"))]
pub type EGLNativeDisplayType = *mut ();
#[cfg(any(target_os = "android", target_env = "ohos"))]
pub type EGLNativePixmapType = ::core::ffi::c_ulong;
#[cfg(any(target_os = "android", target_env = "ohos"))]
pub type EGLNativeWindowType = ::core::ffi::c_ulong;

pub use core::ptr::null_mut;
use std::fmt::Display;

pub const EGL_SUCCESS: u32 = 12288;

pub const EGL_WINDOW_BIT: u32 = 4;
pub const EGL_PBUFFER_BIT: u32 = 1;
pub const EGL_OPENGL_BIT: u32 = 8;
pub const EGL_RENDERABLE_TYPE: u32 = 12352;
pub const EGL_OPENGL_API: u32 = 0x30A2;
pub const EGL_EXTENSIONS: u32 = 0x3055;
pub const EGL_CONTEXT_MAJOR_VERSION_KHR: u32 = 0x3098;
pub const EGL_CONTEXT_MINOR_VERSION_KHR: u32 = 0x30FB;
pub const EGL_PLATFORM_DEVICE_EXT: u32 = 0x313F;
pub const EGL_PLATFORM_SURFACELESS_MESA: u32 = 0x31DD;

pub const EGL_ALPHA_SIZE: u32 = 12321;
pub const EGL_BLUE_SIZE: u32 = 12322;
pub const EGL_GREEN_SIZE: u32 = 12323;
pub const EGL_RED_SIZE: u32 = 12324;
pub const EGL_DEPTH_SIZE: u32 = 12325;
pub const EGL_STENCIL_SIZE: u32 = 12326;
pub const EGL_SAMPLES: u32 = 12337;
pub const EGL_NATIVE_VISUAL_ID: u32 = 12334;
pub const EGL_WIDTH: u32 = 12375;
pub const EGL_HEIGHT: u32 = 12374;
pub const EGL_SURFACE_TYPE: u32 = 12339;
pub const EGL_NONE: u32 = 12344;
pub const EGL_CONTEXT_CLIENT_VERSION: u32 = 12440;

pub type NativeDisplayType = EGLNativeDisplayType;
pub type NativePixmapType = EGLNativePixmapType;
pub type NativeWindowType = EGLNativeWindowType;
pub type EGLint = i32;
pub type EGLBoolean = ::core::ffi::c_uint;
pub type EGLDisplay = *mut ::core::ffi::c_void;
pub type EGLConfig = *mut ::core::ffi::c_void;
pub type EGLSurface = *mut ::core::ffi::c_void;
pub type EGLContext = *mut ::core::ffi::c_void;
pub type __eglMustCastToProperFunctionPointerType = ::std::option::Option<unsafe extern "C" fn()>;

crate::declare_module! {
    LibEgl,
    "libEGL.so",
    "libEGL.so.1",
    ...
    ...
    pub fn eglChooseConfig(
        EGLDisplay,
        *const EGLint,
        *mut EGLConfig,
        EGLint,
        *mut EGLint,
    ) -> EGLBoolean,
    pub fn eglCopyBuffers(
        EGLDisplay,
        EGLSurface,
        EGLNativePixmapType,
    ) -> EGLBoolean,
    pub fn eglCreateContext(
        EGLDisplay,
        EGLConfig,
        EGLContext,
        *const EGLint,
    ) -> EGLContext,
    pub fn eglCreatePbufferSurface(
        EGLDisplay,
        EGLConfig,
        *const EGLint,
    ) -> EGLSurface,
    pub fn eglCreatePixmapSurface(
        EGLDisplay,
        EGLConfig,
        EGLNativePixmapType,
        *const EGLint,
    ) -> EGLSurface,
    pub fn eglCreateWindowSurface(
        EGLDisplay,
        EGLConfig,
        EGLNativeWindowType,
        *const EGLint,
    ) -> EGLSurface,
    pub fn eglDestroyContext(EGLDisplay, EGLContext) -> EGLBoolean,
    pub fn eglDestroySurface(EGLDisplay, EGLSurface) -> EGLBoolean,
    pub fn eglGetConfigAttrib(
        EGLDisplay,
        EGLConfig,
        EGLint,
        *mut EGLint,
    ) -> EGLBoolean,
    pub fn eglGetConfigs(
        EGLDisplay,
        *mut EGLConfig,
        EGLint,
        *mut EGLint,
    ) -> EGLBoolean,
    pub fn eglGetCurrentContext() -> EGLContext,
    pub fn eglGetCurrentDisplay() -> EGLDisplay,
    pub fn eglGetCurrentSurface(EGLint) -> EGLSurface,
    pub fn eglGetDisplay(EGLNativeDisplayType) -> EGLDisplay,
    pub fn eglGetError() -> EGLint,
    pub fn eglGetProcAddress(
        *const ::core::ffi::c_char,
    ) -> __eglMustCastToProperFunctionPointerType,
    pub fn eglInitialize(EGLDisplay, *mut EGLint, *mut EGLint) -> EGLBoolean,
    pub fn eglMakeCurrent(
        EGLDisplay,
        EGLSurface,
        EGLSurface,
        EGLContext,
    ) -> EGLBoolean,
    pub fn eglQueryContext(
        EGLDisplay,
        EGLContext,
        EGLint,
        *mut EGLint,
    ) -> EGLBoolean,
    pub fn eglQueryString(EGLDisplay, EGLint) -> *const ::core::ffi::c_char,
    pub fn eglQuerySurface(
        EGLDisplay,
        EGLSurface,
        EGLint,
        *mut EGLint,
    ) -> EGLBoolean,
    pub fn eglSwapBuffers(EGLDisplay, EGLSurface) -> EGLBoolean,
    pub fn eglTerminate(EGLDisplay) -> EGLBoolean,
    pub fn eglWaitGL() -> EGLBoolean,
    pub fn eglWaitNative(EGLint) -> EGLBoolean,
    pub fn eglBindTexImage(EGLDisplay, EGLSurface, EGLint) -> EGLBoolean,
    pub fn eglBindAPI(u32) -> EGLBoolean,
    pub fn eglReleaseTexImage(EGLDisplay, EGLSurface, EGLint) -> EGLBoolean,
    pub fn eglSurfaceAttrib(
        EGLDisplay,
        EGLSurface,
        EGLint,
        EGLint,
    ) -> EGLBoolean,
    pub fn eglSwapInterval(EGLDisplay, EGLint) -> EGLBoolean,
    ...
    ...
}

#[derive(Debug)]
pub enum EglError {
    NoDisplay,
    InitializeFailed,
    UnsupportedPlatform,
    BindApiFailed,
    NoConfig,
    CreateContextFailed,
    CreateSurfaceFailed,
    MakeCurrentFailed,
}

impl Display for EglError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDisplay => write!(f, "No display"),
            Self::InitializeFailed => write!(f, "Failed to initialize context"),
            Self::UnsupportedPlatform => write!(f, "No headless EGL platform is available"),
            Self::BindApiFailed => write!(f, "Failed to bind the desktop OpenGL EGL API"),
            Self::NoConfig => write!(f, "No compatible EGL pbuffer/OpenGL config was found"),
            Self::CreateContextFailed => write!(f, "Failed to create desktop OpenGL context"),
            Self::CreateSurfaceFailed => write!(f, "Failed to create EGL pbuffer surface"),
            Self::MakeCurrentFailed => write!(f, "Failed to make EGL context current"),
        }
    }
}

impl std::error::Error for EglError {}

pub struct Egl {}

pub unsafe fn create_egl_context(
    egl: &mut LibEgl,
    display: *mut std::ffi::c_void,
    alpha: bool,
    sample_count: i32,
) -> Result<(EGLContext, EGLConfig, EGLDisplay), EglError> {
    let display = (egl.eglGetDisplay)(display as _);
    if display.is_null() {
        // == EGL_NO_DISPLAY
        return Err(EglError::NoDisplay);
    }

    if (egl.eglInitialize)(display, null_mut(), null_mut()) == 0 {
        return Err(EglError::InitializeFailed);
    }

    let alpha_size = if alpha { 8 } else { 0 };
    #[rustfmt::skip]
    let cfg_attributes = [
        EGL_SURFACE_TYPE, EGL_WINDOW_BIT,
        EGL_RED_SIZE, 8,
        EGL_GREEN_SIZE, 8,
        EGL_BLUE_SIZE, 8,
        EGL_ALPHA_SIZE, alpha_size,
        EGL_DEPTH_SIZE, 16,
        EGL_STENCIL_SIZE, 0,
        EGL_SAMPLES, sample_count as u32,
        EGL_NONE,
    ];
    let mut available_cfgs: Vec<EGLConfig> = vec![null_mut(); 32];
    let mut cfg_count = 0;

    (egl.eglChooseConfig)(
        display,
        cfg_attributes.as_ptr() as _,
        available_cfgs.as_ptr() as _,
        32,
        &mut cfg_count as *mut _ as *mut _,
    );
    assert!(cfg_count > 0);
    assert!(cfg_count <= 32);

    // find config with 8-bit rgb buffer if available, ndk sample does not trust egl spec
    let mut config: EGLConfig = null_mut();
    let mut exact_cfg_found = false;
    for c in &mut available_cfgs[0..cfg_count] {
        let mut r: i32 = 0;
        let mut g: i32 = 0;
        let mut b: i32 = 0;
        let mut a: i32 = 0;
        let mut d: i32 = 0;
        if (egl.eglGetConfigAttrib)(display, *c, EGL_RED_SIZE as _, &mut r) == 1
            && (egl.eglGetConfigAttrib)(display, *c, EGL_GREEN_SIZE as _, &mut g) == 1
            && (egl.eglGetConfigAttrib)(display, *c, EGL_BLUE_SIZE as _, &mut b) == 1
            && (egl.eglGetConfigAttrib)(display, *c, EGL_ALPHA_SIZE as _, &mut a) == 1
            && (egl.eglGetConfigAttrib)(display, *c, EGL_DEPTH_SIZE as _, &mut d) == 1
            && r == 8
            && g == 8
            && b == 8
            && (alpha_size == 0 || a == alpha_size as _)
            && d == 16
        {
            exact_cfg_found = true;
            config = *c;
            break;
        }
    }
    if !exact_cfg_found {
        config = available_cfgs[0];
    }
    let ctx_attributes = [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
    let context = (egl.eglCreateContext)(
        display,
        config,
        /* EGL_NO_CONTEXT */ null_mut(),
        ctx_attributes.as_ptr() as _,
    );
    if context.is_null() {
        return Err(EglError::CreateContextFailed);
    }

    Ok((context, config, display))
}
pub struct HeadlessEglContext {
    pub display: EGLDisplay,
    pub surface: EGLSurface,
    pub context: EGLContext,
}

impl HeadlessEglContext {
    pub unsafe fn destroy(self, egl: &mut LibEgl) {
        (egl.eglMakeCurrent)(self.display, null_mut(), null_mut(), null_mut());
        (egl.eglDestroySurface)(self.display, self.surface);
        (egl.eglDestroyContext)(self.display, self.context);
        (egl.eglTerminate)(self.display);
    }
}

type EglDeviceExt = *mut std::ffi::c_void;
type EglQueryDevicesExt =
    unsafe extern "C" fn(EGLint, *mut EglDeviceExt, *mut EGLint) -> EGLBoolean;
type EglGetPlatformDisplayExt =
    unsafe extern "C" fn(u32, *mut std::ffi::c_void, *const EGLint) -> EGLDisplay;

unsafe fn query_devices_proc(egl: &LibEgl) -> Option<EglQueryDevicesExt> {
    let symbol = b"eglQueryDevicesEXT\0";
    let address = (egl.eglGetProcAddress)(symbol.as_ptr() as _)?;
    Some(std::mem::transmute::<
        unsafe extern "C" fn(),
        EglQueryDevicesExt,
    >(address))
}

unsafe fn get_platform_display_proc(egl: &LibEgl) -> Option<EglGetPlatformDisplayExt> {
    let symbol = b"eglGetPlatformDisplayEXT\0";
    let address = (egl.eglGetProcAddress)(symbol.as_ptr() as _)?;
    Some(std::mem::transmute::<
        unsafe extern "C" fn(),
        EglGetPlatformDisplayExt,
    >(address))
}

unsafe fn choose_headless_display(egl: &LibEgl) -> Result<EGLDisplay, EglError> {
    let Some(get_platform_display) = get_platform_display_proc(egl) else {
        return Err(EglError::UnsupportedPlatform);
    };

    if let Some(query_devices) = query_devices_proc(egl) {
        let mut device_count = 0;
        if query_devices(0, null_mut(), &mut device_count) != 0 && device_count > 0 {
            let mut devices = vec![null_mut(); device_count as usize];
            if query_devices(device_count, devices.as_mut_ptr(), &mut device_count) != 0 {
                let selected = std::env::var("PHI_RENDERER_EGL_DEVICE")
                    .ok()
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(0);
                if let Some(device) = devices.get(selected).copied() {
                    let display = get_platform_display(EGL_PLATFORM_DEVICE_EXT, device, null_mut());
                    if !display.is_null() {
                        return Ok(display);
                    }
                }
            }
            return Err(EglError::NoDisplay);
        }
    }

    let display = get_platform_display(EGL_PLATFORM_SURFACELESS_MESA, null_mut(), null_mut());
    if display.is_null() {
        Err(EglError::UnsupportedPlatform)
    } else {
        Ok(display)
    }
}

unsafe fn has_extension(egl: &LibEgl, display: EGLDisplay, extension: &str) -> bool {
    let value = (egl.eglQueryString)(display, EGL_EXTENSIONS as EGLint);
    if value.is_null() {
        return false;
    }
    std::ffi::CStr::from_ptr(value)
        .to_string_lossy()
        .split_whitespace()
        .any(|item| item == extension)
}

pub unsafe fn create_headless_egl_context(
    egl: &mut LibEgl,
    width: i32,
    height: i32,
    alpha: bool,
) -> Result<HeadlessEglContext, EglError> {
    let display = choose_headless_display(egl)?;
    if (egl.eglInitialize)(display, null_mut(), null_mut()) == 0 {
        return Err(EglError::InitializeFailed);
    }
    if (egl.eglBindAPI)(EGL_OPENGL_API) == 0 {
        (egl.eglTerminate)(display);
        return Err(EglError::BindApiFailed);
    }

    let alpha_size = if alpha { 8 } else { 0 };
    #[rustfmt::skip]
    let config_attributes = [
        EGL_SURFACE_TYPE as EGLint, EGL_PBUFFER_BIT as EGLint,
        EGL_RENDERABLE_TYPE as EGLint, EGL_OPENGL_BIT as EGLint,
        EGL_RED_SIZE as EGLint, 8,
        EGL_GREEN_SIZE as EGLint, 8,
        EGL_BLUE_SIZE as EGLint, 8,
        EGL_ALPHA_SIZE as EGLint, alpha_size,
        EGL_DEPTH_SIZE as EGLint, 16,
        EGL_STENCIL_SIZE as EGLint, 0,
        EGL_NONE as EGLint,
    ];
    let mut available_configs = [null_mut(); 32];
    let mut config_count = 0;
    if (egl.eglChooseConfig)(
        display,
        config_attributes.as_ptr(),
        available_configs.as_mut_ptr(),
        available_configs.len() as EGLint,
        &mut config_count,
    ) == 0
        || config_count <= 0
    {
        (egl.eglTerminate)(display);
        return Err(EglError::NoConfig);
    }

    let config = available_configs[0..(config_count as usize).min(available_configs.len())]
        .iter()
        .copied()
        .find(|config| {
            let mut red = 0;
            let mut green = 0;
            let mut blue = 0;
            let mut config_alpha = 0;
            let mut depth = 0;
            (egl.eglGetConfigAttrib)(display, *config, EGL_RED_SIZE as _, &mut red) != 0
                && (egl.eglGetConfigAttrib)(display, *config, EGL_GREEN_SIZE as _, &mut green) != 0
                && (egl.eglGetConfigAttrib)(display, *config, EGL_BLUE_SIZE as _, &mut blue) != 0
                && (egl.eglGetConfigAttrib)(
                    display,
                    *config,
                    EGL_ALPHA_SIZE as _,
                    &mut config_alpha,
                ) != 0
                && (egl.eglGetConfigAttrib)(display, *config, EGL_DEPTH_SIZE as _, &mut depth) != 0
                && red == 8
                && green == 8
                && blue == 8
                && (alpha_size == 0 || config_alpha == alpha_size)
                && depth >= 16
        })
        .unwrap_or(available_configs[0]);

    let context_attributes = if has_extension(egl, display, "EGL_KHR_create_context") {
        vec![
            EGL_CONTEXT_MAJOR_VERSION_KHR as EGLint,
            3,
            EGL_CONTEXT_MINOR_VERSION_KHR as EGLint,
            1,
            EGL_NONE as EGLint,
        ]
    } else {
        vec![EGL_NONE as EGLint]
    };
    let context = (egl.eglCreateContext)(display, config, null_mut(), context_attributes.as_ptr());
    if context.is_null() {
        (egl.eglTerminate)(display);
        return Err(EglError::CreateContextFailed);
    }

    let surface_attributes = [
        EGL_WIDTH as EGLint,
        width.max(1),
        EGL_HEIGHT as EGLint,
        height.max(1),
        EGL_NONE as EGLint,
    ];
    let surface = (egl.eglCreatePbufferSurface)(display, config, surface_attributes.as_ptr());
    if surface.is_null() {
        (egl.eglDestroyContext)(display, context);
        (egl.eglTerminate)(display);
        return Err(EglError::CreateSurfaceFailed);
    }

    if (egl.eglMakeCurrent)(display, surface, surface, context) == 0 {
        (egl.eglDestroySurface)(display, surface);
        (egl.eglDestroyContext)(display, context);
        (egl.eglTerminate)(display);
        return Err(EglError::MakeCurrentFailed);
    }

    Ok(HeadlessEglContext {
        display,
        surface,
        context,
    })
}
