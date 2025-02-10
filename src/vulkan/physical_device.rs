use anyhow::bail;
use ash::vk;

use crate::{Error, Result};

use super::{read_into_uninitialized_vector, Instance};
use std::{
    ffi::{c_char, CStr},
    fmt::{self, Debug},
    sync::Arc,
};

#[derive(Debug)]
pub struct SupportedDeviceExtensions {
    pub video_queue: bool,
    pub video_encode_queue: bool,
    pub video_encode_h264: bool,
}

impl SupportedDeviceExtensions {
    pub fn new(extension_properties: &[vk::ExtensionProperties]) -> Self {
        let has_extension = |name: &CStr| {
            extension_properties.iter().any(|props| {
                let ext_name = unsafe { std::ffi::CStr::from_ptr(props.extension_name.as_ptr()) };
                ext_name == name
            })
        };
        Self {
            video_queue: has_extension(ash::khr::video_queue::NAME),
            video_encode_queue: has_extension(ash::khr::video_encode_queue::NAME),
            video_encode_h264: has_extension(ash::khr::video_encode_h264::NAME),
        }
    }

    pub fn names(&self) -> Vec<*const c_char> {
        let mut names = vec![];
        if self.video_queue {
            names.push(ash::khr::video_queue::NAME.as_ptr());
        }
        if self.video_encode_queue {
            names.push(ash::khr::video_encode_queue::NAME.as_ptr());
        }
        if self.video_encode_h264 {
            names.push(ash::khr::video_encode_h264::NAME.as_ptr());
        }
        names
    }
}

pub struct PhysicalDevice {
    pub handle: ash::vk::PhysicalDevice,
    pub instance: Arc<Instance>,
    pub properties: ash::vk::PhysicalDeviceProperties,
    pub queue_family_properties: Vec<ash::vk::QueueFamilyProperties>,
    pub supported_extensions: SupportedDeviceExtensions,
    pub memory_properties: ash::vk::PhysicalDeviceMemoryProperties,
}

impl PhysicalDevice {
    pub fn new(instance: Arc<Instance>, handle: ash::vk::PhysicalDevice) -> Arc<Self> {
        let properties = unsafe { instance.handle.get_physical_device_properties(handle) };
        let queue_family_properties = unsafe {
            instance
                .handle
                .get_physical_device_queue_family_properties(handle)
        };
        let extensions = unsafe {
            instance
                .handle
                .enumerate_device_extension_properties(handle)
                .unwrap()
        };
        let supported_extensions = SupportedDeviceExtensions::new(&extensions);
        let memory_properties = unsafe {
            instance
                .handle
                .get_physical_device_memory_properties(handle)
        };
        Arc::new(Self {
            instance,
            handle,
            properties,
            queue_family_properties,
            supported_extensions,
            memory_properties,
        })
    }

    pub fn find_queue_family_index(&self, queue_flags: vk::QueueFlags) -> Option<u32> {
        self.queue_family_properties
            .iter()
            .enumerate()
            .find(|(_, family)| family.queue_flags.contains(queue_flags) && family.queue_count > 0)
            .map(|(index, _)| index as u32)
    }

    pub fn get_h264_encode_capabilities(
        &self,
        video_profile: &vk::VideoProfileInfoKHR,
    ) -> Result<(
        vk::VideoCapabilitiesKHR,
        vk::VideoEncodeCapabilitiesKHR,
        vk::VideoEncodeH264CapabilitiesKHR,
    )> {
        if !self.supported_extensions.video_encode_h264 {
            return Err(Error::Other(anyhow::anyhow!("H264 encode not supported")));
        }
        let mut h264_encode_caps = vk::VideoEncodeH264CapabilitiesKHR::default();
        let mut encode_caps = vk::VideoEncodeCapabilitiesKHR::default();
        let mut video_caps = vk::VideoCapabilitiesKHR::default()
            .push_next(&mut encode_caps)
            .push_next(&mut h264_encode_caps);
        unsafe {
            (self
                .instance
                .video_queue_fn
                .get_physical_device_video_capabilities_khr)(
                self.handle,
                video_profile,
                &mut video_caps,
            )
            .result()?;
        }
        let video_caps = vk::VideoCapabilitiesKHR {
            _marker: std::marker::PhantomData,
            p_next: std::ptr::null_mut(),
            ..video_caps
        };
        let encode_caps = vk::VideoEncodeCapabilitiesKHR {
            _marker: std::marker::PhantomData,
            p_next: std::ptr::null_mut(),
            ..encode_caps
        };
        let h264_encode_caps = vk::VideoEncodeH264CapabilitiesKHR {
            _marker: std::marker::PhantomData,
            p_next: std::ptr::null_mut(),
            ..h264_encode_caps
        };
        Ok((video_caps, encode_caps, h264_encode_caps))
    }

    pub fn get_video_format_properties(
        &self,
        usage: vk::ImageUsageFlags,
        video_profile_list: &vk::VideoProfileListInfoKHR,
    ) -> Result<Vec<vk::VideoFormatPropertiesKHR>> {
        let mut video_profile_list = *video_profile_list;
        let video_format_info = vk::PhysicalDeviceVideoFormatInfoKHR::default()
            .image_usage(usage)
            .push_next(&mut video_profile_list);
        unsafe {
            let props = read_into_uninitialized_vector(|count, data| {
                (self
                    .instance
                    .video_queue_fn
                    .get_physical_device_video_format_properties_khr)(
                    self.handle,
                    &video_format_info,
                    count,
                    data,
                )
            })?;
            Ok(props)
        }
    }

    pub fn get_image_format_properties2(
        &self,
        info: &vk::PhysicalDeviceImageFormatInfo2,
    ) -> Result<vk::ImageFormatProperties2> {
        let mut props = vk::ImageFormatProperties2::default();
        unsafe {
            self.instance
                .handle
                .get_physical_device_image_format_properties2(self.handle, info, &mut props)?;
        }
        Ok(props)
    }
}

impl Debug for PhysicalDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDevice")
            .field("handle", &self.handle)
            .field("instance", &self.instance)
            .field("properties", &self.properties)
            .field("queue_family_properties", &self.queue_family_properties)
            .field("supported_extensions", &self.supported_extensions)
            .finish()
    }
}
