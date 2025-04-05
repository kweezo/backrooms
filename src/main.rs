mod window;
use std::mem::swap;

use engine::{Attachment, Buffer, Image, ImageCreateInfo, RenderPass, ResourceManager, ResourceQueue, Shader, Subpass};
use window::*;
use ash::vk::{self};

mod engine;


fn main() {
    let mut window = Window::new(640, 480, "#########1");

    let core = engine::Core::new(&window);
    let swapchain = engine::Swapchain::new(core.get_entry(), &window, core.get_instance(), core.get_device());

    let shader = Shader::new(core.get_device(), "shaders/spirv/triangle.vert.spirv", "shaders/spirv/triangle.frag.spirv");

    let attachment = Attachment {
        format: swapchain.get_image_format(),
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR
    };

    let attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
    };

    let subpass = Subpass {
        bind_point: vk::PipelineBindPoint::GRAPHICS,
        color_attachments: vec![attachment_ref],
        input_attachments: vec![],
        resolve_attachments: vec![]
    };

    let render_pass = RenderPass::new(core.get_device(), &[attachment], &[subpass], &[],
        &swapchain.get_image_views(), swapchain.get_size());

    while !window.should_close() {
//        resource_manager.update();
    }
}
