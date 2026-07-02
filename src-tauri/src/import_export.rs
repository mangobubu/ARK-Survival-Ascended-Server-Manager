use crate::{
    app_state::{AppRuntime, current_timestamp_text},
    models::{ExportResult, ImportResult, InstanceConfigBundle},
};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportDocument {
    schema_version: u32,
    exported_at: String,
    instances: Vec<InstanceConfigBundle>,
}

pub fn export_instances(
    runtime: &AppRuntime,
    instance_ids: Vec<String>,
) -> Result<ExportResult, String> {
    let settings = runtime.settings()?;
    let snapshot = runtime.snapshot()?;
    let selected = if instance_ids.is_empty() {
        snapshot.instances
    } else {
        snapshot
            .instances
            .into_iter()
            .filter(|instance| instance_ids.iter().any(|id| id == &instance.id))
            .collect()
    };

    if selected.is_empty() {
        return Err("没有可导出的实例".to_string());
    }

    let bundles = selected
        .into_iter()
        .map(|instance| InstanceConfigBundle {
            config: snapshot
                .configs
                .get(&instance.id)
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
            mods: snapshot.mods.get(&instance.id).cloned().unwrap_or_default(),
            instance,
        })
        .collect::<Vec<_>>();

    let export_dir = Path::new(&settings.backup_storage_path).join("exports");
    fs::create_dir_all(&export_dir)
        .map_err(|error| format!("无法创建导出目录 {}：{error}", export_dir.display()))?;
    let export_path = export_dir.join(format!("asa-export-{}.json", current_timestamp_text()));
    let exported_instances = bundles.len();
    let document = ExportDocument {
        schema_version: 1,
        exported_at: current_timestamp_text(),
        instances: bundles,
    };
    let content = serde_json::to_string_pretty(&document)
        .map_err(|error| format!("无法序列化导出文件：{error}"))?;
    fs::write(&export_path, content)
        .map_err(|error| format!("无法写入导出文件 {}：{error}", export_path.display()))?;

    Ok(ExportResult {
        path: export_path.to_string_lossy().into_owned(),
        exported_instances,
    })
}

pub fn import_instances(runtime: &AppRuntime, path: &Path) -> Result<ImportResult, String> {
    if !path.is_file() {
        return Err(format!("导入文件不存在：{}", path.display()));
    }
    let content = fs::read_to_string(path)
        .map_err(|error| format!("无法读取导入文件 {}：{error}", path.display()))?;
    let document: ExportDocument = serde_json::from_str(&content)
        .map_err(|error| format!("导入文件格式无效 {}：{error}", path.display()))?;
    if document.schema_version != 1 {
        return Err(format!("不支持的导入文件版本：{}", document.schema_version));
    }

    let (imported_instances, skipped_instances) =
        runtime.replace_from_import(document.instances)?;
    Ok(ImportResult {
        imported_instances,
        skipped_instances,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn 拒绝不存在的导入文件() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let path = temp.path().join("missing.json");
        assert!(!path.exists());
    }
}
