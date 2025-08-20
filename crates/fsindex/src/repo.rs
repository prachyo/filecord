use anyhow::*;
use serde::{Serialize, Deserialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoConfig {
    pub bot_token: String,
    pub application_id: String,
    pub guild_id: String,
    pub root_channel_id: String,
    pub public_key: String,
}

#[derive(Debug, Clone)]
pub struct Repo {
    root: PathBuf,
}

impl Repo {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    pub fn path(&self) -> &PathBuf { &self.root }

    pub fn ensure_layout(&self) -> Result<()> {
        for p in ["refs", "threads", "manifests", "cids", "lost+found", "keys"] {
            fs::create_dir_all(self.root.join(p))?;
        }
        Ok(())
    }

    pub fn load_config(&self) -> Result<RepoConfig> {
        let p = self.root.join("config.json");
        let bytes = fs::read(p)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn save_config(&self, cfg: &RepoConfig) -> Result<()> {
        let p = self.root.join("config.json");
        let tmp = self.root.join("config.json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(cfg)?)?;
        fs::rename(tmp, p)?;
        Ok(())
    }
}
