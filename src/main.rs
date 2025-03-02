mod window;
use engine::{Fence, QueueType};
use window::*;

mod engine;


fn main() {
    let mut window = Window::new(640, 480, "#########1");

    let core = engine::Core::new(&window);
    let swapchain = engine::Swapchain::new(core.get_entry(), &window, core.get_instance(), core.get_device());

    let buffer = engine::Buffer::new(core.get_device(), false, true, engine::BufferType::Vertex, &[1, 2, 3]);


    let fence = Fence::new(core.get_device(), false);


    let command_pool = engine::CommandPool::new(core.get_device(), engine::QueueType::BOTH, false, false);

    let mut command_buffer = command_pool.allocate_command_buffers(false, 1).remove(0);


    command_buffer.begin(None);

    command_buffer.end();

    let submit_info = command_buffer.get_submit_info(false);

    engine::CommandBuffer::submit_buffers(core.get_device(), Some(fence),
     QueueType::BOTH, &vec![submit_info]);

    while !window.should_close() {

    }
}
