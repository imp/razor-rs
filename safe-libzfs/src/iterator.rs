use super::*;

#[derive(Debug)]
pub struct DatasetIterator {
    handles: Vec<ZfsHandle>,
}

impl DatasetIterator {
    pub(crate) fn new(handles: Vec<ZfsHandle>) -> Self {
        Self { handles }
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }
}

impl IntoIterator for DatasetIterator {
    type Item = ZfsHandle;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.handles.into_iter()
    }
}

#[derive(Debug)]
pub struct DatasetIteratorBuilder {
    from_dataset: Option<String>,
    r#type: libzfs::zfs_type_t,
    recursive: bool,
}

impl DatasetIteratorBuilder {
    pub(crate) fn new() -> Self {
        Self {
            from_dataset: None,
            r#type: libzfs::zfs_type_t(0),
            recursive: false,
        }
    }

    pub(crate) fn from(dataset: impl AsRef<str>) -> Self {
        Self {
            from_dataset: Some(dataset.as_ref().to_owned()),
            r#type: libzfs::zfs_type_t(0),
            recursive: false,
        }
    }

    pub fn filesystems(&mut self) -> &mut Self {
        self.r#type |= libzfs::zfs_type_t::ZFS_TYPE_FILESYSTEM;
        self
    }

    pub fn volumes(&mut self) -> &mut Self {
        self.r#type |= libzfs::zfs_type_t::ZFS_TYPE_VOLUME;
        self
    }

    pub fn snapshots(&mut self) -> &mut Self {
        self.r#type |= libzfs::zfs_type_t::ZFS_TYPE_SNAPSHOT;
        self
    }

    pub fn bookmarks(&mut self) -> &mut Self {
        self.r#type |= libzfs::zfs_type_t::ZFS_TYPE_BOOKMARK;
        self
    }

    pub fn all(&mut self) -> &mut Self {
        self.filesystems().volumes().snapshots().bookmarks()
    }

    pub fn recursive(&mut self, yes: bool) -> &mut Self {
        self.recursive = yes;
        self
    }

    pub fn get(&self) -> Result<DatasetIterator, ZfsError> {
        let collector = DatasetCollector {
            types: self.r#type,
            handles: vec![],
        };

        let handles = if let Some(ref parent) = self.from_dataset {
            let parent = ZfsHandle::new(parent)?;
            iter_filesystem(collector, parent)
        } else {
            iter_root(collector)
        };

        Ok(DatasetIterator::new(handles))
    }
}

#[derive(Debug)]
struct DatasetCollector {
    types: libzfs::zfs_type_t,
    handles: Vec<ZfsHandle>,
}

fn iter_root(collector: DatasetCollector) -> Vec<ZfsHandle> {
    let collector = Box::new(collector);
    let ptr = Box::into_raw(collector) as *mut libc::c_void;
    let collector = unsafe {
        libzfs::zfs_iter_root(Some(zfs_list_cb), ptr);
        let ptr = ptr as *mut DatasetCollector;
        Box::from_raw(ptr)
    };
    collector.handles
}

fn iter_filesystem(collector: DatasetCollector, parent: ZfsHandle) -> Vec<ZfsHandle> {
    let collector = Box::new(collector);
    let ptr = Box::into_raw(collector) as *mut libc::c_void;
    let collector = unsafe {
        libzfs::zfs_iter_filesystems(*parent, Some(zfs_list_cb), ptr);
        let ptr = ptr as *mut DatasetCollector;
        Box::from_raw(ptr)
    };
    collector.handles
}

fn _iter_snapshots(collector: DatasetCollector, parent: ZfsHandle) -> Vec<ZfsHandle> {
    let collector = Box::new(collector);
    let ptr = Box::into_raw(collector) as *mut libc::c_void;
    let collector = unsafe {
        libzfs::zfs_iter_snapshots(*parent, false, Some(zfs_list_cb), ptr, 0, 0);
        let ptr = ptr as *mut DatasetCollector;
        Box::from_raw(ptr)
    };
    collector.handles
}

unsafe extern "C" fn zfs_list_cb(
    handle: *mut libzfs::zfs_handle_t,
    ptr: *mut libc::c_void,
) -> libc::c_int {
    let handle = ZfsHandle::from(handle);
    let collector = ptr as *mut DatasetCollector;
    if (*collector).types.contains(handle.r#type()) {
        (*collector).handles.push(handle);
    }
    0
}
