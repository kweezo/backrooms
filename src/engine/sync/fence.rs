use ash::vk;

use crate::engine::Device;

pub struct Fence {
    fence: vk::Fence
}

impl Fence {
    pub fn new(device: &Device, signaled: bool) -> Fence {
        let fence = Fence::create_fence(device, signaled);

        Fence { fence }
    }

    fn create_fence(device: &Device, signaled: bool) -> vk::Fence {
        let flags = if signaled {
            vk::FenceCreateFlags::SIGNALED
        } else {
            vk::FenceCreateFlags::empty()
        };

        let create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags,

            ..Default::default()
        };

        unsafe {
            device.get_ash_device().create_fence(&create_info, None)
        }.expect("Failed to create a fence")
    }

    pub fn get_fence(&self) -> vk::Fence {
        return self.fence;
    }
}