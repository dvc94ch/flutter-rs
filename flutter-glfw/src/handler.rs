use async_std::task;
use flutter_engine::ffi::ExternalTextureFrame;
use flutter_engine::texture_registry::TextureRegistry;
use flutter_engine::FlutterEngineHandler;
use flutter_plugins::platform::{AppSwitcherDescription, PlatformHandler, MimeError};
use flutter_plugins::window::{PositionParams, WindowHandler};
use futures_task::FutureObj;
use glfw::Context;
use parking_lot::Mutex;
use std::ffi::c_void;
use std::future::Future;
use std::sync::Arc;

pub(crate) struct GlfwFlutterEngineHandler {
    pub(crate) glfw: glfw::Glfw,
    pub(crate) window: Arc<Mutex<glfw::Window>>,
    pub(crate) resource_window: Arc<Mutex<glfw::Window>>,
    pub(crate) texture_registry: Arc<Mutex<TextureRegistry>>,
}

impl FlutterEngineHandler for GlfwFlutterEngineHandler {
    fn swap_buffers(&self) -> bool {
        self.window.lock().swap_buffers();
        true
    }

    fn make_current(&self) -> bool {
        self.window.lock().make_current();
        true
    }

    fn clear_current(&self) -> bool {
        glfw::make_context_current(None);
        true
    }

    fn fbo_callback(&self) -> u32 {
        0
    }

    fn make_resource_current(&self) -> bool {
        self.resource_window.lock().make_current();
        true
    }

    fn gl_proc_resolver(&self, proc: *const i8) -> *mut c_void {
        unsafe {
            self.glfw
                .get_proc_address_raw(&glfw::string_from_c_str(proc)) as *mut c_void
        }
    }

    fn wake_platform_thread(&self) {
        unsafe {
            glfw::ffi::glfwPostEmptyEvent();
        }
    }

    fn run_in_background(&self, func: Box<dyn Future<Output = ()> + Send + 'static>) {
        task::spawn(FutureObj::new(func));
    }

    fn get_texture_frame(
        &self,
        texture_id: i64,
        size: (usize, usize),
    ) -> Option<ExternalTextureFrame> {
        let (width, height) = size;
        self.texture_registry
            .lock()
            .get_texture_frame(texture_id, (width as u32, height as u32))
    }
}

pub struct GlfwPlatformHandler {
    pub window: Arc<Mutex<glfw::Window>>,
}

unsafe impl Send for GlfwPlatformHandler {}

impl PlatformHandler for GlfwPlatformHandler {
    fn set_application_switcher_description(&mut self, description: AppSwitcherDescription) {
        self.window.lock().set_title(&description.label);
    }

    fn set_clipboard_data(&mut self, text: String) {
        self.window.lock().set_clipboard_string(&text);
    }

    fn get_clipboard_data(&mut self, mime: &str) -> Result<String, MimeError> {
        match mime {
            "text/plain" => Ok(match self.window.lock().get_clipboard_string() {
                None => "".to_string(),
                Some(val) => val,
            }),
            _ => Err(MimeError),
        }
    }
}

pub struct GlfwWindowHandler {
    window: Arc<Mutex<glfw::Window>>,
    dragging: bool,
    start_cursor_pos: (f64, f64),
}

impl GlfwWindowHandler {
    pub fn new(window: Arc<Mutex<glfw::Window>>) -> Self {
        Self {
            window,
            dragging: false,
            start_cursor_pos: (0.0, 0.0),
        }
    }

    pub fn drag_window(&self, x: f64, y: f64) -> bool {
        if self.dragging {
            let mut window = self.window.lock();
            let (wx, wy) = window.get_pos();
            let dx = (x - self.start_cursor_pos.0) as i32;
            let dy = (y - self.start_cursor_pos.1) as i32;
            window.set_pos(wx + dx, wy + dy);
        }
        self.dragging
    }
}

unsafe impl Send for GlfwWindowHandler {}

impl WindowHandler for GlfwWindowHandler {
    fn close(&mut self) {
        self.window.lock().set_should_close(true);
    }

    fn show(&mut self) {
        self.window.lock().show();
    }

    fn hide(&mut self) {
        self.window.lock().hide();
    }

    fn maximize(&mut self) {
        self.window.lock().maximize();
    }

    fn iconify(&mut self) {
        self.window.lock().iconify();
    }

    fn restore(&mut self) {
        self.window.lock().restore();
    }

    fn is_maximized(&mut self) -> bool {
        self.window.lock().is_maximized()
    }

    fn is_iconified(&mut self) -> bool {
        self.window.lock().is_iconified()
    }

    fn is_visible(&mut self) -> bool {
        self.window.lock().is_visible()
    }

    fn set_pos(&mut self, pos: PositionParams) {
        self.window.lock().set_pos(pos.x as i32, pos.y as i32);
    }

    fn get_pos(&mut self) -> PositionParams {
        let (x, y) = self.window.lock().get_pos();
        PositionParams {
            x: x as f32,
            y: y as f32,
        }
    }

    fn start_drag(&mut self) {
        self.dragging = true;
        self.start_cursor_pos = self.window.lock().get_cursor_pos();
    }

    fn end_drag(&mut self) {
        self.dragging = false;
    }
}
