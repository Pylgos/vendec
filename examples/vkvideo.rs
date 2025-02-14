use std::time::Duration;

use ash::vk;
use vendec::vulkan::*;

fn draw_color_bar(cpu_image: &mut ::image::RgbaImage) {
    for y in 0..cpu_image.height() {
        for x in 0..cpu_image.width() {
            let r = (x as f32 / cpu_image.width() as f32 * 255.0) as u8;
            let g = (y as f32 / cpu_image.height() as f32 * 255.0) as u8;
            let b = 0;
            if x < 10 || x >= cpu_image.width() - 10 || y < 10 || y >= cpu_image.height() - 10 {
                cpu_image.put_pixel(x, y, ::image::Rgba([0, 0, 0, 255]));
            } else {
                cpu_image.put_pixel(x, y, ::image::Rgba([r, g, b, 255]));
            }
        }
    }
}

fn main() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
    let instance = Instance::new().unwrap();
    let physical_devices = instance.enumerate_physical_devices().unwrap();
    for device in &physical_devices {
        log::debug!("{:#?}", device);
    }
    let physical_device = physical_devices[0].clone();
    let queues = Queues::new(physical_device.clone()).unwrap();
    // log::debug!("{:#?}", queues);
    // let encoder = H264Encoder::new(queues.clone()).unwrap();

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
    let mut orig_image = ::image::RgbaImage::new(image.width, image.height);
    draw_color_bar(&mut orig_image);
    orig_image.save("cpu_image.png").unwrap();
    let mut cmd_buf = CommandBuffer::new(queues.compute.clone().unwrap()).unwrap();
    image
        .clone()
        .cmd_update(
            &mut cmd_buf,
            &orig_image,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::PipelineStageFlags2::COMPUTE_SHADER,
            vk::AccessFlags2::SHADER_READ,
        )
        .unwrap();
    let fence = unsafe { cmd_buf.submit().unwrap() };
    fence.wait().unwrap();

    let mut cmd_buf = CommandBuffer::new(queues.compute.clone().unwrap()).unwrap();
    let buf = image
        .clone()
        .cmd_download(
            &mut cmd_buf,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::PipelineStageFlags2::COMPUTE_SHADER,
            vk::AccessFlags2::SHADER_READ,
        )
        .unwrap();
    let fence = unsafe { cmd_buf.submit().unwrap() };
    fence.wait().unwrap();
    let read = unsafe { buf.memory.read().unwrap() };
    let mut downloaded_image = ::image::RgbaImage::new(image.width, image.height);
    downloaded_image.copy_from_slice(&read[..(image.width * image.height * 4) as usize]);
    downloaded_image.save("downloaded_image.png").unwrap();
}
