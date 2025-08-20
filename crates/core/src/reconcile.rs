#[derive(Debug, Default)]
pub struct Report {
    pub repaired: usize,
    pub orphaned: usize,
}

pub async fn reconcile() -> Report {
    // TODO: implement walk from root
    Report::default()
}
