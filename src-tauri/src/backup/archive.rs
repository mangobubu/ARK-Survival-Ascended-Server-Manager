use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

pub(super) fn zip_directory(source: &Path, destination: &Path) -> Result<(), String> {
    let file = File::create(destination)
        .map_err(|error| format!("无法创建备份文件 {}：{error}", destination.display()))?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    add_directory_entries(source, source, &mut writer, options)?;
    writer
        .finish()
        .map_err(|error| format!("无法完成备份压缩 {}：{error}", destination.display()))?;
    Ok(())
}

fn add_directory_entries(
    root: &Path,
    current: &Path,
    writer: &mut ZipWriter<File>,
    options: SimpleFileOptions,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|error| format!("无法读取目录 {}：{error}", current.display()))?
    {
        let entry = entry.map_err(|error| format!("读取目录项失败：{error}"))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("生成备份相对路径失败：{error}"))?;
        let archive_name = relative.to_string_lossy().replace('\\', "/");
        if path.is_dir() {
            writer
                .add_directory(format!("{archive_name}/"), options)
                .map_err(|error| format!("写入备份目录失败：{error}"))?;
            add_directory_entries(root, &path, writer, options)?;
        } else {
            writer
                .start_file(archive_name, options)
                .map_err(|error| format!("写入备份文件头失败：{error}"))?;
            let mut input = File::open(&path)
                .map_err(|error| format!("打开待备份文件 {} 失败：{error}", path.display()))?;
            io::copy(&mut input, writer)
                .map_err(|error| format!("压缩备份文件 {} 失败：{error}", path.display()))?;
        }
    }
    Ok(())
}

pub(super) fn unzip_to_directory(archive_path: &Path, target: &Path) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("无法打开备份文件 {}：{error}", archive_path.display()))?;
    let mut archive = ZipArchive::new(file).map_err(|error| format!("备份压缩包无效：{error}"))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("读取备份压缩包条目失败：{error}"))?;
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| "备份压缩包包含不安全路径，已终止恢复".to_string())?;
        let output_path = target.join(enclosed);
        if entry.is_dir() {
            fs::create_dir_all(&output_path)
                .map_err(|error| format!("创建恢复目录失败：{error}"))?;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("创建恢复目录失败：{error}"))?;
        }
        let mut output = File::create(&output_path)
            .map_err(|error| format!("创建恢复文件 {} 失败：{error}", output_path.display()))?;
        io::copy(&mut entry, &mut output)
            .map_err(|error| format!("恢复文件 {} 失败：{error}", output_path.display()))?;
        output
            .flush()
            .map_err(|error| format!("刷新恢复文件 {} 失败：{error}", output_path.display()))?;
    }
    Ok(())
}
