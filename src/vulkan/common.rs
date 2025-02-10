use ash::vk;
use std::{
    ffi::{c_char, CStr},
    sync::Arc,
};

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct Capabilities {
    pub h264_encode: bool,
}

impl Capabilities {
    pub fn instance_extensions(&self) -> Vec<&'static CStr> {
        let mut extensions = vec![ash::khr::video_queue::NAME];
        if self.h264_encode {
            extensions.push(ash::khr::video_encode_queue::NAME);
        }
        extensions
    }

    pub fn device_extensions(&self) -> Vec<&'static CStr> {
        let mut extensions = vec![ash::khr::synchronization2::NAME];
        if self.h264_encode {
            extensions.push(ash::khr::video_encode_queue::NAME);
        }
        if self.h264_encode {
            extensions.push(ash::khr::video_encode_h264::NAME);
        }
        extensions
    }
}

pub struct Instance {
    pub entry: ash::Entry,
    pub handle: ash::Instance,
    pub video_queue_ext: ash::khr::video_queue::Instance,
    pub video_encode_queue_ext: ash::khr::video_encode_queue::Instance,
}

pub struct PhysicalDevice {
    pub instance: Arc<Instance>,
    pub handle: vk::PhysicalDevice,
}

pub struct Device {
    pub instance: Arc<Instance>,
    pub handle: ash::Device,
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_instance(None);
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_device(None);
        }
    }
}

impl Instance {
    pub fn new() -> Result<Arc<Self>> {
        unsafe {
            let entry = ash::Entry::load()?;
            let layers: Vec<*const c_char> = [c"VK_LAYER_KHRONOS_validation"]
                .iter()
                .map(|&s| s.as_ptr())
                .collect();
            // let extensions: Vec<*const c_char> = [].iter().map(|&s| s.as_ptr()).collect();
            let app_name = c"vendec";
            let app_info = ash::vk::ApplicationInfo::default()
                .application_name(app_name)
                .application_version(0)
                .engine_name(app_name)
                .engine_version(0)
                .api_version(ash::vk::make_api_version(0, 1, 3, 0));
            let info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_layer_names(&layers);
            let handle = entry.create_instance(&info, None)?;
            let video_queue_ext = ash::khr::video_queue::Instance::new(&entry, &handle);
            let video_encode_queue_ext =
                ash::khr::video_encode_queue::Instance::new(&entry, &handle);
            Ok(Arc::new(Self {
                entry,
                handle,
                video_queue_ext,
                video_encode_queue_ext,
            }))
        }
    }

    pub fn enumerate_physical_devices(self: Arc<Self>) -> Result<Vec<Arc<PhysicalDevice>>> {
        unsafe {
            let devices = self.handle.enumerate_physical_devices()?;
            let wrapped = devices
                .iter()
                .map(|&handle| {
                    Arc::new(PhysicalDevice {
                        instance: self.clone(),
                        handle,
                    })
                })
                .collect();
            Ok(wrapped)
        }
    }

    pub fn find_physical_devices(
        self: Arc<Self>,
        required_extensions: &[&CStr],
        required_queue_flags: &[vk::QueueFlags],
    ) -> Result<Vec<Arc<PhysicalDevice>>> {
        let physical_devices = self.enumerate_physical_devices()?;
        let mut filtered = Vec::new();
        for pdev in physical_devices {
            let extensions = pdev.enumerate_device_extension_properties()?;
            let queue_family_indices =
                pdev.find_queue_family_indices(required_queue_flags.iter().copied());
            if required_extensions.iter().all(|&ext| {
                extensions.iter().any(|props| {
                    let name = unsafe { CStr::from_ptr(props.extension_name.as_ptr()) };
                    name == ext
                })
            }) && queue_family_indices.iter().all(|&idx| idx.is_some())
            {
                filtered.push(pdev);
            }
        }
        Ok(filtered)
    }
}

impl PhysicalDevice {
    pub fn queue_family_properties(&self) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.instance
                .handle
                .get_physical_device_queue_family_properties(self.handle)
        }
    }

    pub fn find_queue_family_indices(
        &self,
        flags_iter: impl IntoIterator<Item = vk::QueueFlags>,
    ) -> Vec<Option<u32>> {
        flags_iter
            .into_iter()
            .map(move |flags| {
                self.queue_family_properties()
                    .iter()
                    .enumerate()
                    .find(|(_, props)| props.queue_flags.contains(flags))
                    .map(|(idx, _)| idx as u32)
            })
            .collect()
    }

    pub fn enumerate_device_extension_properties(&self) -> Result<Vec<vk::ExtensionProperties>> {
        unsafe {
            let props = self
                .instance
                .handle
                .enumerate_device_extension_properties(self.handle)?;
            Ok(props)
        }
    }

    pub fn create_device(
        self: Arc<Self>,
        enabled_extensions: &[&CStr],
        queue_flags: &[vk::QueueFlags],
    ) -> Result<Arc<Device>> {
        unsafe {
            let queue_create_infos = queue_flags
                .iter()
                .map(|&flags| {
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(
                            self.find_queue_family_indices(std::iter::once(flags))[0].unwrap(),
                        )
                        .queue_priorities(&[1.0])
                })
                .collect::<Vec<_>>();
            let enable_extensions = enabled_extensions
                .iter()
                .map(|&ext| ext.as_ptr())
                .collect::<Vec<_>>();
            let info = vk::DeviceCreateInfo::default()
                .enabled_extension_names(&enable_extensions)
                .queue_create_infos(&queue_create_infos);
            let handle = self
                .instance
                .handle
                .create_device(self.handle, &info, None)?;
            Ok(Arc::new(Device {
                instance: self.instance.clone(),
                handle,
            }))
        }
    }
}
