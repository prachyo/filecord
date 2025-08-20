use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunkRef {
    pub i: usize,
    pub cid: String,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    pub v: u8,
    pub r#type: String, // "file"
    pub name: String,
    pub size: u64,
    pub chunks: Vec<FileChunkRef>,
    pub file_cid: String,
    #[serde(default)]
    pub enc: Option<EncDesc>,
    #[serde(default)]
    pub z: Option<String>,
    pub ctime: i64,
    pub mtime: i64,
    #[serde(default)]
    pub prev: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncDesc {
    pub alg: String,
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirChild {
    pub t: String,      // "dir" | "file"
    pub name: String,
    pub manifest: String,
    pub thread: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirManifest {
    pub v: u8,
    pub r#type: String, // "dir"
    pub name: String,
    pub children: Vec<DirChild>,
    pub ctime: i64,
    pub mtime: i64,
    #[serde(default)]
    pub prev: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadPointer {
    pub head: String,
    pub prev: Option<String>,
    pub node: String,
}
