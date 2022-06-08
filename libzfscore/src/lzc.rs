use std::borrow::Cow;
use std::ffi;
use std::os::unix::io::AsRawFd;
use std::ptr;

use once_cell::sync::Lazy;

use super::*;

pub use sys::zfs_handle_t;
pub use sys::zfs_prop_t;
pub use sys::zfs_type_t;

use crate::dataset;
use crate::error::CoreError;
use crate::libzfs;

use super::error::value_or_err;
use super::Result;

static LIBZFS_CORE: Lazy<Lzc> = Lazy::new(Lzc::init);

struct Lzc;

impl Lzc {
    fn init() -> Self {
        let _rc = unsafe { sys::libzfs_core_init() };
        libzfs::zfs_version().ensure_compatible();
        Self
    }

    unsafe fn lzc_create(
        &self,
        name: *const libc::c_char,
        dataset_type: sys::lzc_dataset_type,
        props: *mut libnvpair::nvlist_t,
    ) -> libc::c_int {
        let wkeydata = ptr::null_mut();
        let wkeylen = 0;
        sys::lzc_create(name, dataset_type, props, wkeydata, wkeylen)
    }

    unsafe fn lzc_destroy(&self, name: *const libc::c_char) -> libc::c_int {
        sys::lzc_destroy(name)
    }

    unsafe fn lzc_exists(&self, name: *const libc::c_char) -> sys::boolean_t {
        sys::lzc_exists(name)
    }

    unsafe fn lzc_snapshot(
        &self,
        snaps: *mut libnvpair::nvlist_t,
        props: *mut libnvpair::nvlist_t,
        errlist: *mut *mut libnvpair::nvlist_t,
    ) -> libc::c_int {
        sys::lzc_snapshot(snaps, props, errlist)
    }

    unsafe fn lzc_bookmark(
        &self,
        bookmarks: *mut libnvpair::nvlist_t,
        errlist: *mut *mut libnvpair::nvlist_t,
    ) -> libc::c_int {
        sys::lzc_bookmark(bookmarks, errlist)
    }

    unsafe fn lzc_send(
        &self,
        snapname: *const libc::c_char,
        from: *const libc::c_char,
        fd: libc::c_int,
        flags: sys::lzc_send_flags,
    ) -> libc::c_int {
        sys::lzc_send(snapname, from, fd, flags)
    }

    unsafe fn lzc_send_resume(
        &self,
        snapname: *const libc::c_char,
        from: *const libc::c_char,
        fd: libc::c_int,
        flags: sys::lzc_send_flags,
        resumeobj: u64,
        resumeoff: u64,
    ) -> libc::c_int {
        sys::lzc_send_resume(snapname, from, fd, flags, resumeobj, resumeoff)
    }

    unsafe fn lzc_receive(
        &self,
        snapname: *const libc::c_char,
        props: *mut libnvpair::nvlist_t,
        origin: *const libc::c_char,
        force: sys::boolean_t,
        raw: sys::boolean_t,
        fd: libc::c_int,
    ) -> libc::c_int {
        sys::lzc_receive(snapname, props, origin, force, raw, fd)
    }

    unsafe fn lzc_receive_resumable(
        &self,
        snapname: *const libc::c_char,
        props: *mut libnvpair::nvlist_t,
        origin: *const libc::c_char,
        force: sys::boolean_t,
        raw: sys::boolean_t,
        fd: libc::c_int,
    ) -> libc::c_int {
        sys::lzc_receive_resumable(snapname, props, origin, force, raw, fd)
    }
}

pub fn version() -> libzfs::Version {
    libzfs::zfs_version()
}

pub fn create_filesystem(name: impl AsRef<str>, nvl: nvpair::NvList) -> Result<()> {
    create_dataset(name, sys::lzc_dataset_type::LZC_DATSET_TYPE_ZFS, nvl)
}

pub fn create_volume(name: impl AsRef<str>, nvl: nvpair::NvList) -> Result<()> {
    create_dataset(name, sys::lzc_dataset_type::LZC_DATSET_TYPE_ZVOL, nvl)
}

fn create_dataset(
    name: impl AsRef<str>,
    dataset_type: sys::lzc_dataset_type,
    nvl: nvpair::NvList,
) -> Result<()> {
    let cname = cstring(name)?;
    let rc = unsafe { LIBZFS_CORE.lzc_create(cname.as_ptr(), dataset_type, *nvl) };

    value_or_err((), rc)
}

pub fn snapshot(snapshot: impl AsRef<str>) -> Result<()> {
    snapshots_impl([snapshot])
}

pub fn snapshots(
    dataset: impl AsRef<str>,
    snapshot: impl AsRef<str>,
    recursive: bool,
) -> Result<()> {
    let snapshot = snapshot.as_ref();
    let snapshots = zfs_list_from(dataset)
        .filesystems()
        .volumes()
        .recursive(recursive)
        .get_collection()?
        .into_iter()
        .map(|dataset| format!("{}@{}", dataset.name(), snapshot));

    snapshots_impl(snapshots)
}

// TODO Pass props nvlist
fn snapshots_impl(snapshots: impl IntoIterator<Item = impl AsRef<str>>) -> Result<()> {
    let mut snaps = NvList::new();
    for snapshot in snapshots {
        snaps.add_boolean(snapshot)?;
    }
    let props = ptr::null_mut();
    let mut errlist = NvList::new();
    let rc = unsafe { LIBZFS_CORE.lzc_snapshot(*snaps, props, &mut *errlist) };
    value_or_err((), rc)
}

pub fn dataset_exists(name: impl AsRef<str>) -> Result<()> {
    let cname = cstring(name)?;
    let name = cname.as_ptr();
    let rc = unsafe { LIBZFS_CORE.lzc_exists(name) };

    if rc == sys::boolean_t::B_TRUE {
        Ok(())
    } else {
        Err(CoreError::DatasetNotExist)
    }
}

pub fn destroy_dataset(name: impl AsRef<str>) -> Result<()> {
    let cname = cstring(name)?;
    let name = cname.as_ptr();
    let rc = unsafe { LIBZFS_CORE.lzc_destroy(name) };

    value_or_err((), rc)
}

pub fn bookmark(snapshot: impl AsRef<str>, bookmark: impl AsRef<str>) -> Result<()> {
    let mut bookmarks = NvList::new();
    bookmarks.add_string(bookmark, snapshot)?;
    let rc = unsafe { LIBZFS_CORE.lzc_bookmark(*bookmarks, &mut ptr::null_mut()) };
    value_or_err((), rc)
}

pub fn send<S, F, U>(source: S, from: Option<F>, file: U) -> Result<()>
where
    S: AsRef<str>,
    F: AsRef<str>,
    U: AsRawFd,
{
    let source = cstring(source)?;
    let from = from.map(cstring).transpose()?;
    let flags = sys::lzc_send_flags::LZC_SEND_FLAG_EMBED_DATA
        | sys::lzc_send_flags::LZC_SEND_FLAG_LARGE_BLOCK
        | sys::lzc_send_flags::LZC_SEND_FLAG_COMPRESS;
    let rc = unsafe {
        let source = source.as_ptr();
        let from = from.map_or(ptr::null(), |from| from.as_ptr());
        let fd = file.as_raw_fd();
        LIBZFS_CORE.lzc_send(source, from, fd, flags)
    };
    value_or_err((), rc)
}

pub fn send_resume<S, F, U>(
    source: S,
    from: Option<F>,
    file: U,
    resumeobj: u64,
    resumeoff: u64,
) -> Result<()>
where
    S: AsRef<str>,
    F: AsRef<str>,
    U: AsRawFd,
{
    let source = cstring(source)?;
    let from = from.map(cstring).transpose()?;
    let fd = file.as_raw_fd();
    let flags = sys::lzc_send_flags::LZC_SEND_FLAG_EMBED_DATA
        | sys::lzc_send_flags::LZC_SEND_FLAG_LARGE_BLOCK
        | sys::lzc_send_flags::LZC_SEND_FLAG_COMPRESS;
    let rc = unsafe {
        let source = source.as_ptr();
        let from = from.map_or(ptr::null(), |from| from.as_ptr());
        LIBZFS_CORE.lzc_send_resume(source, from, fd, flags, resumeobj, resumeoff)
    };
    value_or_err((), rc)
}

pub fn receive<S, O, U>(
    snapname: S,
    origin: Option<O>,
    force: bool,
    raw: bool,
    file: U,
) -> Result<()>
where
    S: AsRef<str>,
    O: AsRef<str>,
    U: AsRawFd,
{
    let snapname = cstring(snapname)?;
    let origin = origin.map(cstring).transpose()?;
    let props = NvList::new();
    let force = if force {
        sys::boolean_t::B_TRUE
    } else {
        sys::boolean_t::B_FALSE
    };
    let raw = if raw {
        sys::boolean_t::B_TRUE
    } else {
        sys::boolean_t::B_FALSE
    };
    let fd = file.as_raw_fd();
    let rc = unsafe {
        let snapname = snapname.as_ptr();
        let origin = origin.map_or(ptr::null(), |origin| origin.as_ptr());
        LIBZFS_CORE.lzc_receive(snapname, *props, origin, force, raw, fd)
    };
    value_or_err((), rc)
}

pub fn receive_resumable(
    snapname: impl AsRef<str>,
    origin: impl AsRef<str>,
    force: bool,
    raw: bool,
    file: impl AsRawFd,
) -> Result<()> {
    let snapname = cstring(snapname)?;
    let props = NvList::new();
    let origin = cstring(origin)?;
    let force = if force {
        sys::boolean_t::B_TRUE
    } else {
        sys::boolean_t::B_FALSE
    };
    let raw = if raw {
        sys::boolean_t::B_TRUE
    } else {
        sys::boolean_t::B_FALSE
    };
    let fd = file.as_raw_fd();
    let rc = unsafe {
        let snapname = snapname.as_ptr();
        let origin = origin.as_ptr();
        LIBZFS_CORE.lzc_receive_resumable(snapname, *props, origin, force, raw, fd)
    };
    value_or_err((), rc)
}

pub fn zfs_prop_default_string(property: zfs_prop_t) -> Cow<'static, str> {
    unsafe {
        let cstr = libzfs::zfs_prop_default_string(property);
        ffi::CStr::from_ptr(cstr).to_string_lossy()
    }
}

pub fn zfs_prop_default_numeric(property: zfs_prop_t) -> u64 {
    unsafe { libzfs::zfs_prop_default_numeric(property) }
}

pub fn zfs_list() -> dataset::DatasetCollectorBuilder {
    dataset::DatasetCollectorBuilder::new()
}

pub fn zfs_list_from(name: impl AsRef<str>) -> dataset::DatasetCollectorBuilder {
    dataset::DatasetCollectorBuilder::from(name)
}

pub fn zfs_prop_to_name(property: zfs_prop_t) -> Cow<'static, str> {
    unsafe {
        let cstr = libzfs::zfs_prop_to_name(property);
        ffi::CStr::from_ptr(cstr).to_string_lossy()
    }
}

fn cstring(text: impl AsRef<str>) -> Result<ffi::CString, ffi::NulError> {
    ffi::CString::new(text.as_ref())
}