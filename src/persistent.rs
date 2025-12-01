use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Serialize, Deserialize)]
struct PersistentData {
    ip: Option<String>,
}

pub struct Persistance {
    file_path: PathBuf,
    data: PersistentData,
}

impl PersistentData {
    pub fn new<D>(file_path: D) -> Result<Self>
    where
        D: AsRef<Path>,
    {
        let data = if file_path.as_ref().exists() {
            PersistentData::from_file(file_path)?
        } else {
            Self { ip: None }
        };

        Ok(data)
    }

    pub fn from_file<D>(file_path: D) -> Result<PersistentData>
    where
        D: AsRef<Path>,
    {
        let data = fs::read_to_string(file_path)?;
        let data: PersistentData = serde_json::from_str(&data)?;
        Ok(data)
    }

    pub fn sync(&self, file_path: &Path) -> Result<()> {
        let data = serde_json::to_string(self)?;
        fs::write(file_path, data.as_bytes())?;
        Ok(())
    }
}

impl Persistance {
    pub fn new<D, S>(data_dir: D, domain: S) -> Result<Self>
    where
        D: AsRef<Path>,
        S: AsRef<str>,
    {
        let file_name = format!("{}.json", domain.as_ref());
        let file_path = data_dir.as_ref().join(file_name);
        let data = PersistentData::new(&file_path)?;

        Ok(Self { file_path, data })
    }

    pub fn ip_changed<S>(&self, new_ip: S) -> bool
    where
        S: AsRef<str>,
    {
        if let Some(ip) = &self.data.ip {
            ip != new_ip.as_ref()
        } else {
            true
        }
    }

    pub fn update<S>(&mut self, new_ip: S) -> Result<()>
    where
        S: AsRef<str>,
    {
        self.data.ip = Some(new_ip.as_ref().into());
        self.data.sync(&self.file_path)
    }
}
