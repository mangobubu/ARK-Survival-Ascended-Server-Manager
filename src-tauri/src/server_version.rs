mod files;
mod manifest;
mod parser;
mod types;

pub(crate) use files::{
    is_server_log_candidate, read_installed_server_version, server_appmanifest_path,
    with_current_server_version,
};
pub(crate) use manifest::parse_manifest_progress;
#[cfg(test)]
pub(crate) use parser::normalize_server_version_value;
pub(crate) use parser::parse_asa_server_version;

const SERVER_VERSION_SCAN_BYTES: u64 = 512 * 1024;
