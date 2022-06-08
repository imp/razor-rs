#![cfg_attr(feature = "pedantic", warn(clippy::pedantic))]
#![warn(clippy::use_self)]
#![warn(clippy::map_flatten)]
#![warn(clippy::map_unwrap_or)]
#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(noop_method_call)]
#![warn(unreachable_pub)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
#![allow(clippy::missing_safety_doc)]
#![deny(warnings)]

use std::ffi;
use std::mem;
use std::ptr;

use once_cell::sync::Lazy;
use razor_libnvpair as libnvpair;
use razor_libzfscore_sys as sys;

pub use sys::zfs_handle_t;
pub use sys::zfs_prop_t;
pub use sys::zfs_type_t;
pub use version::Version;

use handle::LIBZFS_HANDLE;

mod handle;
mod version;

pub unsafe fn libzfs_errno() -> libc::c_int {
    sys::libzfs_errno(LIBZFS_HANDLE.handle())
}

pub unsafe fn libzfs_error_action() -> *const libc::c_char {
    sys::libzfs_error_action(LIBZFS_HANDLE.handle())
}

pub unsafe fn libzfs_error_description() -> *const libc::c_char {
    sys::libzfs_error_description(LIBZFS_HANDLE.handle())
}

pub unsafe fn zfs_open(name: *const libc::c_char) -> *mut sys::zfs_handle_t {
    let types = sys::zfs_type_t::ZFS_TYPE_FILESYSTEM
        | sys::zfs_type_t::ZFS_TYPE_VOLUME
        | sys::zfs_type_t::ZFS_TYPE_SNAPSHOT
        | sys::zfs_type_t::ZFS_TYPE_POOL
        | sys::zfs_type_t::ZFS_TYPE_BOOKMARK;
    let types = types.0 as i32;
    sys::zfs_open(LIBZFS_HANDLE.handle(), name, types)
}

pub unsafe fn zfs_close(handle: *mut sys::zfs_handle_t) {
    Lazy::force(&LIBZFS_HANDLE);
    sys::zfs_close(handle);
}

pub unsafe fn zfs_get_name(handle: *mut sys::zfs_handle_t) -> *const libc::c_char {
    Lazy::force(&LIBZFS_HANDLE);
    sys::zfs_get_name(handle)
}

pub unsafe fn zfs_get_type(handle: *mut sys::zfs_handle_t) -> sys::zfs_type_t {
    Lazy::force(&LIBZFS_HANDLE);
    sys::zfs_get_type(handle)
}

pub unsafe fn zfs_get_all_props(handle: *mut sys::zfs_handle_t) -> *mut libnvpair::nvlist_t {
    Lazy::force(&LIBZFS_HANDLE);
    sys::zfs_get_all_props(handle)
}

pub unsafe fn zfs_prop_get_numeric(
    handle: *mut sys::zfs_handle_t,
    property: sys::zfs_prop_t,
) -> Result<u64, i32> {
    Lazy::force(&LIBZFS_HANDLE);
    let mut value = 0;
    let mut src = mem::MaybeUninit::uninit();
    let statbuf = ptr::null_mut();
    let statlen = 0;
    let rc = sys::zfs_prop_get_numeric(
        handle,
        property,
        &mut value,
        src.as_mut_ptr(),
        statbuf,
        statlen,
    );
    if rc == 0 {
        Ok(value)
    } else {
        Err(rc)
    }
}

pub unsafe fn zfs_prop_get_int(handle: *mut sys::zfs_handle_t, property: sys::zfs_prop_t) -> u64 {
    Lazy::force(&LIBZFS_HANDLE);
    sys::zfs_prop_get_int(handle, property)
}

pub unsafe fn zfs_prop_set_list(
    dataset_handle: *mut sys::zfs_handle_t,
    nvl: *mut libnvpair::nvlist_t,
) -> libc::c_int {
    Lazy::force(&LIBZFS_HANDLE);
    sys::zfs_prop_set_list(dataset_handle, nvl)
}

pub unsafe fn zfs_prop_to_name(property: sys::zfs_prop_t) -> *const libc::c_char {
    Lazy::force(&LIBZFS_HANDLE);
    sys::zfs_prop_to_name(property)
}

pub unsafe fn zfs_prop_default_string(property: sys::zfs_prop_t) -> *const libc::c_char {
    Lazy::force(&LIBZFS_HANDLE);
    sys::zfs_prop_default_string(property)
}

pub unsafe fn zfs_prop_default_numeric(property: sys::zfs_prop_t) -> u64 {
    Lazy::force(&LIBZFS_HANDLE);
    sys::zfs_prop_default_numeric(property)
}

pub unsafe fn zfs_iter_root(
    f: unsafe extern "C" fn(*mut sys::zfs_handle_t, *mut libc::c_void) -> libc::c_int,
    ptr: *mut libc::c_void,
) {
    sys::zfs_iter_root(LIBZFS_HANDLE.handle(), Some(f), ptr);
}

pub unsafe fn zfs_iter_filesystems(
    handle: *mut sys::zfs_handle_t,
    f: unsafe extern "C" fn(*mut sys::zfs_handle_t, *mut libc::c_void) -> libc::c_int,
    ptr: *mut libc::c_void,
) {
    Lazy::force(&LIBZFS_HANDLE);
    sys::zfs_iter_filesystems(handle, Some(f), ptr);
}

pub unsafe fn zfs_iter_snapshots(
    handle: *mut sys::zfs_handle_t,
    simple: bool,
    f: unsafe extern "C" fn(*mut sys::zfs_handle_t, *mut libc::c_void) -> libc::c_int,
    data: *mut libc::c_void,
    min_txg: u64,
    max_txg: u64,
) {
    Lazy::force(&LIBZFS_HANDLE);
    let simple = simple.into();
    // let simple = match simple {
    //     false => libnvpair::boolean_t::B_TRUE,
    //     true => libnvpair::boolean_t::B_FALSE,
    // };
    sys::zfs_iter_snapshots(handle, simple, Some(f), data, min_txg, max_txg);
}

pub fn zfs_version() -> Version {
    LIBZFS_HANDLE.version().clone()
}