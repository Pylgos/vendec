use ash::vk;
use log::info;
use std::{ptr::null, sync::Arc};

use super::{memory::Memory, read_into_uninitialized_vector, Device};
use crate::{Error, Result};

pub struct H264Encoder {
    device: Arc<Device>,
}

impl H264Encoder {
    pub fn new(device: Arc<Device>) -> Result<Arc<Self>> {
        if device.extensions.video_queue.is_none()
            || device.extensions.video_encode_queue.is_none()
            || !device.extensions.video_encode_h264
            || device.encode_queue.is_none()
        {
            return Err(Error::UnsupportedCodec);
        }

        let mut h264_profile = vk::VideoEncodeH264ProfileInfoKHR::default().std_profile_idc(
            ash::vk::native::StdVideoH264ProfileIdc_STD_VIDEO_H264_PROFILE_IDC_HIGH,
        );
        let mut usage_info = vk::VideoEncodeUsageInfoKHR::default()
            .tuning_mode(vk::VideoEncodeTuningModeKHR::LOW_LATENCY)
            .video_content_hints(vk::VideoEncodeContentFlagsKHR::DEFAULT)
            .video_usage_hints(vk::VideoEncodeUsageFlagsKHR::STREAMING);
        let profile_info = vk::VideoProfileInfoKHR::default()
            .chroma_subsampling(vk::VideoChromaSubsamplingFlagsKHR::TYPE_420)
            .chroma_bit_depth(vk::VideoComponentBitDepthFlagsKHR::TYPE_8)
            .luma_bit_depth(vk::VideoComponentBitDepthFlagsKHR::TYPE_8)
            .video_codec_operation(vk::VideoCodecOperationFlagsKHR::ENCODE_H264)
            .push_next(&mut h264_profile)
            .push_next(&mut usage_info);
        let profile_infos = [profile_info];
        let profile_list = vk::VideoProfileListInfoKHR::default().profiles(&profile_infos);

        let dpb_format = find_pre_encode_format(&device, &profile_list)?;
        let pre_encode_format = find_pre_encode_format(&device, &profile_list)?;

        info!("dpb_format: {:#?}", dpb_format);
        info!("input_format: {:#?}", pre_encode_format);

        let (video_caps, encode_caps, h264_encode_caps) = device
            .physical_device
            .get_h264_encode_capabilities(&profile_info)?;

        info!("video_caps: {:#?}", video_caps);
        info!("encode_caps: {:#?}", encode_caps);
        info!("h264_encode_caps: {:#?}", h264_encode_caps);

        let session_info = vk::VideoSessionCreateInfoKHR::default()
            .max_active_reference_pictures(1)
            .max_coded_extent(video_caps.max_coded_extent)
            .max_dpb_slots(2)
            .video_profile(&profile_info)
            .queue_family_index(device.encode_queue.as_ref().unwrap().family_index)
            .picture_format(pre_encode_format.format)
            .reference_picture_format(dpb_format.format)
            .std_header_version(&video_caps.std_header_version);
        let session = unsafe { VideoSession::new(device.clone(), &session_info)? };

        Ok(Arc::new(Self { device }))
    }
}

#[derive(Debug)]
struct PreEncodeFormat {
    format: vk::Format,
    luma_format: vk::Format,
    chroma_format: vk::Format,
    properties: vk::ImageFormatProperties,
}

fn split_luma_chroma(format: vk::Format) -> Option<(vk::Format, vk::Format)> {
    match format {
        vk::Format::G8_B8R8_2PLANE_420_UNORM_KHR => {
            Some((vk::Format::R8_UNORM, vk::Format::R8G8_UNORM))
        }
        _ => None,
    }
}

fn find_pre_encode_format(
    device: &Device,
    profile_list: &vk::VideoProfileListInfoKHR,
) -> Result<PreEncodeFormat> {
    let usage = vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::VIDEO_ENCODE_SRC_KHR;
    let video_props_list = device
        .physical_device
        .get_video_format_properties(usage, &profile_list)?;
    for video_props in video_props_list {
        info!("prop: {:#?}", video_props);
        let Some((luma_format, chroma_format)) = split_luma_chroma(video_props.format) else {
            continue;
        };
        if video_props.image_type != vk::ImageType::TYPE_2D {
            continue;
        }
        if !video_props.image_usage_flags.contains(usage) {
            continue;
        }
        if video_props.component_mapping.r != vk::ComponentSwizzle::IDENTITY
            || video_props.component_mapping.g != vk::ComponentSwizzle::IDENTITY
            || video_props.component_mapping.b != vk::ComponentSwizzle::IDENTITY
            || video_props.component_mapping.a != vk::ComponentSwizzle::IDENTITY
        {
            continue;
        }
        let mut profile_list = *profile_list;
        let fmt_info = vk::PhysicalDeviceImageFormatInfo2::default()
            .format(video_props.format)
            .ty(vk::ImageType::TYPE_2D)
            .tiling(video_props.image_tiling)
            .usage(usage)
            .flags(vk::ImageCreateFlags::empty())
            .push_next(&mut profile_list);
        let image_props = device
            .physical_device
            .get_image_format_properties2(&fmt_info)?;
        return Ok(PreEncodeFormat {
            format: video_props.format,
            luma_format,
            chroma_format,
            properties: image_props.image_format_properties,
        });
    }
    Err(Error::UnsupportedFormat)
}

#[derive(Debug)]
struct DpbFormat {
    format: vk::Format,
}

fn find_dpb_format(
    device: &Device,
    profile_list: &vk::VideoProfileListInfoKHR,
) -> Result<DpbFormat> {
    let video_props_list = device
        .physical_device
        .get_video_format_properties(vk::ImageUsageFlags::VIDEO_ENCODE_DPB_KHR, &profile_list)?;
    for video_props in video_props_list {
        if video_props.image_type != vk::ImageType::TYPE_2D {
            continue;
        }
        if video_props
            .image_usage_flags
            .contains(vk::ImageUsageFlags::VIDEO_ENCODE_DPB_KHR)
        {
            continue;
        }
        return Ok(DpbFormat {
            format: video_props.format,
        });
    }
    Err(Error::UnsupportedFormat)
}

#[derive(Debug)]
struct VideoSession {
    device: Arc<Device>,
    handle: vk::VideoSessionKHR,
    memories: Vec<Arc<Memory>>,
}

impl VideoSession {
    // Safety: `info` must be a valid pointer to a `vk::VideoSessionCreateInfoKHR` struct.
    pub unsafe fn new(
        device: Arc<Device>,
        info: &vk::VideoSessionCreateInfoKHR,
    ) -> Result<Arc<Self>> {
        let video_queue_fns = device.extensions.video_queue.as_ref().unwrap();
        let mut handle = vk::VideoSessionKHR::null();
        unsafe {
            (video_queue_fns.create_video_session_khr)(
                device.handle.handle(),
                info,
                null(),
                &mut handle,
            )
            .result()?
        };
        let mem_requirements_list = unsafe {
            read_into_uninitialized_vector(|count, data| {
                (video_queue_fns.get_video_session_memory_requirements_khr)(
                    device.handle.handle(),
                    handle,
                    count,
                    data,
                )
            })?
        };
        let mut memories = Vec::new();
        let mut binds = Vec::new();
        for mem_reqs in mem_requirements_list {
            let mem = match device.clone().allocate_memory(
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                &mem_reqs.memory_requirements,
            ) {
                Ok(mem) => mem,
                Err(Error::NoMatchingMemoryType) => device.clone().allocate_memory(
                    vk::MemoryPropertyFlags::empty(),
                    &mem_reqs.memory_requirements,
                )?,
                Err(err) => return Err(err),
            };
            memories.push(mem.clone());
            let bind_info = vk::BindVideoSessionMemoryInfoKHR::default()
                .memory(mem.handle)
                .memory_bind_index(mem_reqs.memory_bind_index)
                .memory_offset(0)
                .memory_size(mem.size);
            binds.push(bind_info);
        }
        unsafe {
            (video_queue_fns.bind_video_session_memory_khr)(
                device.handle.handle(),
                handle,
                binds.len() as u32,
                binds.as_ptr(),
            )
            .result()?
        };
        Ok(Arc::new(Self {
            device,
            handle,
            memories,
        }))
    }
}

impl Drop for VideoSession {
    fn drop(&mut self) {
        unsafe {
            (self
                .device
                .extensions
                .video_queue
                .as_ref()
                .unwrap()
                .destroy_video_session_khr)(
                self.device.handle.handle(), self.handle, null()
            );
        }
    }
}

struct VideoSessionParameters {}

impl VideoSessionParameters {
    pub fn new(h264_profile: &vk::VideoDecodeH264ProfileInfoKHR) -> Result<Self> {
        use ash::vk::native::*;
        let mut sps_flags = StdVideoH264SpsFlags {
            __bindgen_padding_0: Default::default(),
            _bitfield_1: Default::default(),
            _bitfield_align_1: Default::default(),
        };
        sps_flags.set_frame_mbs_only_flag(1);
        sps_flags.set_direct_8x8_inference_flag(1);
        sps_flags.set_frame_cropping_flag(1);

        // let mut sps = StdVideoH264SequenceParameterSet {
        //     flags: sps_flags,
        //     profile_idc: h264_profile.std_profile_idc,
        //     level_idc: StdVideoH264LevelIdc_STD_VIDEO_H264_LEVEL_IDC_INVALID,

        // };

        todo!()
    }
}
