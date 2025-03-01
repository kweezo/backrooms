mod window;
use window::*;

mod engine;


fn main() {
    let mut window = Window::new(640, 480, "#########1");

    let core = engine::Core::new(&window);
    let swapchain = engine::Swapchain::new(core.get_entry(), &window, core.get_instance(), core.get_device());

    let buffer = engine::Buffer::new(core.get_device(), false, true, engine::BufferType::Vertex, &[1, 2, 3]);

    while !window.should_close() {

    }
}
