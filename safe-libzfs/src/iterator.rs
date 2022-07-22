use std::iter;

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
    parent: Option<String>,
    r#type: libzfs::zfs_type_t,
    recursive: bool,
}

impl DatasetIteratorBuilder {
    pub(crate) fn new() -> Self {
        Self {
            parent: None,
            r#type: libzfs::zfs_type_t(0),
            recursive: false,
        }
    }

    pub(crate) fn from(dataset: impl AsRef<str>) -> Self {
        Self {
            parent: Some(dataset.as_ref().to_owned()),
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
        if let Some(ref parent) = self.parent {
            let parent = ZfsHandle::new(parent)?;
            let datasets = if self.recursive {
                let children = self.get_recursive(&parent);
                iter::once(parent).chain(children).collect()
            } else {
                vec![parent]
            };
            Ok(DatasetIterator::new(datasets))
        } else {
            Ok(self.get_all())
        }
    }

    pub fn get_all(&self) -> DatasetIterator {
        let handles = DatasetCollector::new(self.r#type).get_roots();
        DatasetIterator::new(handles)
    }

    fn get_recursive(&self, parent: &ZfsHandle) -> Vec<ZfsHandle> {
        self.get_children(parent)
            .into_iter()
            .flat_map(|child| {
                let children = self.get_recursive(&child);
                iter::once(child).chain(children)
            })
            .collect()
    }

    fn get_children(&self, parent: &ZfsHandle) -> Vec<ZfsHandle> {
        DatasetCollector::new(self.r#type).get_children(parent)
    }
}

#[derive(Debug)]
struct DatasetCollector {
    types: libzfs::zfs_type_t,
    handles: Vec<ZfsHandle>,
}

impl DatasetCollector {
    fn new(types: libzfs::zfs_type_t) -> Self {
        Self {
            types,
            handles: vec![],
        }
    }

    fn get_children(self, parent: &ZfsHandle) -> Vec<ZfsHandle> {
        if self.types.is_filesystem() || self.types.is_volume() {
            iter_filesystem(self, parent)
        } else if self.types.is_snapshot() {
            iter_snapshots(self, parent)
        } else if self.types.is_bookmark() {
            iter_bookmarks(self, parent)
        } else {
            vec![]
        }
    }

    fn get_roots(self) -> Vec<ZfsHandle> {
        iter_root(self)
    }
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

fn iter_filesystem(collector: DatasetCollector, parent: &ZfsHandle) -> Vec<ZfsHandle> {
    let collector = Box::new(collector);
    let ptr = Box::into_raw(collector) as *mut libc::c_void;
    let collector = unsafe {
        libzfs::zfs_iter_filesystems(**parent, Some(zfs_list_cb), ptr);
        let ptr = ptr as *mut DatasetCollector;
        Box::from_raw(ptr)
    };
    collector.handles
}

fn iter_snapshots(collector: DatasetCollector, parent: &ZfsHandle) -> Vec<ZfsHandle> {
    let collector = Box::new(collector);
    let ptr = Box::into_raw(collector) as *mut libc::c_void;
    let collector = unsafe {
        libzfs::zfs_iter_snapshots(**parent, false, Some(zfs_list_cb), ptr, 0, 0);
        let ptr = ptr as *mut DatasetCollector;
        Box::from_raw(ptr)
    };
    collector.handles
}

fn iter_bookmarks(collector: DatasetCollector, parent: &ZfsHandle) -> Vec<ZfsHandle> {
    let collector = Box::new(collector);
    let ptr = Box::into_raw(collector) as *mut libc::c_void;
    let collector = unsafe {
        libzfs::zfs_iter_bookmarks(**parent, Some(zfs_list_cb), ptr);
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
