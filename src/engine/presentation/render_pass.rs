use ash::vk;

use crate::engine::Device;

pub struct Attachment {
    pub format: vk::Format,
    pub samples: vk::SampleCountFlags,
    pub load_op: vk::AttachmentLoadOp,
    pub store_op: vk::AttachmentStoreOp,
    pub initial_layout: vk::ImageLayout,
    pub final_layout: vk::ImageLayout
}

pub struct Subpass {
    pub bind_point: vk::PipelineBindPoint,
    pub input_attachments: Vec<vk::AttachmentReference>,
    pub color_attachments: Vec<vk::AttachmentReference>,
    pub resolve_attachments: Vec<vk::AttachmentReference>,
}

pub struct RenderPass<'a> {
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,

    device: &'a Device
}

impl<'a> RenderPass<'a> {
    pub fn new(device: &'a Device, attachments: &[Attachment], subpasses: &[Subpass], dependencies: &[vk::SubpassDependency], swapchain_attachments: &[vk::ImageView], swapchain_size: (u32, u32)) -> RenderPass<'a> {
        let render_pass = RenderPass::create_render_pass(device, attachments, subpasses, dependencies);
        let framebuffers = RenderPass::create_framebuffers(device, render_pass, swapchain_attachments, swapchain_size.0, swapchain_size.1);

        RenderPass { render_pass, device, framebuffers }
    }

    fn create_render_pass(device: &Device, attachments: &[Attachment], subpasses: &[Subpass], dependencies: &[vk::SubpassDependency]) -> vk::RenderPass {
        let mut vk_attachment  = Vec::with_capacity(attachments.len());

        for attachment in attachments.iter() {
            vk_attachment.push(
                vk::AttachmentDescription {
                    format: attachment.format,
                    samples: attachment.samples,
                    load_op: attachment.load_op,
                    store_op: attachment.store_op,
                    initial_layout: attachment.initial_layout,
                    final_layout: attachment.final_layout,

                    ..Default::default()
                }
            );
        }


        let mut vk_subpasses  = Vec::with_capacity(subpasses.len());

        for subpass in subpasses.iter() {
            vk_subpasses.push(
                vk::SubpassDescription {
                    input_attachment_count: subpass.input_attachments.len() as u32,
                    p_input_attachments: subpass.input_attachments.as_ptr(),

                    color_attachment_count: subpass.color_attachments.len() as u32,
                    p_color_attachments: subpass.color_attachments.as_ptr(),

                    p_resolve_attachments: std::ptr::null(), // TODO

                    ..Default::default()
                }
            );
        }

        let render_pass_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,

            attachment_count: vk_attachment.len() as u32,
            p_attachments: vk_attachment.as_ptr(),

            subpass_count: vk_subpasses.len() as u32,
            p_subpasses: vk_subpasses.as_ptr(),

            dependency_count: dependencies.len() as u32,
            p_dependencies: dependencies.as_ptr(),

            ..Default::default()
        };

        unsafe {
            device.get_ash_device().create_render_pass(&render_pass_info, None)
            .expect("Failed to create a render pass")
        }
    }

    fn create_framebuffers(device: &Device, render_pass: vk::RenderPass, attachments: &[vk::ImageView], width: u32, height: u32) -> Vec<vk::Framebuffer> {
        let mut framebuffers = Vec::with_capacity(attachments.len());

        for attachment in attachments.iter() {
            let framebuffer_info = vk::FramebufferCreateInfo {
                s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
                render_pass,
                
                attachment_count: 1,
                p_attachments: std::ptr::from_ref(attachment),

                width,
                height,
                layers: 1,

                ..Default::default()
            };

            framebuffers.push(
                unsafe{
                    device.get_ash_device().create_framebuffer(&framebuffer_info, None).expect("Failed to create a framebuffer")
                }
            );
        }
        
        framebuffers
    }
}
