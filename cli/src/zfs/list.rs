use super::*;

#[derive(Debug, clap::Args)]
pub struct List {
    /// Recursively list children datasets
    #[clap(short)]
    recursive: bool,
    ///  A comma-separated list of types to display
    #[clap(short, value_enum, value_delimiter = ',', default_value = "filesystem")]
    types: Vec<Type>,
    /// Name of the dataset to create
    dataset: Option<String>,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Type {
    Filesystem,
    Snapshot,
    Volume,
    Bookmark,
    All,
}

impl List {
    pub fn exec(self) -> anyhow::Result<String> {
        let mut list = if let Some(parent) = self.dataset {
            zfs::Zfs::list_from(parent)
        } else {
            zfs::Zfs::list()
        };

        for r#type in self.types {
            match r#type {
                Type::Filesystem => list.filesystems(),
                Type::Snapshot => list.snapshots(),
                Type::Volume => list.volumes(),
                Type::Bookmark => list.bookmarks(),
                Type::All => list.all(),
            };
        }

        let text = list
            .recursive(self.recursive)
            // .types(self.types)
            .get()?
            .into_iter()
            .map(|dataset| {
                {
                    fmtools::fmt! {{ dataset.name() } " " { dataset.r#type().name() }}
                }
                .to_string()
            })
            .collect::<Vec<_>>();

        Ok(fmtools::join("\n", text).to_string())
    }
}
