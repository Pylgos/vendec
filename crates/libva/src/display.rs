use std::fs;
use std::os::fd::IntoRawFd;
use std::os::fd::OwnedFd;
use std::os::unix::fs::FileTypeExt;
use std::sync::Arc;

use crate::sys;
use crate::ConfigAttributes;
use crate::Entrypoint;
use crate::ErrorStatus;
use crate::Library;
use crate::Profile;
use crate::VaResult;
use crate::VaStatusExt;

pub struct Display {
    handle: sys::VADisplay,
    library: Arc<Library>,
}

impl Display {
    pub fn from_drm(library: Arc<Library>, drm_fd: OwnedFd) -> VaResult<Arc<Self>> {
        let handle = unsafe { library.lib().vaGetDisplayDRM(drm_fd.into_raw_fd()) };
        let mut major_version = 0;
        let mut minor_version = 0;
        unsafe {
            library
                .lib()
                .vaInitialize(handle, &mut major_version, &mut minor_version)
                .va_result()?;
        }
        Ok(Arc::new(Self { handle, library }))
    }

    pub fn enumerate(library: Arc<Library>) -> impl Iterator<Item = Arc<Self>> {
        let mut render_nodes = fs::read_dir("/dev/dri")
            .ok()
            .into_iter()
            .flat_map(move |dir| {
                dir.filter_map(move |entry| {
                    let entry = entry.ok()?;
                    if !entry.file_type().ok()?.is_char_device() {
                        return None;
                    }
                    let path = entry.path();
                    let file_name = path.file_name()?.to_str()?;
                    if !file_name.starts_with("renderD") {
                        return None;
                    }
                    let node_id: u32 = file_name["renderD".len()..].parse().ok()?;
                    Some(node_id)
                })
            })
            .collect::<Vec<_>>();
        render_nodes.sort();
        render_nodes.into_iter().filter_map(move |node_id| {
            let path = format!("/dev/dri/renderD{}", node_id);
            let drm_file = fs::File::options().read(true).write(true).open(path).ok()?;
            Self::from_drm(library.clone(), drm_file.into()).ok()
        })
    }

    pub fn handle(&self) -> sys::VADisplay {
        self.handle
    }

    pub fn library(&self) -> &Arc<Library> {
        &self.library
    }

    pub fn query_config_profiles(&self) -> VaResult<Vec<Profile>> {
        let mut profiles_count = unsafe { self.library.lib().vaMaxNumProfiles(self.handle) };
        let mut raw_profiles = vec![sys::VAProfileNone; profiles_count as usize];
        unsafe {
            self.library
                .lib()
                .vaQueryConfigProfiles(self.handle, raw_profiles.as_mut_ptr(), &mut profiles_count)
                .va_result()?;
        }
        Ok(raw_profiles
            .iter()
            .take(profiles_count as usize)
            .filter_map(|&raw| Profile::from_raw(raw))
            .collect())
    }

    pub fn get_config_attributes(
        &self,
        profile: Option<Profile>,
        entrypoint: Entrypoint,
    ) -> VaResult<ConfigAttributes> {
        let raw_profile = profile.map(|p| p.to_raw()).unwrap_or(sys::VAProfileNone);
        let raw_entrypoint = entrypoint.to_raw();
        let mut raw_attrib_list = ConfigAttributes::default_raw_attrib_list();
        unsafe {
            self.library
                .lib()
                .vaGetConfigAttributes(
                    self.handle(),
                    raw_profile,
                    raw_entrypoint,
                    raw_attrib_list.as_mut_ptr(),
                    raw_attrib_list.len() as _,
                )
                .va_result()?;
        }
        Ok(ConfigAttributes::from_raw_attrib_list(&raw_attrib_list))
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe { self.library.lib().vaTerminate(self.handle) };
    }
}

// Safety: VA-API is thread-safe.
unsafe impl Send for Display {}
unsafe impl Sync for Display {}
