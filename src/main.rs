mod window;
use engine::{Fence, QueueType, ResourceManager, ResourceQueue};
use window::*;

mod engine;


fn main() {
    let mut window = Window::new(640, 480, "#########1");

    let core = engine::Core::new(&window);
    let swapchain = engine::Swapchain::new(core.get_entry(), &window, core.get_instance(), core.get_device());

    let buffer = engine::Buffer::new(core.get_device(), false, true, engine::BufferType::Vertex, &[1, 2, 3]);



    let mut resource_manager = ResourceManager::new(core.get_device());

    let mut queue = ResourceQueue::new();
    queue.add_copy_ops(vec![buffer.get_copy_op()]);

    resource_manager.submit_queue(&mut queue);

    unsafe{
        device!(core).device_wait_idle();
    }
}
