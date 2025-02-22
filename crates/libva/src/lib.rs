use bitflags::bitflags;
use paste::paste;

pub use vendec_libva_sys as sys;

mod config;
mod display;
mod error;
mod library;
mod surface;
pub use config::*;
pub use display::*;
pub use error::*;
pub use library::*;
pub use surface::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum Profile {
    MPEG2Simple,
    MPEG2Main,
    MPEG4Simple,
    MPEG4AdvancedSimple,
    MPEG4Main,
    H264Baseline,
    H264Main,
    H264High,
    VC1Simple,
    VC1Main,
    VC1Advanced,
    H263Baseline,
    JPEGBaseline,
    H264ConstrainedBaseline,
    VP8Version0_3,
    H264MultiviewHigh,
    H264StereoHigh,
    HEVCMain,
    HEVCMain10,
    VP9Profile0,
    VP9Profile1,
    VP9Profile2,
    VP9Profile3,
    HEVCMain12,
    HEVCMain422_10,
    HEVCMain422_12,
    HEVCMain444,
    HEVCMain444_10,
    HEVCMain444_12,
    HEVCSccMain,
    HEVCSccMain10,
    HEVCSccMain444,
    AV1Profile0,
    AV1Profile1,
    HEVCSccMain444_10,
    Protected,
    H264High10,
    VVCMain10,
    VVCMultilayerMain10,
}

impl Profile {
    pub fn from_raw(raw: sys::VAProfile) -> Option<Self> {
        match raw {
            sys::VAProfileMPEG2Simple => Some(Self::MPEG2Simple),
            sys::VAProfileMPEG2Main => Some(Self::MPEG2Main),
            sys::VAProfileMPEG4Simple => Some(Self::MPEG4Simple),
            sys::VAProfileMPEG4AdvancedSimple => Some(Self::MPEG4AdvancedSimple),
            sys::VAProfileMPEG4Main => Some(Self::MPEG4Main),
            sys::VAProfileH264Baseline => Some(Self::H264Baseline),
            sys::VAProfileH264Main => Some(Self::H264Main),
            sys::VAProfileH264High => Some(Self::H264High),
            sys::VAProfileVC1Simple => Some(Self::VC1Simple),
            sys::VAProfileVC1Main => Some(Self::VC1Main),
            sys::VAProfileVC1Advanced => Some(Self::VC1Advanced),
            sys::VAProfileH263Baseline => Some(Self::H263Baseline),
            sys::VAProfileJPEGBaseline => Some(Self::JPEGBaseline),
            sys::VAProfileH264ConstrainedBaseline => Some(Self::H264ConstrainedBaseline),
            sys::VAProfileVP8Version0_3 => Some(Self::VP8Version0_3),
            sys::VAProfileH264MultiviewHigh => Some(Self::H264MultiviewHigh),
            sys::VAProfileH264StereoHigh => Some(Self::H264StereoHigh),
            sys::VAProfileHEVCMain => Some(Self::HEVCMain),
            sys::VAProfileHEVCMain10 => Some(Self::HEVCMain10),
            sys::VAProfileVP9Profile0 => Some(Self::VP9Profile0),
            sys::VAProfileVP9Profile1 => Some(Self::VP9Profile1),
            sys::VAProfileVP9Profile2 => Some(Self::VP9Profile2),
            sys::VAProfileVP9Profile3 => Some(Self::VP9Profile3),
            sys::VAProfileHEVCMain12 => Some(Self::HEVCMain12),
            sys::VAProfileHEVCMain422_10 => Some(Self::HEVCMain422_10),
            sys::VAProfileHEVCMain422_12 => Some(Self::HEVCMain422_12),
            sys::VAProfileHEVCMain444 => Some(Self::HEVCMain444),
            sys::VAProfileHEVCMain444_10 => Some(Self::HEVCMain444_10),
            sys::VAProfileHEVCMain444_12 => Some(Self::HEVCMain444_12),
            sys::VAProfileHEVCSccMain => Some(Self::HEVCSccMain),
            sys::VAProfileHEVCSccMain10 => Some(Self::HEVCSccMain10),
            sys::VAProfileHEVCSccMain444 => Some(Self::HEVCSccMain444),
            sys::VAProfileAV1Profile0 => Some(Self::AV1Profile0),
            sys::VAProfileAV1Profile1 => Some(Self::AV1Profile1),
            sys::VAProfileHEVCSccMain444_10 => Some(Self::HEVCSccMain444_10),
            sys::VAProfileProtected => Some(Self::Protected),
            sys::VAProfileH264High10 => Some(Self::H264High10),
            sys::VAProfileVVCMain10 => Some(Self::VVCMain10),
            sys::VAProfileVVCMultilayerMain10 => Some(Self::VVCMultilayerMain10),
            _ => None,
        }
    }

    pub fn to_raw(self) -> sys::VAProfile {
        match self {
            Self::MPEG2Simple => sys::VAProfileMPEG2Simple,
            Self::MPEG2Main => sys::VAProfileMPEG2Main,
            Self::MPEG4Simple => sys::VAProfileMPEG4Simple,
            Self::MPEG4AdvancedSimple => sys::VAProfileMPEG4AdvancedSimple,
            Self::MPEG4Main => sys::VAProfileMPEG4Main,
            Self::H264Baseline => sys::VAProfileH264Baseline,
            Self::H264Main => sys::VAProfileH264Main,
            Self::H264High => sys::VAProfileH264High,
            Self::VC1Simple => sys::VAProfileVC1Simple,
            Self::VC1Main => sys::VAProfileVC1Main,
            Self::VC1Advanced => sys::VAProfileVC1Advanced,
            Self::H263Baseline => sys::VAProfileH263Baseline,
            Self::JPEGBaseline => sys::VAProfileJPEGBaseline,
            Self::H264ConstrainedBaseline => sys::VAProfileH264ConstrainedBaseline,
            Self::VP8Version0_3 => sys::VAProfileVP8Version0_3,
            Self::H264MultiviewHigh => sys::VAProfileH264MultiviewHigh,
            Self::H264StereoHigh => sys::VAProfileH264StereoHigh,
            Self::HEVCMain => sys::VAProfileHEVCMain,
            Self::HEVCMain10 => sys::VAProfileHEVCMain10,
            Self::VP9Profile0 => sys::VAProfileVP9Profile0,
            Self::VP9Profile1 => sys::VAProfileVP9Profile1,
            Self::VP9Profile2 => sys::VAProfileVP9Profile2,
            Self::VP9Profile3 => sys::VAProfileVP9Profile3,
            Self::HEVCMain12 => sys::VAProfileHEVCMain12,
            Self::HEVCMain422_10 => sys::VAProfileHEVCMain422_10,
            Self::HEVCMain422_12 => sys::VAProfileHEVCMain422_12,
            Self::HEVCMain444 => sys::VAProfileHEVCMain444,
            Self::HEVCMain444_10 => sys::VAProfileHEVCMain444_10,
            Self::HEVCMain444_12 => sys::VAProfileHEVCMain444_12,
            Self::HEVCSccMain => sys::VAProfileHEVCSccMain,
            Self::HEVCSccMain10 => sys::VAProfileHEVCSccMain10,
            Self::HEVCSccMain444 => sys::VAProfileHEVCSccMain444,
            Self::AV1Profile0 => sys::VAProfileAV1Profile0,
            Self::AV1Profile1 => sys::VAProfileAV1Profile1,
            Self::HEVCSccMain444_10 => sys::VAProfileHEVCSccMain444_10,
            Self::Protected => sys::VAProfileProtected,
            Self::H264High10 => sys::VAProfileH264High10,
            Self::VVCMain10 => sys::VAProfileVVCMain10,
            Self::VVCMultilayerMain10 => sys::VAProfileVVCMultilayerMain10,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum Entrypoint {
    Vld,
    Izz,
    Idct,
    MoComp,
    Deblocking,
    EncSlice,
    EncPicture,
    EncSliceLP,
    VideoProc,
    Fei,
    Stats,
    ProtectedTeeComm,
    ProtectedContent,
}

impl Entrypoint {
    pub fn from_raw(raw: sys::VAEntrypoint) -> Option<Self> {
        match raw {
            sys::VAEntrypointVLD => Some(Self::Vld),
            sys::VAEntrypointIZZ => Some(Self::Izz),
            sys::VAEntrypointIDCT => Some(Self::Idct),
            sys::VAEntrypointMoComp => Some(Self::MoComp),
            sys::VAEntrypointDeblocking => Some(Self::Deblocking),
            sys::VAEntrypointEncSlice => Some(Self::EncSlice),
            sys::VAEntrypointEncPicture => Some(Self::EncPicture),
            sys::VAEntrypointEncSliceLP => Some(Self::EncSliceLP),
            sys::VAEntrypointVideoProc => Some(Self::VideoProc),
            sys::VAEntrypointFEI => Some(Self::Fei),
            sys::VAEntrypointStats => Some(Self::Stats),
            sys::VAEntrypointProtectedTEEComm => Some(Self::ProtectedTeeComm),
            sys::VAEntrypointProtectedContent => Some(Self::ProtectedContent),
            _ => None,
        }
    }

    pub fn to_raw(self) -> sys::VAEntrypoint {
        match self {
            Self::Vld => sys::VAEntrypointVLD,
            Self::Izz => sys::VAEntrypointIZZ,
            Self::Idct => sys::VAEntrypointIDCT,
            Self::MoComp => sys::VAEntrypointMoComp,
            Self::Deblocking => sys::VAEntrypointDeblocking,
            Self::EncSlice => sys::VAEntrypointEncSlice,
            Self::EncPicture => sys::VAEntrypointEncPicture,
            Self::EncSliceLP => sys::VAEntrypointEncSliceLP,
            Self::VideoProc => sys::VAEntrypointVideoProc,
            Self::Fei => sys::VAEntrypointFEI,
            Self::Stats => sys::VAEntrypointStats,
            Self::ProtectedTeeComm => sys::VAEntrypointProtectedTEEComm,
            Self::ProtectedContent => sys::VAEntrypointProtectedContent,
        }
    }
}

trait FromConfigAttrib: Sized {
    fn from_config_attrib(attr: u32) -> Option<Self>;
}

trait ToConfigAttrib {
    fn into_config_attrib(self) -> u32;
}

impl FromConfigAttrib for bool {
    fn from_config_attrib(attr: u32) -> Option<Self> {
        match attr {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }
}

impl ToConfigAttrib for bool {
    fn into_config_attrib(self) -> u32 {
        self as u32
    }
}

impl FromConfigAttrib for u32 {
    fn from_config_attrib(attr: u32) -> Option<Self> {
        Some(attr)
    }
}

impl ToConfigAttrib for u32 {
    fn into_config_attrib(self) -> u32 {
        self
    }
}

macro_rules! va_bitflags {
    {$name:ident: $sys_name:ty; $prefix:ident; $($elem_name:ident ;)*} => {
        paste! {
            bitflags! {
                    #[repr(transparent)]
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                    pub struct $name: $sys_name {
                        $(
                            const $elem_name = sys::[<$prefix $elem_name>];
                        )*
                        const _ = !0;
                    }
            }
        }

        impl FromConfigAttrib for $name {
            fn from_config_attrib(attrib_val: u32) -> Option<Self> {
                Self::from_bits(attrib_val)
            }
        }

        impl ToConfigAttrib for $name {
            fn into_config_attrib(self) -> u32 {
                self.bits()
            }
        }
    }
}

macro_rules! va_enum {
    {$name:ident: $sys_name:ty; $prefix:ident; $($elem_name:ident ;)*} => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $(
                $elem_name,
            )*
        }

        paste! {
            impl FromConfigAttrib for $name {
                fn from_config_attrib(attrib_val: u32) -> Option<Self> {
                    match attrib_val {
                        $(
                            sys::[<$prefix $elem_name:snake:upper>] => Some(Self::$elem_name),
                        )*
                        _ => None,
                    }
                }
            }

            impl ToConfigAttrib for $name {
                fn into_config_attrib(self) -> u32 {
                    match self {
                        $(
                            Self::$elem_name => sys::[<$prefix $elem_name:snake:upper>],
                        )*
                    }
                }
            }
        }
    }
}

macro_rules! va_config_attrib {
    {$name:ident ; $($type_name:ident : $attrib_name:ident : $attrib_type:ty ,)*} => {
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
        pub struct $name {
            $(
                $attrib_name: Option<$attrib_type>,
            )*
        }

        paste! {
            impl $name {
                pub fn default_raw_attrib_list() -> Vec<sys::VAConfigAttrib> {
                    vec![
                        $(
                            sys::VAConfigAttrib {
                                type_: sys::[<VAConfigAttrib $type_name>],
                                value: 0,
                            },
                        )*
                    ]
                }

                pub fn to_raw_attrib_list(&self) -> Vec<sys::VAConfigAttrib> {
                    let mut result = Vec::new();
                    $(
                        if let Some(attr) = self.$attrib_name.as_ref() {
                            result.push(sys::VAConfigAttrib {
                                type_: sys::[<VAConfigAttrib $type_name>],
                                value: attr.into_config_attrib(),
                            });
                        }
                    )*
                    result
                }

                pub fn from_raw_attrib_list(attrs: &[sys::VAConfigAttrib]) -> Self {
                    let mut result = Self::default();
                    for attr in attrs {
                        match attr.type_ {
                            $(
                                sys::[<VAConfigAttrib $type_name>] => {
                                    if attr.value != sys::VA_ATTRIB_NOT_SUPPORTED {
                                        result.$attrib_name = <$attrib_type as FromConfigAttrib>::from_config_attrib(attr.value);
                                    }
                                }
                            )*
                            _ => {}
                        }
                    }
                    result
                }


            }
        }
    };
}

va_bitflags! {
    RtFormat: u32;
    VA_RT_FORMAT_;
    YUV420;
    YUV422;
    YUV444;
    YUV411;
    YUV400;

    YUV420_10;
    YUV422_10;
    YUV444_10;

    YUV420_12;
    YUV422_12;
    YUV444_12;

    RGB16;
    RGB32;
    RGBP;
    RGB32_10;
    PROTECTED;
    RGB32_10BPP;
    YUV420_10BPP;
}

va_enum! {
    DecSliceMode: u32;
    VA_DEC_SLICE_MODE_;
    Normal;
    Base;
}

va_bitflags! {
    EncPackedHeaders: sys::VAEncPackedHeaderType;
    VA_ENC_PACKED_HEADER_;
    SEQUENCE;
    PICTURE;
    SLICE;
    MISC;
    RAW_DATA;
}

va_bitflags! {
    EncInterlaced: u32;
    VA_ENC_INTERLACED_;
    FRAME;
    FIELD;
    MBAFF;
    PAFF;
}

va_bitflags! {
    EncSliceStructure: u32;
    VA_ENC_SLICE_STRUCTURE_;
    POWER_OF_TWO_ROWS;
    ARBITRARY_MACROBLOCKS;
    EQUAL_ROWS;
    MAX_SLICE_SIZE;
    ARBITRARY_ROWS;
    EQUAL_MULTI_ROWS;
}

va_bitflags! {
    EncQuantization: u32;
    VA_ENC_QUANTIZATION_;
    TRELLIS_SUPPORTED;
}

va_bitflags! {
    EncIntraRefresh: u32;
    VA_ENC_INTRA_REFRESH_;
    ROLLING_COLUMN;
    ROLLING_ROW;
    ADAPTIVE;
    CYCLIC;
    P_FRAME;
    B_FRAME;
    MULTI_REF;
}

// https://intel.github.io/libva/group__api__core.html#ga2c3be94ce142fb92a4bf93e9b1b4fa01
va_config_attrib! {
    ConfigAttributes;
    RTFormat: rt_format: RtFormat,
    DecSliceMode: dec_slice_mode: DecSliceMode,
    DecProcessing: dec_processing: bool,
    EncPackedHeaders: enc_packed_headers: EncPackedHeaders,
    EncInterlaced: enc_interlaced: EncInterlaced,
    EncMaxRefFrames: enc_max_ref_frames: u32,
    EncMaxSlices: enc_max_slices: u32,
    EncSliceStructure: enc_slice_structure: EncSliceStructure,
    EncMacroblockInfo: enc_macroblock_info: u32,
    MaxPictureWidth: max_picture_width: u32,
    MaxPictureHeight: max_picture_height: u32,
    EncQualityRange: enc_quality_range: u32,
    EncQuantization: enc_quantization: EncQuantization,
    EncIntraRefresh: enc_intra_refresh: EncIntraRefresh,
    EncSkipFrame: enc_skip_frame: bool,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test() {
        let lib = library::Library::load().unwrap();
        let display = Display::enumerate(lib).next().unwrap();
        let profiles = display.query_config_profiles().unwrap();
        let config_attrs = display
            .get_config_attributes(None, Entrypoint::VideoProc)
            .unwrap();
        println!("{:?}", profiles);
        println!("{:?}", config_attrs);
        let config =
            Config::new(display.clone(), None, Entrypoint::VideoProc, &config_attrs).unwrap();
    }
}
