use glfw::WindowEvent;

pub struct Window {
    glfw_context: glfw::Glfw,
    window_handle: glfw::PWindow,

    events: glfw::GlfwReceiver<(f64, WindowEvent)>,

    width: u32,
    height: u32,
}

impl Window {

    fn error_callback(err: glfw::Error, description: String) {
        eprintln!("GLFW error {:?}: {:?}", err, description);
    }

    pub fn new(width: u32, height: u32, title: &str) -> Window {

        let mut context = glfw::init(Window::error_callback)
        .expect("Failed to initialize GLFW");


        context.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
        context.window_hint(glfw::WindowHint::Resizable(false));


        let (mut window, events) = context
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create a GLFW window");


        window.set_key_polling(true);

        Window {
            glfw_context: context,
            events,
            window_handle: window,
            width,
            height,
        }
    }

    pub fn should_close(&mut self) -> bool {
        self.glfw_context.poll_events();

        self.window_handle.should_close()
    }

    pub fn get_context(&self) -> &glfw::Glfw {
        &self.glfw_context
    }

    pub fn get_window(&self) -> &glfw::Window {
        &self.window_handle
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}