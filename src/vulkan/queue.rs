use ash::vk;

use super::{Device, PhysicalDevice};
use crate::Result;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct Queue {
    pub(super) handle: ash::vk::Queue,
    pub(super) queue_lock: Mutex<()>,
    pub(super) family_index: u32,
    pub(super) queue_index: u32,
    pub(super) command_pool: ash::vk::CommandPool,
    pub(super) command_pool_lock: Mutex<()>,
    pub(super) device: Arc<Device>,
}

impl Queue {
    pub fn from_raw(
        device: Arc<Device>,
        handle: vk::Queue,
        family_index: u32,
        queue_index: u32,
    ) -> Result<Arc<Self>> {
        let info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(family_index)
            .flags(vk::CommandPoolCreateFlags::TRANSIENT);
        let command_pool = unsafe { device.handle.create_command_pool(&info, None)? };
        Ok(Arc::new(Self {
            handle,
            queue_lock: Mutex::new(()),
            family_index,
            queue_index,
            command_pool,
            command_pool_lock: Mutex::new(()),
            device,
        }))
    }

    pub fn device(&self) -> &Arc<Device> {
        &self.device
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle
                .destroy_command_pool(self.command_pool, None);
        }
    }
}

#[derive(Debug)]
pub struct Queues {
    pub compute: Option<Arc<Queue>>,
    pub h264_encode: Option<Arc<Queue>>,
    pub device: Arc<Device>,
}

impl Queues {
    pub fn new(physical_device: Arc<PhysicalDevice>) -> Result<Arc<Self>> {
        let compute_queue_family_index =
            physical_device.find_queue_family_index(vk::QueueFlags::COMPUTE);
        let h264_encode_queue_family_index = physical_device
            .find_video_queue_family_index(vk::VideoCodecOperationFlagsKHR::ENCODE_H264);

        let mut indices = HashSet::new();
        if let Some(compute_queue_family_index) = compute_queue_family_index {
            indices.insert(compute_queue_family_index);
        }
        if let Some(video_encode_queue_family_index) = h264_encode_queue_family_index {
            indices.insert(video_encode_queue_family_index);
        }

        let queue_create_infos: Vec<_> = indices
            .iter()
            .map(|&index| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(index)
                    .queue_priorities(&[1.0])
            })
            .collect();
        let extension_names = physical_device.supported_extensions.names();
        let mut synchronization2_feature =
            vk::PhysicalDeviceSynchronization2Features::default().synchronization2(true);
        let mut features =
            vk::PhysicalDeviceFeatures2::default().push_next(&mut synchronization2_feature);
        let info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&extension_names)
            .queue_create_infos(&queue_create_infos)
            .push_next(&mut features);
        let device_handle = unsafe {
            physical_device
                .instance
                .handle
                .create_device(physical_device.handle, &info, None)?
        };
        let device = Device::from_raw(physical_device, device_handle)?;

        let compute = if let Some(family_index) = compute_queue_family_index {
            let queue = unsafe { device.handle.get_device_queue(family_index, 0) };
            Some(Queue::from_raw(device.clone(), queue, family_index, 0)?)
        } else {
            None
        };
        let h264_encode = if let Some(family_index) = h264_encode_queue_family_index {
            let queue = unsafe { device.handle.get_device_queue(family_index, 0) };
            Some(Queue::from_raw(device.clone(), queue, family_index, 0)?)
        } else {
            None
        };

        Ok(Arc::new(Self {
            compute,
            h264_encode,
            device,
        }))
    }
}
