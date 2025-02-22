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

trait GenericValue: Sized {
    fn from_raw(attr: u32) -> Option<Self>;
    fn to_raw(self) -> u32;
}

impl GenericValue for u32 {
    fn from_raw(attr: u32) -> Option<Self> {
        Some(attr)
    }

    fn to_raw(self) -> u32 {
        self
    }
}

impl GenericValue for bool {
    fn from_raw(attr: u32) -> Option<Self> {
        Some(attr != 0)
    }

    fn to_raw(self) -> u32 {
        self as u32
    }
}

macro_rules! va_enum {
    {$name:ident: $sys_type:ty; $prefix:ident; $($elem_name:ident ,)*} => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $(
                $elem_name,
            )*
        }

        paste! {
            impl TryFrom<$sys_type> for $name {
                type Error = ();
                fn try_from(value: $sys_type) -> Result<Self, Self::Error> {
                    match value {
                        $(
                            sys::[<$prefix $elem_name>] => Ok(Self::$elem_name),
                        )*
                        _ => Err(()),
                    }
                }
            }
            impl From<$name> for $sys_type {
                fn from(value: $name) -> Self {
                    match value {
                        $(
                            $name::$elem_name => sys::[<$prefix $elem_name>],
                        )*
                    }
                }
            }
        }
    }
}

macro_rules! va_bitflags {
    {$name:ident; $prefix:ident; $($elem_name:ident ;)*} => {
        paste! {
            bitflags! {
                    #[repr(transparent)]
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
                    pub struct $name: u32 {
                        $(
                            const $elem_name = sys::[<$prefix $elem_name>] as u32;
                        )*
                        const _ = !0;
                    }
            }
        }

        impl GenericValue for $name {
            fn from_raw(raw: u32) -> Option<Self> {
                Some(Self::from_bits_truncate(raw))
            }

            fn to_raw(self) -> u32 {
                self.bits() as u32
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
                        if let Some(&attr) = self.$attrib_name.as_ref() {
                            result.push(sys::VAConfigAttrib {
                                type_: sys::[<VAConfigAttrib $type_name>],
                                value: GenericValue::to_raw(attr),
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
                                        result.$attrib_name = <$attrib_type as GenericValue>::from_raw(attr.value);
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

va_enum! {
    Profile: sys::VAProfile; VAProfile;
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

va_enum! {
    Entrypoint: sys::VAEntrypoint; VAEntrypoint;
    VLD,
    IZZ,
    IDCT,
    MoComp,
    Deblocking,
    EncSlice,
    EncPicture,
    EncSliceLP,
    VideoProc,
    FEI,
    Stats,
    ProtectedTEEComm,
    ProtectedContent,
}

va_bitflags! {
    RtFormat; VA_RT_FORMAT_;
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

va_bitflags! {
    DecSliceMode; VA_DEC_SLICE_MODE_;
    NORMAL;
    BASE;
}

va_bitflags! {
    EncPackedHeaders; VA_ENC_PACKED_HEADER_;
    SEQUENCE;
    PICTURE;
    SLICE;
    MISC;
    RAW_DATA;
}

va_bitflags! {
    EncInterlaced; VA_ENC_INTERLACED_;
    FRAME;
    FIELD;
    MBAFF;
    PAFF;
}

va_bitflags! {
    EncSliceStructure; VA_ENC_SLICE_STRUCTURE_;
    POWER_OF_TWO_ROWS;
    ARBITRARY_MACROBLOCKS;
    EQUAL_ROWS;
    MAX_SLICE_SIZE;
    ARBITRARY_ROWS;
    EQUAL_MULTI_ROWS;
}

va_bitflags! {
    EncQuantization; VA_ENC_QUANTIZATION_;
    TRELLIS_SUPPORTED;
}

va_bitflags! {
    EncIntraRefresh; VA_ENC_INTRA_REFRESH_;
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
