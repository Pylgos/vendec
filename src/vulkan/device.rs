use std::{ffi::CStr, fmt, sync::Arc};

use crate::Result;

use super::PhysicalDevice;

pub struct DeviceExtensions {
    pub(super) video_queue: Option<ash::khr::video_queue::DeviceFn>,
    pub(super) video_encode_queue: Option<ash::khr::video_encode_queue::DeviceFn>,
    pub(super) video_encode_h264: bool,
}

pub struct Device {
    pub(super) handle: ash::Device,
    pub(super) physical_device: Arc<PhysicalDevice>,
    pub(super) extensions: DeviceExtensions,
    pub(super) executor: async_executor::Executor<'static>,
}

impl Device {
    pub fn from_raw(
        physical_device: Arc<PhysicalDevice>,
        handle: ash::Device,
    ) -> Result<Arc<Self>> {
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
            executor: async_executor::Executor::new(),
        }))
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
            .finish()
    }
}
