use ash::vk;

pub fn pick_physical_device(instance: &ash::Instance) -> vk::PhysicalDevice {

    //TODO better selection system
    let devices = unsafe { instance.enumerate_physical_devices() }
        .expect("Failed to enumerate the list of physical devices");

    let mut best_device: Option<vk::PhysicalDevice> = None;

    for device in devices.iter() {
        let properties = unsafe { instance.get_physical_device_properties(*device) };

        if vk::PhysicalDeviceType::DISCRETE_GPU == properties.device_type {
            best_device = Some(*device);
        }
    }

    match best_device {
        None => devices[0],
        Some(device) => device,
    }
}