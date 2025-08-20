#[derive(Debug, Clone, Copy)]
pub struct ChunkSpec {
    pub index: usize,
    pub offset: usize,
    pub size: usize,
}

/// Plan chunks for a given total size and target max Base64 chars per chunk body.
/// Assumes ~4 chars encode 3 bytes; keeps a little slack for headers.
pub fn plan_chunks(total_size: usize, max_payload_chars: usize) -> Vec<ChunkSpec> {
    let mut max_bytes = (max_payload_chars / 4) * 3;
    if max_bytes == 0 { max_bytes = 1024; }
    let mut out = Vec::new();
    let mut off = 0usize;
    let mut idx = 0usize;
    while off < total_size {
        let n = max_bytes.min(total_size - off);
        out.push(ChunkSpec { index: idx, offset: off, size: n });
        off += n; idx += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plans_nonempty() {
        let v = plan_chunks(5000, 1600);
        assert!(!v.is_empty());
        assert_eq!(v.iter().map(|c| c.size).sum::<usize>(), 5000);
    }
}
