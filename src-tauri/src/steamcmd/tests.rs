use super::{
    install::{
        calculate_speed, cleanup_staging, ensure_install_target, extract_archive, install_inner,
    },
    *,
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tauri::ipc::InvokeResponseBody;
use tempfile::tempdir;
use zip::{ZipWriter, write::SimpleFileOptions};

#[test]
fn 检测有效与无效目录() {
    let temp = tempdir().expect("创建临时目录");
    let invalid = inspect_steamcmd(temp.path());
    assert!(!invalid.valid);

    File::create(temp.path().join("steamcmd.exe")).expect("创建测试程序");
    let valid = inspect_steamcmd(temp.path());
    assert!(valid.valid);
    assert!(valid.reason.is_none());
}

#[test]
fn 拒绝覆盖非空目标目录() {
    let temp = tempdir().expect("创建临时目录");
    let target = temp.path().join("SteamCMD");
    fs::create_dir(&target).expect("创建目标目录");
    File::create(target.join("其他文件.txt")).expect("创建占位文件");

    let error = ensure_install_target(temp.path()).expect_err("应拒绝非空目录");
    assert!(error.contains("不为空"));
}

#[test]
fn 安全解压并验证程序() {
    let temp = tempdir().expect("创建临时目录");
    let archive_path = temp.path().join("steamcmd.zip");
    let destination = temp.path().join("output");
    let file = File::create(&archive_path).expect("创建压缩包");
    let mut writer = ZipWriter::new(file);
    writer
        .start_file("steamcmd.exe", SimpleFileOptions::default())
        .expect("写入压缩包条目");
    writer.write_all(b"test").expect("写入测试数据");
    writer.finish().expect("完成压缩包");

    extract_archive(&archive_path, &destination).expect("解压成功");
    let mut content = String::new();
    File::open(destination.join("steamcmd.exe"))
        .expect("打开解压文件")
        .read_to_string(&mut content)
        .expect("读取解压文件");
    assert_eq!(content, "test");
}

#[test]
fn 拒绝压缩包目录穿越() {
    let temp = tempdir().expect("创建临时目录");
    let archive_path = temp.path().join("unsafe.zip");
    let file = File::create(&archive_path).expect("创建压缩包");
    let mut writer = ZipWriter::new(file);
    writer
        .start_file("../outside.exe", SimpleFileOptions::default())
        .expect("写入压缩包条目");
    writer.write_all(b"test").expect("写入测试数据");
    writer.finish().expect("完成压缩包");

    let error =
        extract_archive(&archive_path, &temp.path().join("output")).expect_err("应拒绝不安全路径");
    assert!(error.contains("不安全路径"));
    assert!(!temp.path().join("outside.exe").exists());
}

#[test]
fn 正确计算下载速度() {
    assert_eq!(calculate_speed(1_000, Duration::from_secs(2)), 500);
    assert_eq!(calculate_speed(0, Duration::from_secs(1)), 0);
}

#[test]
fn 清理安装临时目录() {
    let temp = tempdir().expect("创建临时目录");
    let staging = temp.path().join(".steamcmd-installing-test");
    fs::create_dir(&staging).expect("创建安装临时目录");
    File::create(staging.join("steamcmd.zip")).expect("创建临时下载文件");

    cleanup_staging(&staging);
    assert!(!staging.exists());
}

#[tokio::test]
#[ignore = "需要访问 Valve 下载服务器并运行 SteamCMD 首次初始化"]
async fn 真实下载并静默初始化() {
    let temp = tempdir().expect("创建真实下载测试目录");
    let progress_count = Arc::new(AtomicUsize::new(0));
    let progress_count_for_channel = Arc::clone(&progress_count);
    let channel = Channel::new(move |body| {
        if matches!(body, InvokeResponseBody::Json(_)) {
            progress_count_for_channel.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    });

    let result = install_inner(temp.path(), &channel)
        .await
        .expect("真实下载安装应成功");
    assert!(Path::new(&result.executable_path).is_file());
    assert!(progress_count.load(Ordering::Relaxed) >= 4);
    assert!(!temp.path().join("steamcmd.zip").exists());
    assert!(
        fs::read_dir(temp.path())
            .expect("读取真实下载测试目录")
            .all(|entry| !entry
                .expect("读取目录项")
                .file_name()
                .to_string_lossy()
                .starts_with(".steamcmd-installing-"))
    );
}
