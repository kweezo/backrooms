use ash::vk;

use crate::engine::Device;

pub struct DescriptorSet<'a> {
    set: vk::DescriptorSet,
    pool: vk::DescriptorPool,

    device: &'a Device
}

impl<'a> DescriptorSet<'a> {
    pub fn new(device: &'a Device, descriptors: &[(vk::DescriptorType, u32, u32)]) {
        let pool = DescriptorSet::create_pool(device, descriptors);
        let set = DescriptorSet::create_set(device, pool, descriptors);
    }

    fn create_pool(device: &'a Device, descriptors: &[(vk::DescriptorType, u32, u32)]) -> vk::DescriptorPool{
        let mut pool_sizes = Vec::with_capacity(descriptors.len());

        for descriptor in descriptors {
            pool_sizes.push(
                vk::DescriptorPoolSize {
                    ty: descriptor.0,
                    descriptor_count: descriptor.1
                }
            );
        }

        let pool_info = vk::DescriptorPoolCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,


            max_sets: std::u32::MAX,
            p_pool_sizes: pool_sizes.as_ptr(),
            pool_size_count: pool_sizes.len() as u32,

            ..Default::default()
        };

       unsafe {
            device.get_ash_device().create_descriptor_pool(&pool_info, None).expect("Failed to create a descriptor pool")
       } 
    }

    fn create_set(device: &'a Device, pool: vk::DescriptorPool, descriptors: &[(vk::DescriptorType, u32, u32)]) -> vk::DescriptorSet {
        let layout = DescriptorSet::create_layout(device, descriptors);

        let alloc_info = vk::DescriptorSetAllocateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,

            descriptor_pool: pool,

            descriptor_set_count: 1,
            p_set_layouts: [layout].as_ptr(),

            ..Default::default()
        };

        let set = unsafe{
            device.get_ash_device().allocate_descriptor_sets(&alloc_info)
        }.expect("Failed to alloc descriptor sets")[0];

        unsafe {
            device.get_ash_device().destroy_descriptor_set_layout(layout, None);
        }

        set
    }

    fn create_layout(device: &'a Device, descriptors: &[(vk::DescriptorType, u32, u32)]) -> vk::DescriptorSetLayout {
        let mut layout_bindings = Vec::with_capacity(descriptors.len());

        for descriptor in descriptors {
            layout_bindings.push(
                vk::DescriptorSetLayoutBinding {
                    binding: descriptor.2,
                    descriptor_type: descriptor.0,
                    descriptor_count: descriptor.1,
                    stage_flags: vk::ShaderStageFlags::ALL,

                    ..Default::default()
                }
            );
        }

        let layout_info = vk::DescriptorSetLayoutCreateInfo {
            s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,

            binding_count: layout_bindings.len() as u32,
            p_bindings: layout_bindings.as_ptr(),

            ..Default::default()
        };

        unsafe {
            device.get_ash_device().create_descriptor_set_layout(&layout_info, None).expect("Failed to create set layout")
        } 
    }
}


impl<'a> Drop for DescriptorSet<'a> {
    fn drop(&mut self) {
        unsafe {
            self.device.get_ash_device().destroy_descriptor_pool(self.pool, None);
        }
    }
}