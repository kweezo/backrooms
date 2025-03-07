mod window;
use engine::{Buffer, BufferCopyInfo, Fence, QueueType, ResourceManager, ResourceQueue};
use window::*;

mod engine;


fn main() {
    let mut window = Window::new(640, 480, "#########1");

    let core = engine::Core::new(&window);
    let swapchain = engine::Swapchain::new(core.get_entry(), &window, core.get_instance(), core.get_device());

    let mut buffers = Vec::<Buffer>::with_capacity(50);
    let mut queues = Vec::<ResourceQueue>::with_capacity(50);

    for i in 0..50 {
        buffers.push(
            Buffer::new(core.get_device(), true, true, engine::BufferType::Vertex, &[1, 2, 3])
        );

        queues.push(ResourceQueue::new());

        let len = queues.len();
        queues[len-1].add_copy_ops(vec![buffers[buffers.len()-1].get_copy_op()]);
    }


    let mut resource_manager = ResourceManager::new(core.get_device());

    for (i, queue) in queues.iter_mut().enumerate() {
        dbg!(i);
        resource_manager.submit_queue(queue);
    }

    unsafe {
        let _ = device!(core).device_wait_idle();
    }

    while !window.should_close() {
    }
}
