use crate::sys;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ErrorStatus {
    OperationFailed,
    AllocationFailed,
    InvalidDisplay,
    InvalidConfig,
    InvalidContext,
    InvalidSurface,
    InvalidBuffer,
    InvalidImage,
    InvalidSubpicture,
    AttrNotSupported,
    MaxNumExceeded,
    UnsupportedProfile,
    UnsupportedEntrypoint,
    UnsupportedRtFormat,
    UnsupportedBufferType,
    SurfaceBusy,
    FlagNotSupported,
    InvalidParameter,
    ResolutionNotSupported,
    Unimplemented,
    SurfaceInDisplaying,
    InvalidImageFormat,
    DecodingError,
    EncodingError,
    InvalidValue,
    UnsupportedFilter,
    InvalidFilterChain,
    HwBusy,
    UnsupportedMemoryType,
    NotEnoughBuffer,
    TimedOut,
    Unknown,
}

impl ErrorStatus {
    pub fn from_status(status: sys::VAStatus) -> Option<Self> {
        if status as u32 == sys::VA_STATUS_SUCCESS {
            return None;
        }
        Some(match status as u32 {
            sys::VA_STATUS_ERROR_OPERATION_FAILED => ErrorStatus::OperationFailed,
            sys::VA_STATUS_ERROR_ALLOCATION_FAILED => ErrorStatus::AllocationFailed,
            sys::VA_STATUS_ERROR_INVALID_DISPLAY => ErrorStatus::InvalidDisplay,
            sys::VA_STATUS_ERROR_INVALID_CONFIG => ErrorStatus::InvalidConfig,
            sys::VA_STATUS_ERROR_INVALID_CONTEXT => ErrorStatus::InvalidContext,
            sys::VA_STATUS_ERROR_INVALID_SURFACE => ErrorStatus::InvalidSurface,
            sys::VA_STATUS_ERROR_INVALID_BUFFER => ErrorStatus::InvalidBuffer,
            sys::VA_STATUS_ERROR_INVALID_IMAGE => ErrorStatus::InvalidImage,
            sys::VA_STATUS_ERROR_INVALID_SUBPICTURE => ErrorStatus::InvalidSubpicture,
            sys::VA_STATUS_ERROR_ATTR_NOT_SUPPORTED => ErrorStatus::AttrNotSupported,
            sys::VA_STATUS_ERROR_MAX_NUM_EXCEEDED => ErrorStatus::MaxNumExceeded,
            sys::VA_STATUS_ERROR_UNSUPPORTED_PROFILE => ErrorStatus::UnsupportedProfile,
            sys::VA_STATUS_ERROR_UNSUPPORTED_ENTRYPOINT => ErrorStatus::UnsupportedEntrypoint,
            sys::VA_STATUS_ERROR_UNSUPPORTED_RT_FORMAT => ErrorStatus::UnsupportedRtFormat,
            sys::VA_STATUS_ERROR_UNSUPPORTED_BUFFERTYPE => ErrorStatus::UnsupportedBufferType,
            sys::VA_STATUS_ERROR_SURFACE_BUSY => ErrorStatus::SurfaceBusy,
            sys::VA_STATUS_ERROR_FLAG_NOT_SUPPORTED => ErrorStatus::FlagNotSupported,
            sys::VA_STATUS_ERROR_INVALID_PARAMETER => ErrorStatus::InvalidParameter,
            sys::VA_STATUS_ERROR_RESOLUTION_NOT_SUPPORTED => ErrorStatus::ResolutionNotSupported,
            sys::VA_STATUS_ERROR_UNIMPLEMENTED => ErrorStatus::Unimplemented,
            sys::VA_STATUS_ERROR_SURFACE_IN_DISPLAYING => ErrorStatus::SurfaceInDisplaying,
            sys::VA_STATUS_ERROR_INVALID_IMAGE_FORMAT => ErrorStatus::InvalidImageFormat,
            sys::VA_STATUS_ERROR_DECODING_ERROR => ErrorStatus::DecodingError,
            sys::VA_STATUS_ERROR_ENCODING_ERROR => ErrorStatus::EncodingError,
            sys::VA_STATUS_ERROR_INVALID_VALUE => ErrorStatus::InvalidValue,
            sys::VA_STATUS_ERROR_UNSUPPORTED_FILTER => ErrorStatus::UnsupportedFilter,
            sys::VA_STATUS_ERROR_INVALID_FILTER_CHAIN => ErrorStatus::InvalidFilterChain,
            sys::VA_STATUS_ERROR_HW_BUSY => ErrorStatus::HwBusy,
            sys::VA_STATUS_ERROR_UNSUPPORTED_MEMORY_TYPE => ErrorStatus::UnsupportedMemoryType,
            sys::VA_STATUS_ERROR_NOT_ENOUGH_BUFFER => ErrorStatus::NotEnoughBuffer,
            sys::VA_STATUS_ERROR_TIMEDOUT => ErrorStatus::TimedOut,
            _ => ErrorStatus::Unknown,
        })
    }

    pub fn result(status: sys::VAStatus) -> Result<(), ErrorStatus> {
        match Self::from_status(status) {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}

pub trait VaStatusExt {
    fn va_result(self) -> Result<(), ErrorStatus>;
    fn va_result_with_success<T>(self, success: T) -> Result<T, ErrorStatus>;
}

impl VaStatusExt for sys::VAStatus {
    fn va_result(self) -> Result<(), ErrorStatus> {
        ErrorStatus::result(self)
    }
    fn va_result_with_success<T>(self, success: T) -> Result<T, ErrorStatus> {
        ErrorStatus::result(self).map(|_| success)
    }
}

impl std::fmt::Display for ErrorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for ErrorStatus {}

pub type VaResult<T> = Result<T, ErrorStatus>;
