use std::ffi::CString;

use nvpair::NvListAccess;

use crate::error::CoreError;

use super::error::value_or_err;
use super::libzfs_handler::LIB_ZFS_HANDLER;
use super::nvpair;
use super::sys;
use super::Result;

fn init() {
    let guard = LIB_ZFS_HANDLER.lock();
    drop(guard);
}

pub fn create_filesystem(name: impl AsRef<str>, nvl: &nvpair::NvList) -> Result<nvpair::NvList> {
    create_dataset(name, nvl, sys::lzc_dataset_type::LZC_DATSET_TYPE_ZFS)
}

pub fn create_volume(name: impl AsRef<str>, nvl: &nvpair::NvList) -> Result<nvpair::NvList> {
    create_dataset(name, nvl, sys::lzc_dataset_type::LZC_DATSET_TYPE_ZVOL)
}

fn create_dataset(
    name: impl AsRef<str>,
    nvl: &nvpair::NvList,
    prop: sys::lzc_dataset_type,
) -> Result<nvpair::NvList> {
    init();
    let cname = CString::new(name.as_ref())?;

    let rc = unsafe { sys::lzc_create(cname.as_ptr(), prop, nvl.nvl(), std::ptr::null_mut(), 0) };

    value_or_err((), rc)?;

    let zfs_handle =
        unsafe { sys::make_dataset_handle(LIB_ZFS_HANDLER.lock().handler(), cname.as_ptr()) };

    let nvl = nvpair::NvListRef::from_raw(zfs_handle.zfs_props, &zfs_handle);
    unsafe {
        nvpair::NvList {
            raw: (*zfs_handle).zfs_props,
        }
    };

    value_or_err(nvl, rc)
}

pub fn get_dataset_nvlist(name: impl AsRef<str>) -> Result<nvpair::NvList> {
    init();
    let cname = CString::new(name.as_ref())?;
    let zfs_handle =
        unsafe { sys::make_dataset_handle(LIB_ZFS_HANDLER.lock().handler(), cname.as_ptr()) };

    if zfs_handle.is_null() {
        return Err(CoreError::DatasetNotExist);
    }

    let nvl = unsafe {
        nvpair::NvList {
            raw: (*zfs_handle).zfs_props,
        }
    };

    Ok(nvl)
}

pub fn destroy_dataset(name: impl AsRef<str>) -> Result<()> {
    init();
    let cname = CString::new(name.as_ref())?;
    let rc = unsafe { sys::lzc_destroy(cname.as_ptr()) };

    value_or_err((), rc)
}

// TODO: 1.check mounted
//       2. implement the same for relative, devices, exec, readonly, setuid, nbmand
pub fn default_atime(name: CString) -> u64 {
    let res = unsafe { sys::zfs_prop_default_numeric(sys::zfs_prop_t::ZFS_PROP_ATIME) };
    let mut mnttab: sys::mnttab = unsafe { std::mem::zeroed() };
    let mnttab_ptr: *mut sys::mnttab = &mut mnttab;
    let mut mntent: sys::mntent = unsafe { std::mem::zeroed() };
    let mntent_ptr: *mut sys::mntent = &mut mntent;
    dbg!("I GOT A TIME", res);

    let zfs_handle =
        unsafe { sys::make_dataset_handle(LIB_ZFS_HANDLER.lock().handler(), name.as_ptr()) };

    let rc = unsafe {
        sys::libzfs_mnttab_find(
            LIB_ZFS_HANDLER.lock().handler(),
            (*zfs_handle).zfs_name.as_ptr(),
            mnttab_ptr,
        )
    };

    if rc == 0 {
        unsafe {
            (*zfs_handle).zfs_mntopts =
                sys::zfs_strdup(LIB_ZFS_HANDLER.lock().handler(), (*mnttab_ptr).mnt_mntopts)
        }

        // TODO: boolean_t already exist in libnvpair
        unsafe { (*zfs_handle).zfs_mntcheck = sys::boolean_t::B_TRUE }
    }

    if unsafe { (*zfs_handle).zfs_mntopts.is_null() } {
        dbg!("zfs mntops is null");
        unsafe { (*mntent_ptr).mnt_opts = std::ptr::null_mut() };
    } else {
        dbg!("zfs mntops is not null");
        unsafe { (*mntent_ptr).mnt_opts = (*zfs_handle).zfs_mntopts }
    }

    if unsafe { !(*mntent_ptr).mnt_opts.is_null() } {
        if unsafe {
            !sys::hasmntopt(
                mntent_ptr,
                CString::from_vec_unchecked(b"atime".to_vec()).as_ptr(),
            )
            .is_null()
        } && res == 0
        {
            return 1;
        } else if unsafe {
            !sys::hasmntopt(
                mntent_ptr,
                CString::from_vec_unchecked(b"noatime".to_vec()).as_ptr(),
            )
            .is_null()
        } && res != 0
        {
            return 0;
        }
    }

    res
}

pub fn default_type() -> u64 {
    unsafe { sys::zfs_prop_default_numeric(sys::zfs_prop_t::ZFS_PROP_TYPE) }
}

pub fn default_available() -> u64 {
    unsafe { sys::zfs_prop_default_numeric(sys::zfs_prop_t::ZFS_PROP_AVAILABLE) }
}

pub fn default_logicalused() -> u64 {
    unsafe { sys::zfs_prop_default_numeric(sys::zfs_prop_t::ZFS_PROP_LOGICALUSED) }
}

pub fn default_canmount() -> u64 {
    unsafe { sys::zfs_prop_default_numeric(sys::zfs_prop_t::ZFS_PROP_CANMOUNT) }
}

pub fn default_mounted(name: CString) -> bool {
    let zfs_handle =
        unsafe { sys::make_dataset_handle(LIB_ZFS_HANDLER.lock().handler(), name.as_ptr()) };

    unsafe { (*zfs_handle).zfs_mntopts.is_null() }
}

pub fn default_checksum() -> u64 {
    unsafe { sys::zfs_prop_default_numeric(sys::zfs_prop_t::ZFS_PROP_CHECKSUM) }
}

pub fn default_volmode() -> u64 {
    unsafe { sys::zfs_prop_default_numeric(sys::zfs_prop_t::ZFS_PROP_VOLMODE) }
}

pub fn default_compression() -> u64 {
    unsafe { sys::zfs_prop_default_numeric(sys::zfs_prop_t::ZFS_PROP_COMPRESSION) }
}
