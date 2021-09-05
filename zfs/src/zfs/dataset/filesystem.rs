use std::ffi::CString;

use super::core;
use super::libnvpair;
use super::property;
use super::zfs_prop_t;
use super::Result;
use super::ZfsDatasetHandler;

#[derive(Debug)]
pub struct Filesystem {
    dataset_handler: ZfsDatasetHandler,
}

impl Filesystem {
    pub fn destroy(self) -> Result<()> {
        core::destroy_dataset(self.name()).map_err(|err| err.into())
    }

    pub fn name(&self) -> String {
        self.dataset_handler.get_name()
    }

    pub fn available(&self) -> u64 {
        self.dataset_handler
            .get_prop_default_numeric(zfs_prop_t::ZFS_PROP_AVAILABLE)
    }

    pub fn atime(&self) -> property::OnOff {
        let prop = self.dataset_handler.search_property("atime");

        let res = if let Ok(prop) = prop {
            match prop {
                libnvpair::Value::U64(val) => val.into(),
                _ => todo!(),
            }
        } else {
            let default = self
                .dataset_handler
                .get_prop_default_numeric(zfs_prop_t::ZFS_PROP_ATIME);

            if self.dataset_handler.check_mnt_option("atime") && default == 0 {
                property::OnOff::On
            } else if self.dataset_handler.check_mnt_option("noatime") && default != 0 {
                property::OnOff::Off
            } else {
                default.into()
            }
        };

        res
    }

    pub fn get_filesystem(name: impl AsRef<str>) -> Result<Filesystem> {
        let cname = CString::new(name.as_ref())?;
        let dataset_handler = ZfsDatasetHandler::new(cname)?;

        Ok(Filesystem { dataset_handler })
    }
}

#[derive(Debug)]
pub struct FileSystemBuilder {
    nvlist: Result<libnvpair::NvList>,
    name: String,
}

impl FileSystemBuilder {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            nvlist: libnvpair::NvList::new(libnvpair::NvFlag::UniqueName).map_err(|err| err.into()),
            name: name.as_ref().to_string(),
        }
    }

    pub fn atime(mut self, v: impl Into<property::OnOff>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("atime", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn canmount(mut self, v: impl Into<property::OnOffNoAuto>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("canmount", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
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

    pub fn devices(mut self, v: impl Into<property::OnOff>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("devices", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn nbmand(mut self, v: impl Into<property::OnOff>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("nbmand", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn overlay(mut self, v: impl Into<property::OnOff>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("overlay", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn readonly(mut self, v: impl Into<property::OnOff>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("readonly", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn relatime(mut self, v: impl Into<property::OnOff>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("relatime", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn setuid(mut self, v: impl Into<property::OnOff>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("setuid", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn vscan(mut self, v: impl Into<property::OnOff>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("vscan", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn zoned(mut self, v: impl Into<property::OnOff>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("zoned", value.as_str()) {
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

    pub fn exec(mut self, v: impl Into<property::OnOff>) -> Self {
        let value = v.into();
        if let Ok(nvlist) = &mut self.nvlist {
            if let Err(err) = nvlist.add_string("exec", value.as_str()) {
                self.nvlist = Err(err.into());
            }
        }

        self
    }

    pub fn create(mut self) -> Result<Filesystem> {
        let cname = CString::new(self.name.as_bytes())?;
        match self.nvlist.as_mut() {
            Ok(nvlist) => {
                core::create_filesystem(&self.name, nvlist)?;
                let dataset_handler = ZfsDatasetHandler::new(cname)?;
                let filesystem: Filesystem = Filesystem { dataset_handler };

                Ok(filesystem)
            }
            Err(err) => Err(err.clone()), // TODO: check this line because it clones here
        }
    }
}
