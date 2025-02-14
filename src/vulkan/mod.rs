// use std::{ffi::c_char, sync::Arc};

use std::ptr;

use ash::vk;

mod buffer;
mod command_buffer;
mod device;
mod encoder;
mod image;
mod instance;
mod memory;
mod physical_device;
mod queue;

pub use buffer::*;
pub use command_buffer::*;
pub use device::*;
pub use encoder::*;
pub use image::*;
pub use instance::*;
pub use memory::*;
pub use physical_device::*;
pub use queue::*;

use crate::Error;

impl From<vk::Result> for Error {
    fn from(result: vk::Result) -> Self {
        Self::Other(result.into())
    }
}

impl From<ash::LoadingError> for Error {
    fn from(error: ash::LoadingError) -> Self {
        Self::LibLoading(error.into())
    }
}

unsafe fn read_into_vector<T: Default + Clone>(
    f: impl Fn(&mut u32, *mut T) -> vk::Result,
) -> ash::prelude::VkResult<Vec<T>> {
    loop {
        let mut count = 0u32;
        f(&mut count, ptr::null_mut()).result()?;
        let mut data = vec![T::default(); count as _];

        let err_code = f(&mut count, data.as_mut_ptr());
        if err_code != vk::Result::INCOMPLETE {
            if err_code == vk::Result::SUCCESS {
                data.set_len(count as _);
                return Ok(data);
            } else {
                return Err(err_code.result().unwrap_err());
            }
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
// }
// pub struct Encoder {}

// impl Encoder {
//     pub fn new() -> Result<Self, Error> {
//         let instance = Instance::new()?;
//         let device_extensions = [
//             ash::khr::synchronization2::NAME,
//             ash::khr::video_queue::NAME,
//             ash::khr::video_encode_queue::NAME,
//         ];
//         let queue_flags = [
//             vk::QueueFlags::VIDEO_ENCODE_KHR,
//             vk::QueueFlags::COMPUTE | vk::QueueFlags::TRANSFER,
//         ];
//         let physical_devices = instance.find_physical_devices(&device_extensions, &queue_flags)?;
//         if physical_devices.is_empty() {
//             return Err(Error::NoDeviceAvailable);
//         }
//         let physical_device = physical_devices[0].clone();
//         let queue_family_index = physical_device.find_queue_family_indices([
//             vk::QueueFlags::VIDEO_ENCODE_KHR,
//             vk::QueueFlags::COMPUTE | vk::QueueFlags::TRANSFER,
//         ]);
//         let encode_queue_family_index = queue_family_index[0].unwrap();
//         let compute_queue_family_index = queue_family_index[1].unwrap();
//         let device = physical_device.create_device(&device_extensions, &queue_flags)?;
//         Ok(Self {})
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn test_encoder() {
//         env_logger::builder()
//             .is_test(true)
//             .filter_level(log::LevelFilter::Debug)
//             .try_init()
//             .unwrap();
//         let encoder = Encoder::new().unwrap();
//     }
// }
