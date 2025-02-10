use std::{ffi::CStr, fmt, ptr::null, sync::Arc};

use ash::vk;
use log::info;

use crate::{Error, Result};

use super::{memory::Memory, PhysicalDevice};

pub struct DeviceExtensions {
    pub video_queue: Option<ash::khr::video_queue::DeviceFn>,
    pub video_encode_queue: Option<ash::khr::video_encode_queue::DeviceFn>,
    pub video_encode_h264: bool,
}

#[derive(Debug)]
pub struct Queue {
    pub handle: ash::vk::Queue,
    pub family_index: u32,
    pub queue_index: u32,
}

pub struct Device {
    pub handle: ash::Device,
    pub physical_device: Arc<PhysicalDevice>,
    pub extensions: DeviceExtensions,
    pub encode_queue: Option<Queue>,
    pub compute_queue: Option<Queue>,
}

impl Device {
    pub fn new(physical_device: Arc<PhysicalDevice>) -> Result<Arc<Self>> {
        let encode_queue_family_index =
            physical_device.find_queue_family_index(vk::QueueFlags::VIDEO_ENCODE_KHR);
        let compute_queue_family_index =
            physical_device.find_queue_family_index(vk::QueueFlags::COMPUTE);
        let mut queue_create_infos = vec![];
        if let Some(video_encode_queue_family_index) = encode_queue_family_index {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(video_encode_queue_family_index)
                    .queue_priorities(&[1.0]),
            );
        }
        if let Some(compute_queue_family_index) = compute_queue_family_index {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(compute_queue_family_index)
                    .queue_priorities(&[1.0]),
            );
        }
        let extension_names = physical_device.supported_extensions.names();
        let info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&extension_names)
            .queue_create_infos(&queue_create_infos);
        let handle = unsafe {
            physical_device
                .instance
                .handle
                .create_device(physical_device.handle, &info, None)?
        };
        let encode_queue = encode_queue_family_index.map(|family_index| Queue {
            handle: unsafe { handle.get_device_queue(family_index, 0) },
            family_index,
            queue_index: 0,
        });
        let compute_queue = compute_queue_family_index.map(|family_index| Queue {
            handle: unsafe { handle.get_device_queue(family_index, 0) },
            family_index,
            queue_index: 0,
        });
        let loader = |name: &CStr| unsafe {
            core::mem::transmute(
                physical_device
                    .instance
                    .handle
                    .get_device_proc_addr(handle.handle(), name.as_ptr()),
            )
        };
        let extensions = DeviceExtensions {
            video_queue: if physical_device.supported_extensions.video_queue {
                Some(ash::khr::video_queue::DeviceFn::load(loader))
            } else {
                None
            },
            video_encode_queue: if physical_device.supported_extensions.video_encode_queue {
                Some(ash::khr::video_encode_queue::DeviceFn::load(loader))
            } else {
                None
            },
            video_encode_h264: physical_device.supported_extensions.video_encode_h264,
        };
        Ok(Arc::new(Self {
            handle,
            physical_device,
            extensions,
            compute_queue,
            encode_queue,
        }))
    }

    pub fn allocate_memory(
        self: Arc<Self>,
        required_properties: vk::MemoryPropertyFlags,
        requirements: &vk::MemoryRequirements,
    ) -> Result<Arc<Memory>> {
        for memory_type_index in 0..self.physical_device.memory_properties.memory_type_count {
            if requirements.memory_type_bits & (1 << memory_type_index) == 0 {
                continue;
            }
            let properties = self.physical_device.memory_properties.memory_types
                [memory_type_index as usize]
                .property_flags;
            if !properties.contains(required_properties) {
                continue;
            }
            info!(
                "Allocating memory of size {} with memory type index {} properties {:?}",
                requirements.size, memory_type_index, properties
            );
            let info = vk::MemoryAllocateInfo::default()
                .allocation_size(requirements.size)
                .memory_type_index(memory_type_index);
            let handle = unsafe { self.handle.allocate_memory(&info, None)? };
            return Ok(Arc::new(Memory {
                handle,
                device: self,
                size: requirements.size,
                memory_type_index,
            }));
        }
        Err(Error::NoMatchingMemoryType)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_device(None);
        }
    }
}

impl fmt::Debug for DeviceExtensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! opaque_opt {
            ($opt:expr) => {
                match $opt {
                    Some(_) => format_args!("Some(...)"),
                    None => format_args!("None"),
                }
            };
        }
        f.debug_struct("DeviceExtensions")
            .field("video_queue", &opaque_opt!(self.video_queue))
            .field("video_encode_queue", &opaque_opt!(self.video_encode_queue))
            .field("video_encode_h264", &self.video_encode_h264)
            .finish()
    }
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Device")
            .field("handle", &self.handle.handle())
            .field("physical_device", &self.physical_device)
            .field("extensions", &self.extensions)
            .field("encode_queue", &self.encode_queue)
            .field("compute_queue", &self.compute_queue)
            .finish()
    }
}
