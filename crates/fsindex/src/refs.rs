use anyhow::*;
use serde::{Serialize, Deserialize};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeadRef {
    pub head: String,
    pub prev: Option<String>,
    pub node: String,
}

fn sanitize(path: &str) -> String {
    path.replace('/', "_")
}

pub fn load_head(repo_root: &str, rel_path: &str) -> Result<Option<HeadRef>> {
    let p = Path::new(repo_root).join("refs").join(format!("{}.head.json", sanitize(rel_path)));
    if !p.exists() { return Ok(None); }
    let bytes = fs::read(p)?;
    let head = serde_json::from_slice::<HeadRef>(&bytes)?;
    Ok(Some(head))
}

pub fn save_head(repo_root: &str, rel_path: &str, head: &HeadRef) -> Result<()> {
    let dir = Path::new(repo_root).join("refs");
    fs::create_dir_all(&dir)?;
    let p = dir.join(format!("{}.head.json", sanitize(rel_path)));
    let tmp = dir.join(format!("{}.head.json.tmp", sanitize(rel_path)));
    fs::write(&tmp, serde_json::to_vec_pretty(head)?)?;
    fs::rename(tmp, p)?;
    Ok(())
}
