use once_cell::sync::Lazy;
use parking_lot::Mutex;

use super::*;

static DATASET_ITERATOR: Lazy<Mutex<DatasetIterator>> = Lazy::new(|| {
    let iterator = DatasetIterator::new();
    Mutex::new(iterator)
});

struct DatasetIterator {}

impl DatasetIterator {
    fn new() -> Self {
        Self {}
    }

    fn iter_root(&self) -> Vec<*mut libzfs::zfs_handle_t> {
        let handles: Box<Vec<*mut libzfs::zfs_handle_t>> = Box::new(vec![]);
        let ptr = Box::into_raw(handles) as *mut libc::c_void;
        let handles = unsafe {
            libzfs::zfs_iter_root(Some(zfs_list_cb), ptr);
            let ptr = ptr as *mut Vec<*mut libzfs::zfs_handle_t>;
            Box::from_raw(ptr)
        };
        *handles
    }

    fn iter_filesystem(&self, parent: *mut libzfs::zfs_handle_t) -> Vec<*mut libzfs::zfs_handle_t> {
        let handles: Box<Vec<*mut libzfs::zfs_handle_t>> = Box::new(vec![]);
        let ptr = Box::into_raw(handles) as *mut libc::c_void;
        let handles = unsafe {
            libzfs::zfs_iter_filesystems(parent, Some(zfs_list_cb), ptr);
            let ptr = ptr as *mut Vec<*mut libzfs::zfs_handle_t>;
            Box::from_raw(ptr)
        };
        *handles
    }

    fn iter_snapshots(&self, parent: *mut libzfs::zfs_handle_t) -> Vec<*mut libzfs::zfs_handle_t> {
        let handles: Box<Vec<*mut libzfs::zfs_handle_t>> = Box::new(vec![]);
        let ptr = Box::into_raw(handles) as *mut libc::c_void;
        let handles = unsafe {
            libzfs::zfs_iter_snapshots(parent, false, Some(zfs_list_cb), ptr, 0, 0);
            let ptr = ptr as *mut Vec<*mut libzfs::zfs_handle_t>;
            Box::from_raw(ptr)
        };
        *handles
    }
}

#[derive(Debug)]
pub struct DatasetIteratorBuilder {
    from_dataset: Option<String>,
    datasets: Vec<ZfsHandle>,
    r#type: libzfs::zfs_type_t,
    recursive: bool,
}

impl DatasetIteratorBuilder {
    pub(crate) fn new() -> Self {
        Self {
            from_dataset: None,
            datasets: Vec::new(),
            r#type: libzfs::zfs_type_t(0),
            recursive: false,
        }
    }

    pub(crate) fn from(dataset: impl AsRef<str>) -> Self {
        Self {
            from_dataset: Some(dataset.as_ref().to_owned()),
            datasets: Vec::new(),
            r#type: libzfs::zfs_type_t(0),
            recursive: false,
        }
    }

    pub fn filesystems(mut self) -> Self {
        self.r#type |= libzfs::zfs_type_t::ZFS_TYPE_FILESYSTEM;

        self
    }

    pub fn volumes(mut self) -> Self {
        self.r#type |= libzfs::zfs_type_t::ZFS_TYPE_VOLUME;

        self
    }

    pub fn snapshots(mut self) -> Self {
        self.r#type |= libzfs::zfs_type_t::ZFS_TYPE_SNAPSHOT;

        self
    }

    pub fn bookmarks(mut self) -> Self {
        self.r#type |= libzfs::zfs_type_t::ZFS_TYPE_BOOKMARK;

        self
    }

    pub fn recursive(mut self, yes: bool) -> Self {
        self.recursive = yes;
        self
    }

    fn recursive_children(&mut self, handle: Option<&ZfsHandle>) {
        let childrens = Self::get_children(handle, self.r#type);

        for child in childrens {
            self.recursive_children(Some(&child));
            if self.r#type.contains(child.r#type()) {
                self.datasets.push(child);
            }
        }
    }

    pub fn get_collection(mut self) -> DatasetCollector {
        let handle = self
            .from_dataset
            .as_ref()
            .and_then(|name| ZfsHandle::new(name).ok());

        let children = Self::get_children(handle.as_ref(), self.r#type);

        for child in children {
            if self.recursive {
                self.recursive_children(Some(&child))
            }

            if self.r#type.contains(child.r#type()) {
                self.datasets.push(child);
            }
        }

        DatasetCollector::new(self.datasets)
    }

    pub fn get_children(
        parent: Option<&ZfsHandle>,
        r#type: libzfs::zfs_type_t,
    ) -> impl Iterator<Item = ZfsHandle> {
        parent
            .map(|parent| parent.handle)
            .map_or_else(
                || DATASET_ITERATOR.lock().iter_root(),
                |parent| {
                    let mut fs = Vec::new();
                    let mut snapshots = Vec::new();
                    if r#type.is_filesystem() || r#type.is_volume() {
                        fs = DATASET_ITERATOR.lock().iter_filesystem(parent);
                    }

                    if r#type.is_snapshot() {
                        snapshots = DATASET_ITERATOR.lock().iter_snapshots(parent);
                    }

                    fs.append(&mut snapshots);
                    fs
                },
            )
            .into_iter()
            .map(ZfsHandle::from)
    }
}

#[derive(Debug)]
pub struct DatasetCollector {
    datasets: Vec<ZfsHandle>,
}

impl DatasetCollector {
    pub(crate) fn new(datasets: Vec<ZfsHandle>) -> Self {
        Self { datasets }
    }

    pub fn len(&self) -> usize {
        self.datasets.len()
    }
}

impl IntoIterator for DatasetCollector {
    type Item = ZfsHandle;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.datasets.into_iter()
    }
}

unsafe extern "C" fn zfs_list_cb(
    handle: *mut libzfs::zfs_handle_t,
    ptr: *mut libc::c_void,
) -> libc::c_int {
    let children = ptr as *mut Vec<*mut libzfs::zfs_handle_t>;
    (*children).push(handle);
    0
}
