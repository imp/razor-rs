use std::ffi::CString;

use razor_zfscore::ZfsDatasetHandler;

use super::core;
use super::libnvpair;
use super::property;
use super::Result;

#[derive(Debug)]
pub struct Volume {
    dataset_handler: ZfsDatasetHandler,
}

impl Volume {
    pub fn destroy(self) -> Result<()> {
        core::destroy_dataset(self.name()).map_err(|err| err.into())
    }

    pub fn name(&self) -> String {
        self.dataset_handler.get_name()
    }

    pub fn get_volume(name: impl AsRef<str>) -> Result<Volume> {
        let cname = CString::new(name.as_ref())?;
        let dataset_handler = ZfsDatasetHandler::new(cname)?;

        Ok(Volume { dataset_handler })
    }
}

#[derive(Debug)]
pub struct VolumeBuilder {
    nvlist: Result<libnvpair::NvList>,
    name: String,
    volblocksize: u64,
}

impl VolumeBuilder {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            nvlist: libnvpair::NvList::new(libnvpair::NvFlag::UniqueName)
                .map_err(|nvlist_err| nvlist_err.into()),
            name: name.as_ref().to_string(),
            volblocksize: Self::calculate_default_volblocksize(),
        }
    }

    pub fn checksum(mut self, v: impl Into<property::CheckSumAlgo>) -> Self {
        let value = v.into();

        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("checksum", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn compression(mut self, v: impl Into<property::CompressionAlgo>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("compression", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn blocksize(mut self, v: u64) -> Self {
        self.volblocksize = v;
        self
    }

    // TODO: implement calculation algorithm
    fn calculate_default_volblocksize() -> u64 {
        8192
    }

    pub fn volmode(mut self, v: impl Into<property::VolModeId>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_uint64("volmode", value.into()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    // TODO: 1. default block size should be calculated
    //       2. volsize should be multiple of volblocksize and rounded to nearest 128k bytes
    //       3. add noreserve functionality
    //       4. add parents creation if needed
    //       5. add zfs_mount_and_share functionality
    pub fn create(mut self, size: u64) -> Result<Volume> {
        #[inline]
        fn _is_power_of_two(num: u64) -> bool {
            (num != 0) && ((num & (num - 1)) == 0)
        }
        dbg!("creating volume");
        let cname = CString::new(self.name.as_bytes())?;
        match self.nvlist.as_mut() {
            Ok(nvlist) => {
                nvlist.add_uint64("volsize", size)?;

                // TODO: check if volblocksize is power of 2 and between 512 and 128000
                nvlist.add_uint64("volblocksize", self.volblocksize)?;

                core::create_volume(&self.name, nvlist)?;
                let dataset_handler = ZfsDatasetHandler::new(cname)?;

                let volume: Volume = Volume { dataset_handler };

                Ok(volume)
            }
            Err(err) => Err(err.clone()), // TODO: check this line because it clones here
        }
    }
}
