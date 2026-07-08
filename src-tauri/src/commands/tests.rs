use crate::{
    server_log::classify_server_log_level,
    server_version::{
        normalize_server_version_value, parse_asa_server_version, parse_manifest_progress,
        read_installed_server_version,
    },
    steamcmd_progress::{
        is_retryable_steamcmd_configuration_error, parse_steamcmd_progress_line, tail_detail,
    },
};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

#[test]
fn parses_steamcmd_download_progress() {
    let parsed = parse_steamcmd_progress_line(
        "Update state (0x61) downloading, Progress: 12.34 (1234000 / 10000000)",
    )
    .expect("SteamCMD progress line should parse");

    assert_eq!(parsed.phase, "downloading");
    assert!(matches!(parsed.percent, Some(percent) if (percent - 12.34).abs() < 0.001));
    assert_eq!(parsed.downloaded_bytes, 1_234_000);
    assert_eq!(parsed.total_bytes, Some(10_000_000));
}

#[test]
fn treats_zero_total_progress_as_unknown_size() {
    let parsed =
        parse_steamcmd_progress_line("Update state (0x3) reconfiguring, progress: 0.00 (0 / 0)")
            .expect("SteamCMD zero-size progress line should parse");

    assert_eq!(parsed.phase, "running");
    assert_eq!(parsed.percent, Some(0.0));
    assert_eq!(parsed.downloaded_bytes, 0);
    assert_eq!(parsed.total_bytes, None);
}

#[test]
fn parses_manifest_download_progress() {
    let manifest = r#"
"AppState"
{
    "BytesToDownload"        "8248424336"
    "BytesDownloaded"        "1678229152"
    "BytesToStage"        "13202439198"
    "BytesStaged"        "4081357112"
}
"#;
    let parsed = parse_manifest_progress(manifest).expect("manifest progress should parse");

    assert_eq!(parsed.phase, "downloading");
    assert_eq!(parsed.downloaded_bytes, 1_678_229_152);
    assert_eq!(parsed.total_bytes, Some(8_248_424_336));
}

#[test]
fn parses_official_asa_server_version_text() {
    assert_eq!(
        parse_asa_server_version("Current ARK Official Server Network Servers Version: v89.24")
            .as_deref(),
        Some("v89.24")
    );
    assert_eq!(
        parse_asa_server_version("2026.07.03 Log: Server Version: 89.24").as_deref(),
        Some("v89.24")
    );
    assert_eq!(
        parse_asa_server_version(r#"{ "BuildVersion": "++ArkAscended+Release-89.24" }"#).as_deref(),
        Some("v89.24")
    );
    assert_eq!(
        parse_asa_server_version(r#""buildid"        "17824567""#),
        None
    );
    assert_eq!(
        normalize_server_version_value("89.24").as_deref(),
        Some("v89.24")
    );
}

#[test]
fn reads_installed_server_version_from_saved_server_log() {
    let temp = tempfile::tempdir().expect("创建临时目录");
    let log_dir = temp.path().join("ShooterGame").join("Saved").join("Logs");
    std::fs::create_dir_all(&log_dir).expect("创建日志目录");
    std::fs::write(
        log_dir.join("server.log"),
        "Current ARK Official Server Network Servers Version: v89.24",
    )
    .expect("写入服务端日志");

    assert_eq!(
        read_installed_server_version(temp.path()).as_deref(),
        Some("v89.24")
    );
}

#[test]
fn tail_detail_prefers_steamcmd_error_over_late_bootstrap_output() {
    let tail = Arc::new(Mutex::new(VecDeque::from([
        "ERROR! Failed to install app '2430930' (Missing configuration)".to_string(),
        "Unloading Steam API...OK".to_string(),
        "Redirecting stderr to 'D:\\Game\\SteamCMD\\logs\\stderr.txt'".to_string(),
        "[----] Verifying installation...".to_string(),
    ])));

    assert_eq!(
        tail_detail(&tail, "SteamCMD 安装/更新失败"),
        "ERROR! Failed to install app '2430930' (Missing configuration)"
    );
}

#[test]
fn detects_retryable_missing_configuration_error() {
    assert!(is_retryable_steamcmd_configuration_error(
        "ERROR! Failed to install app '2430930' (Missing configuration)"
    ));
    assert!(!is_retryable_steamcmd_configuration_error(
        "SteamCMD 安装/更新失败，退出代码：exit code: 1"
    ));
}

#[test]
fn 服务端_info_debug_日志即使来自_stderr_也显示普通级别() {
    assert_eq!(
        classify_server_log_level(
            "07-03 11:09:28.015 25588 1244 I Info/GameAnalytics : Event queue: No events to send",
            "error"
        ),
        "info"
    );
    assert_eq!(
        classify_server_log_level(
            "07-03 11:09:32.751 25588 1244 D Debug/GameAnalytics : body: {}",
            "error"
        ),
        "info"
    );
}

#[test]
fn 服务端警告和错误日志按内容着色() {
    assert_eq!(
        classify_server_log_level("CFCore : Couldn't load mods library from disk", "info"),
        "warn"
    );
    assert_eq!(
        classify_server_log_level(
            "[2026.07.03-07.51.59:997][ 10]Failed Spawned Dino: Piranha | 0.9x",
            "info"
        ),
        "warn"
    );
    assert_eq!(
        classify_server_log_level("Fatal error: Failed to bind server port", "info"),
        "error"
    );
}
