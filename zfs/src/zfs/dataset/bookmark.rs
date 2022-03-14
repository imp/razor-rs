use super::*;

#[derive(Debug)]
pub struct Bookmark {
    dataset: ZfsDatasetHandle,
}

impl Bookmark {
    pub fn get(name: impl AsRef<str>) -> Result<Self> {
        let name = ffi::CString::new(name.as_ref())?;
        let dataset = ZfsDatasetHandle::new(name)?;

        Ok(Self { dataset })
    }

    pub fn destroy(self) -> Result<()> {
        lzc::destroy_dataset(self.name()).map_err(|err| err.into())
    }

    pub fn name(&self) -> String {
        self.dataset.name().to_string()
    }

    #[inline]
    pub fn guid(&self) -> u64 {
        self.dataset.numeric_property(ZFS_PROP_GUID)
    }

    #[inline]
    pub fn creation(&self) -> u64 {
        self.dataset.numeric_property(ZFS_PROP_CREATION)
    }

    #[inline]
    pub fn createtxg(&self) -> u64 {
        self.dataset.numeric_property(ZFS_PROP_CREATETXG)
    }
}
