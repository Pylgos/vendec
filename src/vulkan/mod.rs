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

#[derive(Debug, Clone, Copy)]
pub struct QueueFamilyProperties {
    pub queue_flags: vk::QueueFlags,
    pub queue_count: u32,
    pub timestamp_valid_bits: u32,
    pub min_image_transfer_granularity: vk::Extent3D,
    pub video_codec_operations: vk::VideoCodecOperationFlagsKHR,
}

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

#[cfg(test)]
mod test {
    use super::*;
    use log::info;

    fn draw_color_bar(width: u32, height: u32, frame: u32) -> ::image::RgbaImage {
        let mut image = ::image::RgbaImage::new(width, height);
        let bar_width = width / 7;
        let colors = [
            [192, 192, 192, 255], // White
            [192, 192, 0, 255],   // Yellow
            [0, 192, 192, 255],   // Cyan
            [0, 192, 0, 255],     // Green
            [192, 0, 192, 255],   // Magenta
            [192, 0, 0, 255],     // Red
            [0, 0, 192, 255],     // Blue
        ];

        for (i, color) in colors.iter().enumerate() {
            let start_x = ((i as u32 * bar_width) + frame) % width;
            for x in 0..bar_width {
                for y in 0..height {
                    let pixel_x = (start_x + x) % width;
                    image.put_pixel(pixel_x, y, ::image::Rgba(*color));
                }
            }
        }
        image
    }

    #[test]
    fn test() {
        env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init()
            .unwrap();
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
        let instance = Instance::new().unwrap();
        let physical_devices = instance.enumerate_physical_devices().unwrap();
        for physical_device in &physical_devices {
            info!("{:#?}", physical_device);
        }
        let physical_device = physical_devices[0].clone();
        let queues = Queues::new(physical_device.clone()).unwrap();

        let image = Image::new(
            queues.device.clone(),
            1920,
            1080,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageUsageFlags::STORAGE
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::SAMPLED,
        )
        .unwrap();

        let mut encoder = H264Encoder::new(queues.clone()).unwrap();

        // for i in 0..5 {
        //     let img = draw_color_bar(image.width, image.height, i * 100);
        // }

        // let mut orig_image = ::image::RgbaImage::new(image.width, image.height);
        // draw_color_bar(&mut orig_image);
        // orig_image.save("cpu_image.png").unwrap();
        // let mut cmd_buf = CommandBuffer::new(queues.compute.clone().unwrap()).unwrap();
        // image
        //     .clone()
        //     .cmd_update(
        //         &mut cmd_buf,
        //         &orig_image,
        //         vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        //         vk::PipelineStageFlags2::COMPUTE_SHADER,
        //         vk::AccessFlags2::SHADER_READ,
        //     )
        //     .unwrap();
        // let fence = unsafe { cmd_buf.submit().unwrap() };
        // fence.wait().unwrap();
        //
        // let mut cmd_buf = CommandBuffer::new(queues.compute.clone().unwrap()).unwrap();
        // let buf = image
        //     .clone()
        //     .cmd_download(
        //         &mut cmd_buf,
        //         vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        //         vk::PipelineStageFlags2::COMPUTE_SHADER,
        //         vk::AccessFlags2::SHADER_READ,
        //     )
        //     .unwrap();
        // let fence = unsafe { cmd_buf.submit().unwrap() };
        // fence.wait().unwrap();
        // let read = unsafe { buf.memory.read().unwrap() };
        // let mut downloaded_image = ::image::RgbaImage::new(image.width, image.height);
        // downloaded_image.copy_from_slice(&read[..(image.width * image.height * 4) as usize]);
        // downloaded_image.save("downloaded_image.png").unwrap();
    }
}
