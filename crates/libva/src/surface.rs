use std::sync::Arc;

use crate::{sys, ConfigAttributes, Display, RtFormat};

pub struct Surface {
    handle: sys::VASurfaceID,
    display: Arc<Display>,
}

impl Surface {
    pub fn new_many(
        display: Arc<Display>,
        format: RtFormat,
        width: u32,
        height: u32,
        count: u32,
        attributes: &ConfigAttributes,
    ) -> sys::VAStatus {
        let mut raw_attributes = attributes.to_raw_attrib_list();
        let mut handles = vec![0; count as usize];
        todo!()
        // unsafe {
        //     display.library().lib().vaCreateSurfaces(
        //         display.handle(),
        //         format.bits(),
        //         width,
        //         height,
        //         handles.as_mut_ptr(),
        //         count,
        //         raw_attributes.as_mut_ptr(),
        //         raw_attributes.len() as _,
        //     )
        // }
        // display.library().lib().vaMaxNumConfigAttributes(dpy)
    }
}
