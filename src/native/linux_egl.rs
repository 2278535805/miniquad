use crate::{
    event::EventHandler,
    native::{egl, Clipboard, NativeDisplayData, Request},
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
    update_requested: &mut bool,
    event_handler: &mut dyn EventHandler,
) {
    match request {
        Request::ScheduleUpdate => *update_requested = true,
        Request::SetWindowSize {
            new_width,
            new_height,
        } => {
            let changed = {
                let mut display = crate::native_display().lock().unwrap();
                if display.screen_width == new_width as i32
                    && display.screen_height == new_height as i32
                {
                    false
                } else {
                    display.screen_width = new_width as i32;
                    display.screen_height = new_height as i32;
                    true
                }
            };
            if changed {
                event_handler.resize_event(new_width as f32, new_height as f32);
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
}

pub fn run<F>(conf: &crate::conf::Conf, f: &mut Option<F>) -> Result<(), String>
where
    F: 'static + FnOnce() -> Box<dyn EventHandler>,
{
    let width = conf.window_width.max(1);
    let height = conf.window_height.max(1);
    let mut egl_lib = egl::LibEgl::try_load()
        .map_err(|error| format!("load libEGL for headless rendering: {error}"))?;
    let egl_context = unsafe {
        egl::create_headless_egl_context(
            &mut egl_lib,
            width,
            height,
            conf.platform.framebuffer_alpha,
        )
    }
    .map_err(|error| format!("create headless desktop OpenGL context: {error}"))?;

    crate::native::gl::load_gl_funcs(|proc| {
        let name = std::ffi::CString::new(proc).expect("OpenGL symbol contains NUL");
        unsafe { (egl_lib.eglGetProcAddress)(name.as_ptr() as _) }
    });

    if let Some(interval) = conf.platform.swap_interval {
        unsafe {
            (egl_lib.eglSwapInterval)(egl_context.display, interval);
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    crate::set_display(NativeDisplayData {
        high_dpi: conf.high_dpi,
        dpi_scale: 1.0,
        blocking_event_loop: conf.platform.blocking_event_loop,
        ..NativeDisplayData::new(width, height, tx, Box::new(HeadlessClipboard))
    });

    let mut event_handler =
        f.take()
            .ok_or_else(|| "headless renderer callback was already consumed".to_owned())?();
    let mut update_requested = true;

    while !crate::native_display().try_lock().unwrap().quit_ordered {
        while let Ok(request) = rx.try_recv() {
            process_request(request, &mut update_requested, &mut *event_handler);
        }

        if !conf.platform.blocking_event_loop || update_requested {
            update_requested = false;
            event_handler.update();
            event_handler.draw();
            unsafe {
                (egl_lib.eglSwapBuffers)(egl_context.display, egl_context.surface);
            }
        } else {
            std::thread::yield_now();
        }
    }

    unsafe {
        egl_context.destroy(&mut egl_lib);
    }
    Ok(())
}
