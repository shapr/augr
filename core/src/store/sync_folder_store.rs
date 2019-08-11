use crate::{Meta, Patch, PatchRef, Store};
use snafu::{ResultExt, Snafu};
use std::{
    fs::{create_dir_all, read_to_string, OpenOptions},
    io::Write,
    path::PathBuf,
};
use toml;

pub struct SyncFolderStore {
    /// Whether the repository should create a new file if one is not found
    init: bool,
    root_folder: PathBuf,
    patch_folder: PathBuf,
    device_id: String,
}

#[derive(Debug, Snafu)]
pub enum SyncFolderStoreError {
    #[snafu(display("Unable to deserialize meta {}: {}", device_id, source))]
    DeserializeMeta {
        source: toml::de::Error,
        device_id: String,
    },

    #[snafu(display("Unable to serialize meta {}: {}", device_id, source))]
    SerializeMeta {
        source: toml::ser::Error,
        device_id: String,
    },

    #[snafu(display("Unable to deserialize meta {}: {}", patch_ref, source))]
    DeserializePatch {
        source: toml::de::Error,
        patch_ref: String,
    },

    #[snafu(display("Unable to read file {}: {}", path.display(), source))]
    ReadFile {
        source: std::io::Error,
        path: PathBuf,
    },

    #[snafu(display("Unable to write file {}: {}", path.display(), source))]
    WriteFile {
        source: std::io::Error,
        path: PathBuf,
    },
}

impl SyncFolderStore {
    pub fn new(root_folder: PathBuf, device_id: String) -> Self {
        Self {
            init: false,
            device_id,
            patch_folder: root_folder.join("patches"),
            root_folder: root_folder,
        }
    }

    pub fn should_init(mut self, should_init: bool) -> Self {
        self.init = true;
        self
    }

    fn meta_file_path(&self) -> PathBuf {
        self.root_folder
            .join("meta")
            .join(self.device_id.clone())
            .with_extension("toml")
    }
}

impl Store for SyncFolderStore {
    type Error = SyncFolderStoreError;

    fn get_meta(&self) -> Result<Meta, Self::Error> {
        let path = self.meta_file_path();

        let meta;
        if path.exists() {
            let contents = read_to_string(&path).context(ReadFile { path })?;

            meta = toml::de::from_str(&contents).context(DeserializeMeta {
                device_id: self.device_id.clone(),
            })?;
        } else {
            meta = Meta::new();
        }

        Ok(meta)
    }

    fn save_meta(&mut self, meta: &Meta) -> Result<(), Self::Error> {
        let contents = toml::ser::to_vec(&meta).context(SerializeMeta {
            device_id: self.device_id.clone(),
        })?;

        let path = self.meta_file_path();

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                create_dir_all(parent).context(WriteFile { path: parent })?;
            }
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(path.clone())
            .context(WriteFile { path: path.clone() })?;

        file.write_all(contents.as_slice())
            .context(WriteFile { path: path.clone() })?;

        Ok(())
    }

    fn get_patch(&self, patch_ref: &PatchRef) -> Result<Patch, Self::Error> {
        let path = self
            .patch_folder
            .join(patch_ref.to_string())
            .with_extension("toml");

        let contents = read_to_string(&path).context(ReadFile { path })?;

        let patch = toml::de::from_str(&contents).context(DeserializePatch {
            patch_ref: patch_ref.to_string(),
        })?;

        Ok(patch)
    }

    fn add_patch(&mut self, patch: &Patch) -> Result<(), Self::Error> {
        let patch_ref = patch.patch_ref().to_string();
        let path = self.patch_folder.join(&patch_ref).with_extension("toml");

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                create_dir_all(parent).context(WriteFile { path: parent })?;
            }
        }

        let contents = toml::ser::to_vec(patch).context(SerializeMeta {
            device_id: self.device_id.clone(),
        })?;

        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.clone())
            .context(WriteFile { path: path.clone() })?;

        file.write_all(contents.as_slice())
            .context(WriteFile { path: path.clone() })?;

        Ok(())
    }
}