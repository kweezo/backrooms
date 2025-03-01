use ash::{khr, vk, Entry};
use std::{cmp::*, mem::swap};

use super::super::core::*;
use crate::{instance, Window};

const PREFERRED_IMAGE_COUNT: u32 = 3;

pub struct Swapchain<'a> {
    device: &'a Device,

    surface_khr: vk::SurfaceKHR,
    surface_instance: khr::surface::Instance,
    format: vk::SurfaceFormatKHR,

    swapchain: vk::SwapchainKHR,
    swapchain_device: khr::swapchain::Device,
    extent: vk::Extent2D,

    image_views: Vec<vk::ImageView>,

    image_count: u32,
}

pub struct SwapchainInfo {
    pub extent: vk::Extent2D,
    pub image_count: u32,
}

#[inline]
fn clamp(x: u32, min: u32, max: u32) -> u32 {
    if x < min {
        return min;
    } else if x > max{
        return max;
    } 

    x
}

impl<'a> Swapchain<'a> {
    fn choose_surface_format(formats: Vec<vk::SurfaceFormatKHR>) -> vk::SurfaceFormatKHR {
        for format in formats.iter() {
            if format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {

                return *format;
            }
        }

        formats[0]
    }

    fn get_actual_swapchain_size(window: &Window, capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        let extent = if capabilities.current_extent.width
            != clamp(
                capabilities.current_extent.width,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            )
            || capabilities.current_extent.height
                != clamp(
                    capabilities.current_extent.height,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ) {
            vk::Extent2D {
                width: window.get_size().0,
                height: window.get_size().1,
            }
        } else {
            capabilities.current_extent
        };

        return extent;
    }

    fn get_actual_swapchain_image_count(capabilities: &vk::SurfaceCapabilitiesKHR) -> u32 {
        let image_count = if capabilities.max_image_count != 0 {
            clamp(
                PREFERRED_IMAGE_COUNT,
                capabilities.min_image_count,
                capabilities.max_image_count,
            )
        } else {
            min(capabilities.min_image_count, PREFERRED_IMAGE_COUNT)
        };

        image_count
    }

    fn create_swapchain(
        window: &Window,

        device: &Device,
        swapchain_device: &khr::swapchain::Device,

        surface_khr: vk::SurfaceKHR,
        surface_instance: &khr::surface::Instance,

        format: vk::SurfaceFormatKHR,

        old_swapchain: vk::SwapchainKHR
    ) -> (vk::SwapchainKHR, vk::Extent2D, u32) {

        let capabilities = unsafe {
            surface_instance.get_physical_device_surface_capabilities(
                device.get_vk_physical_device(),
                surface_khr,
            )
        }
        .expect("Failed to get the physical device surface capabilities");

        let extent = Swapchain::get_actual_swapchain_size(window, &capabilities);
        let image_count = Swapchain::get_actual_swapchain_image_count(&capabilities);
        let family_indices = device.get_queue_family_indices(vec![QueueType::BOTH]);

        let create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
            surface: surface_khr,

            min_image_count: image_count,
            image_color_space: format.color_space,
            image_extent: extent,
            image_array_layers: 1u32,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | ash::vk::ImageUsageFlags::TRANSFER_DST,
            image_sharing_mode: ash::vk::SharingMode::EXCLUSIVE,
            image_format: format.format,

            queue_family_index_count: family_indices.len() as u32,
            p_queue_family_indices: family_indices.as_ptr(),

            pre_transform: capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            present_mode: vk::PresentModeKHR::FIFO,

            clipped: true as u32, // because yes
            old_swapchain: old_swapchain,

            ..Default::default()
        };

        let swapchain = unsafe { swapchain_device.create_swapchain(&create_info, None) }
            .expect("Failed to create the swapchain");

        (swapchain, extent, image_count)

    }

    fn create_swapchain_image_views(
        device: &Device,
        swapchain_device: &ash::khr::swapchain::Device,
        swapchain: vk::SwapchainKHR,
        format: vk::Format,
    ) -> Vec<vk::ImageView> {

        let images = unsafe { swapchain_device.get_swapchain_images(swapchain) }
            .expect("Failed to retrieve the swapchain images");
        let mut image_views: Vec<ash::vk::ImageView> = Vec::with_capacity(images.len());

        let mut image_view_info = ash::vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            components: vk::ComponentMapping::default(),
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };

        for image in images {
            image_view_info.image = image;

            image_views.push(
                unsafe {
                    device
                        .get_ash_device()
                        .create_image_view(&image_view_info, None)
                }
                .expect("Failed to create a swapchain image view"),
            );
        }

        image_views
    }

    pub fn create_surface(window: &Window, instance: &Instance) -> vk::SurfaceKHR {
        let mut surface_khr: vk::SurfaceKHR = vk::SurfaceKHR::null();

        window
            .get_window()
            .create_window_surface(
                instance.get_ash_instance().handle(),
                std::ptr::null(),
                &mut surface_khr,
            )
            .result()
            .expect("Failed to create the VkSurfaceKHR");

        surface_khr
    }

    pub fn new(
        entry: &Entry,
        window: &Window,
        instance: &Instance,
        device: &'a Device,
    ) -> Swapchain<'a> {

        let surface_khr = Swapchain::create_surface(window, instance);

        let surface_instance = ash::khr::surface::Instance::new(entry, instance.get_ash_instance());

        let format = Swapchain::choose_surface_format(unsafe {
            surface_instance
                .get_physical_device_surface_formats(device.get_vk_physical_device(), surface_khr)
                .expect("Failed to get physsical device surface formats")
        });

        let swapchain_device = khr::swapchain::Device::new(instance.get_ash_instance(), device.get_ash_device());

        let (swapchain, extent, image_count) = Swapchain::create_swapchain(
            window,
            device,
            &swapchain_device,
            surface_khr,
            &surface_instance,
            format,
            vk::SwapchainKHR::null()
        );
        let image_views = Swapchain::create_swapchain_image_views(
            device,
            &swapchain_device,
            swapchain,
            format.format,
        );

        Swapchain { device, surface_khr, surface_instance, format, swapchain, swapchain_device, extent, image_views, image_count }
    }

    pub fn recreate(&mut self, window: &Window, instance: &Instance) {
        (self.swapchain, self.extent, _) = Swapchain::create_swapchain(window, &self.device, &self.swapchain_device, self.surface_khr, &self.surface_instance, self.format, self.swapchain);

        self.image_views = Swapchain::create_swapchain_image_views(&self.device, &self.swapchain_device, self.swapchain, self.format.format);
    }

    pub fn get_image_views(&self) -> &Vec<vk::ImageView> {
        &self.image_views
    }

    pub fn get_physical_device_surface_capabilities(&self, physical_device: vk::PhysicalDevice) -> vk::SurfaceCapabilitiesKHR{
        unsafe{
            self.surface_instance.get_physical_device_surface_capabilities(physical_device, self.surface_khr)
        }.expect("Failed to get the physical device surface capabilities")
    }
    }
