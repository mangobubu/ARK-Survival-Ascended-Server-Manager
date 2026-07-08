#[derive(Clone, Copy, Default)]
pub(crate) struct TransferSnapshot {
    pub(crate) percent: Option<f64>,
    pub(crate) downloaded_bytes: u64,
    pub(crate) total_bytes: Option<u64>,
    pub(crate) bytes_per_second: u64,
}

pub(crate) struct ParsedSteamCmdProgress {
    pub(crate) phase: String,
    pub(crate) message: String,
    pub(crate) percent: Option<f64>,
    pub(crate) downloaded_bytes: u64,
    pub(crate) total_bytes: Option<u64>,
}
