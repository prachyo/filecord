pub enum CidAlgo { Blake3, Sha256 }

pub fn cid_blake3(data: &[u8]) -> String {
    let h = blake3::hash(data);
    format!("blake3:{}", h.to_hex())
}

pub fn cid_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let out = hasher.finalize();
    format!("sha256:{}", hex::encode(out))
}
