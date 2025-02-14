pub mod vulkan;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to load library: {0}")]
    LibLoading(anyhow::Error),
    #[error("no suitable device found")]
    NoDeviceAvailable,
    #[error("unsupported codec")]
    UnsupportedCodec,
    #[error("no video queue available")]
    NoVideoQueueAvailable,
    #[error("no compute queue available")]
    NoComputeQueueAvailable,
    #[error("unsupported format")]
    UnsupportedFormat,
    #[error("missing extension")]
    MissingExtension,
    #[error("no matching memory type found")]
    NoMatchingMemoryType,
    #[error(transparent)]
    Other(anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// use anyhow::Result;
// use ash::vk::{
//     self,
//     native::{StdVideoH264ProfileIdc, StdVideoH264ProfileIdc_STD_VIDEO_H264_PROFILE_IDC_BASELINE},
//     ImageUsageFlags, PhysicalDeviceVideoFormatInfoKHR, VideoCapabilitiesKHR,
//     VideoChromaSubsamplingFlagsKHR, VideoCodecOperationFlagsKHR, VideoComponentBitDepthFlagsKHR,
//     VideoEncodeCapabilitiesKHR, VideoEncodeContentFlagsKHR, VideoEncodeH264CapabilitiesKHR,
//     VideoEncodeH264ProfileInfoKHR, VideoEncodeTuningModeKHR, VideoEncodeUsageFlagsKHR,
//     VideoEncodeUsageInfoKHR, VideoFormatPropertiesKHR, VideoProfileInfoKHR,
//     VideoProfileListInfoKHR, VideoSessionCreateInfoKHR,
// };
// use log::info;
// use std::{ptr::null_mut, sync::Arc};
// use vulkano::{
//     device::{
//         physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Queue,
//         QueueCreateInfo, QueueFlags,
//     },
//     image::{ImageFormatInfo, ImageType, ImageUsage},
//     instance::{Instance, InstanceCreateInfo},
//     Handle, VulkanLibrary, VulkanObject,
// };
//
// pub struct Encoder {
//     device: Arc<Device>,
//     encode_queue: Arc<Queue>,
// }
//
// struct Format {}
//
// unsafe fn get_format_info(
//     physical_device: &PhysicalDevice,
//     usage: ImageUsageFlags,
//     profile_list: &VideoProfileListInfoKHR,
// ) -> Result<Format> {
//     let mut profile_list = *profile_list;
//     let mut format_info = PhysicalDeviceVideoFormatInfoKHR::default()
//         .image_usage(usage)
//         .push_next(&mut profile_list);
//     let handle = physical_device.handle();
//     let fns = physical_device.instance().fns();
//     let mut num_formats = 0;
//
//     (fns.khr_video_queue
//         .get_physical_device_video_format_properties_khr)(
//         handle,
//         &format_info,
//         &mut num_formats,
//         null_mut(),
//     )
//     .result()?;
//
//     let mut formats = vec![VideoFormatPropertiesKHR::default(); num_formats as usize];
//
//     (fns.khr_video_queue
//         .get_physical_device_video_format_properties_khr)(
//         handle,
//         &format_info,
//         &mut num_formats,
//         formats.as_mut_ptr(),
//     )
//     .result()?;
//
//     let properties = formats
//         .iter()
//         .filter_map(|fmt| {
//             info!("format: {:#?}", fmt);
//             Some(ImageFormatInfo {
//                 image_type: fmt.image_type.try_into().ok()?,
//                 format: fmt.format.try_into().ok()?,
//                 flags: fmt.image_create_flags.into(),
//                 usage: fmt.image_usage_flags.try_into().ok()?,
//                 tiling: fmt.image_tiling.try_into().ok()?,
//                 ..Default::default()
//             })
//         })
//         .map(|fmt_info| {
//             physical_device
//                 .image_format_properties(fmt_info)
//                 .map_err(Into::into)
//         })
//         .collect::<Result<Vec<_>>>()?;
//
//     info!("properties: {:#?}", properties);
//
//     todo!()
// }
//
// impl Encoder {
//     pub fn new() -> Result<Self> {
//         let library = VulkanLibrary::new()?;
//         let instance = Instance::new(
//             library,
//             InstanceCreateInfo {
//                 enabled_layers: ["VK_LAYER_KHRONOS_validation".to_string()].into(),
//                 ..Default::default()
//             },
//         )?;
//         let physical_device = instance.enumerate_physical_devices()?.next().unwrap();
//         for (index, family) in physical_device.queue_family_properties().iter().enumerate() {
//             info!(
//                 "queue_family[{index}]: count: {:?}, flags: {:?}",
//                 family.queue_count, family.queue_flags
//             );
//         }
//         let encode_queue_family_index = physical_device
//             .queue_family_properties()
//             .iter()
//             .position(|props| props.queue_flags.contains(QueueFlags::VIDEO_ENCODE))
//             .unwrap() as u32;
//         let compute_queue_family_index = physical_device
//             .queue_family_properties()
//             .iter()
//             .position(|props| {
//                 props
//                     .queue_flags
//                     .contains(QueueFlags::COMPUTE | QueueFlags::TRANSFER)
//             })
//             .unwrap() as u32;
//         let (device, mut queues) = Device::new(
//             physical_device.clone(),
//             DeviceCreateInfo {
//                 queue_create_infos: vec![
//                     QueueCreateInfo {
//                         queue_family_index: encode_queue_family_index,
//                         ..Default::default()
//                     },
//                     QueueCreateInfo {
//                         queue_family_index: compute_queue_family_index,
//                         ..Default::default()
//                     },
//                 ],
//                 enabled_extensions: DeviceExtensions {
//                     khr_video_queue: true,
//                     khr_video_encode_queue: true,
//                     ..Default::default()
//                 },
//                 ..Default::default()
//             },
//         )?;
//         let encode_queue = queues.next().unwrap();
//         let compute_queue = queues.next().unwrap();
//         info!("created device and queue");
//
//         let mut h264_info = VideoEncodeH264ProfileInfoKHR::default()
//             .std_profile_idc(StdVideoH264ProfileIdc_STD_VIDEO_H264_PROFILE_IDC_BASELINE);
//         let mut video_encoding_usage_info = VideoEncodeUsageInfoKHR::default()
//             .tuning_mode(VideoEncodeTuningModeKHR::LOW_LATENCY)
//             .video_content_hints(VideoEncodeContentFlagsKHR::DEFAULT)
//             .video_usage_hints(VideoEncodeUsageFlagsKHR::STREAMING);
//         let profile_info = VideoProfileInfoKHR::default()
//             .chroma_subsampling(VideoChromaSubsamplingFlagsKHR::TYPE_420)
//             .chroma_bit_depth(VideoComponentBitDepthFlagsKHR::TYPE_8)
//             .luma_bit_depth(VideoComponentBitDepthFlagsKHR::TYPE_8)
//             .video_codec_operation(VideoCodecOperationFlagsKHR::ENCODE_H264)
//             .push_next(&mut h264_info)
//             .push_next(&mut video_encoding_usage_info);
//         let profile_infos = [profile_info];
//         let profile_list = VideoProfileListInfoKHR::default().profiles(&profile_infos);
//
//         let mut video_encode_h264_caps = VideoEncodeH264CapabilitiesKHR::default();
//         let mut video_encode_caps = VideoEncodeCapabilitiesKHR::default();
//         let mut video_caps = VideoCapabilitiesKHR::default()
//             .push_next(&mut video_encode_caps)
//             .push_next(&mut video_encode_h264_caps);
//         unsafe {
//             (instance
//                 .fns()
//                 .khr_video_queue
//                 .get_physical_device_video_capabilities_khr)(
//                 physical_device.handle(),
//                 &profile_info,
//                 &mut video_caps,
//             )
//             .result()?;
//         }
//         info!("video_caps: {:#?}", video_caps);
//         info!("video_encode_caps: {:#?}", video_encode_caps);
//         info!("video_encode_h264_caps: {:#?}", video_encode_h264_caps);
//
//         unsafe {
//             get_format_info(
//                 &physical_device,
//                 ImageUsageFlags::TRANSFER_DST,
//                 &profile_list,
//             )?;
//         }
//
//         let session_info = VideoSessionCreateInfoKHR::default()
//             .max_active_reference_pictures(1)
//             .max_dpb_slots(2)
//             .max_coded_extent(vk::Extent2D {
//                 width: 1920,
//                 height: 1080,
//             })
//             .video_profile(&profile_info)
//             .queue_family_index(encode_queue_family_index);
//
//         Ok(Encoder {
//             device,
//             encode_queue,
//         })
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         env_logger::builder()
//             .is_test(true)
//             .filter_level(log::LevelFilter::Debug)
//             .try_init()
//             .unwrap();
//         let mut encoder = Encoder::new().unwrap();
//     }
// }
