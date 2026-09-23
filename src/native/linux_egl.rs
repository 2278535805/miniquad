use std::{
    convert::TryFrom,
    ffi::{CStr, CString},
    sync::mpsc::{Receiver, RecvTimeoutError},
    time::Duration,
};

use crate::{
    event::EventHandler,
    native::{egl, gl, Clipboard, NativeDisplayData, Request},
};

struct HeadlessClipboard;

impl Clipboard for HeadlessClipboard {
    fn get(&mut self) -> Option<String> {
        None
    }

    fn set(&mut self, _string: &str) {}
}

fn process_request(
    request: Request,
    egl_context: &mut egl::HeadlessEglContext,
    egl_lib: &egl::LibEgl,
    event_handler: &mut dyn EventHandler,
    update_requested: &mut bool,
) -> Result<(), String> {
    match request {
        Request::ScheduleUpdate => *update_requested = true,
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
                    egl_context
                        .resize(egl_lib, width, height)
                        .map_err(|error| format!("resize headless EGL pbuffer: {error}"))?;
                }
                {
                    let mut display = crate::native_display().lock().unwrap();
                    display.screen_width = width;
                    display.screen_height = height;
                }
                event_handler.resize_event(width as _, height as _);
            }
            *update_requested = true;
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
    conf: &crate::conf::Conf,
    egl_context: &mut egl::HeadlessEglContext,
    egl_lib: &egl::LibEgl,
    handler: &mut dyn EventHandler,
    rx: Receiver<Request>,
) -> Result<(), String> {
    let mut update_requested = true;
    let rx_timeout = conf
        .platform
        .sleep_interval_ms
        .map(|sleep| Duration::from_millis(sleep as u64));
    loop {
        if quit_requested(handler) {
            break;
        }
        while let Ok(request) = rx.try_recv() {
            process_request(
                request,
                egl_context,
                egl_lib,
                handler,
                &mut update_requested,
            )?;
        }
        if quit_requested(handler) {
            break;
        }
        if !conf.platform.blocking_event_loop || update_requested {
            update_requested = false;
            handler.update();
            if quit_requested(handler) {
                break;
            }
            handler.draw();
            // A pbuffer has no presentation step, but flush the commands so the
            // framebuffer is up to date for glReadPixels.
            unsafe {
                gl::glFlush();
            }
        } else {
            match crate::native::rx_recv(&rx, rx_timeout) {
                Ok(request) => process_request(
                    request,
                    egl_context,
                    egl_lib,
                    handler,
                    &mut update_requested,
                )?,
                // Timeout so time to do a periodic update().
                Err(RecvTimeoutError::Timeout) => update_requested = true,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }
    Ok(())
}

pub fn run<F>(conf: &crate::conf::Conf, f: F) -> Result<(), String>
where
    F: 'static + FnOnce() -> Box<dyn EventHandler>,
{
    let width = conf.window_width.max(1);
    let height = conf.window_height.max(1);
    let mut egl_lib = egl::LibEgl::try_load()
        .map_err(|error| format!("load libEGL for headless rendering: {error}"))?;

    for name in [
        "glGetString",
        "glFlush",
        "glGenVertexArrays",
        "glDrawElementsInstanced",
    ] {
        let cname = CString::new(name).expect("OpenGL symbol contains NUL");
        if unsafe { (egl_lib.eglGetProcAddress)(cname.as_ptr() as _) }.is_none() {
            return Err(format!(
                "headless EGL is missing required OpenGL function {name}"
            ));
        }
    }

    let mut egl_context = unsafe {
        egl::create_headless_egl_context(
            &mut egl_lib,
            width,
            height,
            conf.platform.framebuffer_alpha,
            conf.sample_count,
        )
    }
    .map_err(|error| format!("create headless desktop OpenGL context: {error}"))?;

    gl::load_gl_funcs(|proc| {
        let name = CString::new(proc).expect("OpenGL symbol contains NUL");
        unsafe { (egl_lib.eglGetProcAddress)(name.as_ptr() as _) }
    });

    if let Err(error) = unsafe { check_gl_version() } {
        unsafe {
            egl_context.destroy(&egl_lib);
        }
        return Err(error);
    }

    if let Some(interval) = conf.platform.swap_interval {
        unsafe {
            (egl_lib.eglSwapInterval)(egl_context.display, interval);
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    crate::set_display(NativeDisplayData {
        blocking_event_loop: conf.platform.blocking_event_loop,
        ..NativeDisplayData::new(width, height, tx, Box::new(HeadlessClipboard))
    });

    // The handler (and its GPU resources) must drop while the context is current.
    let mut handler = f();
    let result = main_loop(conf, &mut egl_context, &egl_lib, &mut *handler, rx);
    drop(handler);
    unsafe {
        egl_context.destroy(&egl_lib);
    }
    result
}

unsafe fn check_gl_version() -> Result<(), String> {
    let version = gl::glGetString(gl::GL_VERSION);
    if version.is_null() {
        return Err("headless EGL did not create a current desktop OpenGL context".to_owned());
    }
    let version = CStr::from_ptr(version as _).to_string_lossy();
    let major = version
        .split('.')
        .next()
        .and_then(|major| major.trim().parse::<u32>().ok());
    if major.map_or(true, |major| major < 3) {
        return Err(format!(
            "headless EGL context is not desktop OpenGL 3.x or newer (GL_VERSION: {version})"
        ));
    }
    let renderer = gl::glGetString(gl::GL_RENDERER);
    if !renderer.is_null() {
        eprintln!(
            "Headless renderer: {}",
            CStr::from_ptr(renderer as _).to_string_lossy()
        );
    }
    Ok(())
}
