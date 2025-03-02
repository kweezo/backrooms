use ash::vk;
use std::{
    collections::HashSet, os::raw::c_void, pin::Pin, sync::{Arc, RwLock}
};

use super::{device_config, physical_device, QueueFamily, QueueType};


pub struct Device {
    device: ash::Device,
    physical_device: vk::PhysicalDevice,
    curr_queue_index: RwLock<u32>,

    graphics_family: QueueFamily,
    transfer_family: Option<QueueFamily>,

    allocator: Arc<RwLock<Arc<vk_mem::Allocator>>>,
}

impl Device {

    fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        graphics_family: &QueueFamily,
        transfer_family: &Option<QueueFamily>
    ) -> ash::Device {

        let mut queue_infos = Vec::<vk::DeviceQueueCreateInfo>::new();
        let (grahics_family_create_info, _graphics_priorities) = graphics_family.get_queue_create_info();
        let _transfer_priorities: Pin<Vec<f32>>;

        if matches!(transfer_family, Some(_)) {
            let transfer_family_create_info;
            (transfer_family_create_info, _transfer_priorities) = transfer_family.as_ref().unwrap().get_queue_create_info();

            queue_infos.push(transfer_family_create_info);
        }

        queue_infos.push(grahics_family_create_info);


        let (extensions, _extensions_ptr) = device_config::get_extensions(instance, physical_device);

        let mut descriptor_indexing_features = vk::PhysicalDeviceDescriptorIndexingFeatures {
            s_type: vk::StructureType::PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES,

            runtime_descriptor_array: true as u32,

            descriptor_binding_partially_bound: true as u32,
            descriptor_binding_variable_descriptor_count: true as u32,

            shader_storage_buffer_array_non_uniform_indexing: true as u32,
            shader_sampled_image_array_non_uniform_indexing: true as u32,
            shader_storage_texel_buffer_array_non_uniform_indexing: true as u32,

            descriptor_binding_storage_buffer_update_after_bind: true as u32,
            descriptor_binding_sampled_image_update_after_bind: true as u32,
            descriptor_binding_storage_image_update_after_bind: true as u32,
            descriptor_binding_uniform_buffer_update_after_bind: true as u32,

            ..Default::default()
        };

        let device_features = vk::PhysicalDeviceFeatures2 {
            s_type: vk::StructureType::PHYSICAL_DEVICE_FEATURES_2,
            p_next: &mut descriptor_indexing_features as *mut _ as *mut c_void,
            ..Default::default()
        };

        let create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: &device_features as *const _ as *const c_void,

            queue_create_info_count: queue_infos.len() as u32,
            p_queue_create_infos: queue_infos.as_ptr(),

            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),

            ..Default::default()
        };

        unsafe { instance.create_device(physical_device, &create_info, None) }
            .expect("Failed to create the Vulkan device")
    }

    fn get_device_queues(
        device: &ash::Device,
        graphics_family: &mut QueueFamily,
        transfer_family: &mut Option<QueueFamily>) {

        graphics_family.fill_with_queues(device);

        if matches!(transfer_family, Some(_)) {
            transfer_family.as_mut().unwrap().fill_with_queues(device);
        }
    }

    fn create_allocator(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
    ) -> vk_mem::Allocator {
        let create_info = vk_mem::AllocatorCreateInfo::new(instance, device, physical_device);

        unsafe {
            vk_mem::Allocator::new(create_info).expect("Failed to create a new VMA allocator")
        }
    }

    fn pick_queue_families(families: Vec<QueueFamily>) -> (QueueFamily, Option<QueueFamily>) {
        let mut graphics: Option<QueueFamily> = None;
        let mut transfer: Option<QueueFamily> = None;

        for family in families {
            if matches!(family.get_type(), QueueType::BOTH) {
                graphics = Some(family)
            } else if matches!(family.get_type(), QueueType::TRANSFER){
                transfer = Some(family);
            }
        }

        (graphics.unwrap(), transfer)
    }

    pub fn new(instance: &ash::Instance) -> Device {
        let physical_device = physical_device::pick_physical_device(instance);

        let queue_families = QueueFamily::new(instance, physical_device);
        let (mut graphics_family, mut transfer_family) =
         Device::pick_queue_families(queue_families);


        let device = Device::create_logical_device(
            instance,
            physical_device,
            &graphics_family,
            &transfer_family
        );

        Device::get_device_queues(&device, &mut graphics_family, &mut transfer_family);

        let allocator = Device::create_allocator(instance, physical_device, &device);

        Device {
            device,
            physical_device,
            graphics_family,
            transfer_family,
            curr_queue_index: RwLock::new(0),
            allocator: Arc::new(RwLock::new(Arc::new(allocator))),
        }
    }

    pub fn get_ash_device(&self) -> &ash::Device {
        &self.device
    }

    pub fn get_vk_physical_device(&self) -> ash::vk::PhysicalDevice {
        self.physical_device
    }

    pub fn get_allocator(&self) -> Arc<vk_mem::Allocator> {
        self.allocator.read().unwrap().clone()
    }

    pub fn get_allocator_lock(&self) -> Arc<RwLock<Arc<vk_mem::Allocator>>> {
        self.allocator.clone()
    }

    fn pick_queue_family(&self, queue_type: QueueType) -> &QueueFamily {

        if matches!(queue_type, QueueType::TRANSFER) && matches!(self.transfer_family, Some(_)) {
            return &self.transfer_family.as_ref().unwrap();
        } 

        &self.graphics_family
    }

    pub fn get_queue(&self, queue_type: QueueType) -> ash::vk::Queue {
        if matches!(queue_type, QueueType::TRANSFER) && matches!(self.transfer_family, Some(_)) {
            return self.transfer_family.as_ref().unwrap().get_queue();
        } 

        self.graphics_family.get_queue()
    }

    pub fn get_queue_family_indices(&self, queue_types: Vec<QueueType>) -> Vec<u32> {
        let mut indices = HashSet::<u32>::with_capacity(queue_types.len());

        for queue_type in queue_types {
            if matches!(queue_type, QueueType::TRANSFER) {
                if matches!(self.transfer_family, Some(_)) {
                    indices.insert(self.transfer_family.as_ref().unwrap().get_index());
                } else {
                    indices.insert(self.graphics_family.get_index());
                }

                continue;
            } 

            indices.insert(self.graphics_family.get_index());
        }

        indices.into_iter().collect()
    }
}
