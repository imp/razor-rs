#![allow(non_camel_case_types)]
#![allow(deref_nullptr)]

use std::borrow::Cow;
use std::ffi::CStr;

use razor_libnvpair::*;

include!(concat!(env!("OUT_DIR"), "/zfs.rs"));

impl zfs_type_t {
    /// Returns true if the type is a filesystem.
    ///
    pub fn is_filesystem(&self) -> bool {
        *self & zfs_type_t::ZFS_TYPE_FILESYSTEM != zfs_type_t(0)
    }

    /// Returns true if the type is a snapshot.
    ///
    pub fn is_snapshot(&self) -> bool {
        *self & zfs_type_t::ZFS_TYPE_SNAPSHOT != zfs_type_t(0)
    }

    /// Returns true if the type is a volume.
    ///
    pub fn is_volume(&self) -> bool {
        *self & zfs_type_t::ZFS_TYPE_VOLUME != zfs_type_t(0)
    }

    /// Returns true if the type is a bookmark.
    ///
    pub fn is_bookmark(&self) -> bool {
        *self & zfs_type_t::ZFS_TYPE_BOOKMARK != zfs_type_t(0)
    }

    /// Returns true if the type is a pool.
    ///
    pub fn is_pool(&self) -> bool {
        *self & zfs_type_t::ZFS_TYPE_POOL != zfs_type_t(0)
    }

    pub fn contains(&self, other: zfs_type_t) -> bool {
        *self & other != zfs_type_t(0)
    }

    pub fn name(&self) -> Cow<'static, str> {
        (*self).into()
    }
}

// It is safe to call `zfs_type_to_name` anytime, even before before libzfs initialization.
// So, this should be safe to use anywhere
impl From<zfs_type_t> for Cow<'static, str> {
    fn from(r#type: zfs_type_t) -> Self {
        unsafe {
            let cstr = zfs_type_to_name(r#type);
            CStr::from_ptr(cstr).to_string_lossy()
        }
    }
}
