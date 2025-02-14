use std::{
    ffi::{c_char, CStr},
    fmt,
    sync::Arc,
};

use ash::{
    khr::{video_encode_queue, video_queue},
    vk::{self, Handle},
};
use log::info;

use crate::Result;

use super::physical_device::PhysicalDevice;

pub struct Instance {
    pub(super) entry: ash::Entry,
    pub(super) handle: ash::Instance,
    pub(super) video_queue_fn: ash::khr::video_queue::InstanceFn,
    pub(super) video_encode_queue_fn: ash::khr::video_encode_queue::InstanceFn,
}

impl Instance {
    pub fn new() -> Result<Arc<Self>> {
        unsafe {
            let entry = ash::Entry::load()?;
            let app_name = c"vendec";
            let app_info = ash::vk::ApplicationInfo::default()
                .application_name(app_name)
                .application_version(0)
                .engine_name(app_name)
                .engine_version(0)
                .api_version(ash::vk::make_api_version(0, 1, 3, 0));
            let info = vk::InstanceCreateInfo::default().application_info(&app_info);
            let handle = entry.create_instance(&info, None)?;
            let loader = |name: &CStr| {
                core::mem::transmute(entry.get_instance_proc_addr(handle.handle(), name.as_ptr()))
            };
            let video_queue_fn = video_queue::InstanceFn::load(loader);
            let video_encode_queue_fn = video_encode_queue::InstanceFn::load(loader);
            Ok(Arc::new(Self {
                entry,
                handle,
                video_queue_fn,
                video_encode_queue_fn,
            }))
        }
    }

    pub fn enumerate_physical_devices(self: &Arc<Self>) -> Result<Vec<Arc<PhysicalDevice>>> {
        unsafe {
            let devices = self.handle.enumerate_physical_devices()?;
            let wrapped = devices
                .iter()
                .map(|&handle| PhysicalDevice::from_raw(self.clone(), handle))
                .collect();
            Ok(wrapped)
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.handle.destroy_instance(None);
        }
    }
}

impl fmt::Debug for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Instance")
            .field("handle", &self.handle.handle())
            .finish()
    }
}
