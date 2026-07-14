use crate::{
    app_state::{AppRuntime, current_timestamp_text},
    asa_config_metadata,
    models::{ExportResult, ImportResult, InstanceConfigBundle},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::Path};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportDocument {
    schema_version: u32,
    exported_at: String,
    instances: Vec<InstanceConfigBundle>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportTransfer {
    pub(crate) file_name: String,
    pub(crate) content: String,
    pub(crate) exported_instances: usize,
}

pub fn export_instances(
    runtime: &AppRuntime,
    instance_ids: Vec<String>,
) -> Result<ExportResult, String> {
    let transfer = export_instances_for_transfer(runtime, instance_ids)?;
    let settings = runtime.settings()?;
    let export_dir = Path::new(&settings.backup_storage_path).join("exports");
    fs::create_dir_all(&export_dir)
        .map_err(|error| format!("无法创建导出目录 {}：{error}", export_dir.display()))?;
    let export_path = export_dir.join(&transfer.file_name);
    fs::write(&export_path, transfer.content)
        .map_err(|error| format!("无法写入导出文件 {}：{error}", export_path.display()))?;

    Ok(ExportResult {
        path: export_path.to_string_lossy().into_owned(),
        exported_instances: transfer.exported_instances,
    })
}

pub(crate) fn export_instances_for_transfer(
    runtime: &AppRuntime,
    instance_ids: Vec<String>,
) -> Result<ExportTransfer, String> {
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
        .map(|instance| {
            let mut config = snapshot
                .configs
                .get(&instance.id)
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            redact_sensitive_config(&mut config);
            InstanceConfigBundle {
                config,
                mods: snapshot.mods.get(&instance.id).cloned().unwrap_or_default(),
                instance,
            }
        })
        .collect::<Vec<_>>();

    let exported_at = current_timestamp_text();
    let exported_instances = bundles.len();
    let document = ExportDocument {
        schema_version: 1,
        exported_at: exported_at.clone(),
        instances: bundles,
    };
    let content = serde_json::to_string_pretty(&document)
        .map_err(|error| format!("无法序列化导出文件：{error}"))?;

    Ok(ExportTransfer {
        file_name: format!("asa-export-{exported_at}.json"),
        content,
        exported_instances,
    })
}

fn redact_sensitive_config(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map.iter_mut() {
                if is_sensitive_export_key(key) {
                    *value = Value::String(String::new());
                } else {
                    redact_sensitive_config(value);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                redact_sensitive_config(item);
            }
        }
        _ => {}
    }
}

fn is_sensitive_export_key(key: &str) -> bool {
    asa_config_metadata::is_sensitive_export_key(key)
}

pub fn import_instances(runtime: &AppRuntime, path: &Path) -> Result<ImportResult, String> {
    if !path.is_file() {
        return Err(format!("导入文件不存在：{}", path.display()));
    }
    let content = fs::read_to_string(path)
        .map_err(|error| format!("无法读取导入文件 {}：{error}", path.display()))?;
    let document: ExportDocument = serde_json::from_str(&content)
        .map_err(|error| format!("导入文件格式无效 {}：{error}", path.display()))?;
    import_document(runtime, document)
}

pub(crate) fn import_instances_from_content(
    runtime: &AppRuntime,
    content: &str,
) -> Result<ImportResult, String> {
    let document: ExportDocument =
        serde_json::from_str(content).map_err(|error| format!("JSON 解析失败：{error}"))?;
    import_document(runtime, document)
}

fn import_document(runtime: &AppRuntime, document: ExportDocument) -> Result<ImportResult, String> {
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
    use super::*;
    use serde_json::json;

    #[test]
    fn 拒绝不存在的导入文件() {
        let temp = tempfile::tempdir().expect("创建临时目录");
        let path = temp.path().join("missing.json");
        assert!(!path.exists());
    }

    #[test]
    fn 敏感导出字段由配置元数据驱动递归脱敏() {
        let mut value = json!({
            "sessionName": "公开名称",
            "serverPassword": "join-secret",
            "adminPassword": "admin-secret",
            "nested": {
                "spectatorPassword": "spectator-secret",
                "webAdminPassword": "web-secret",
                "items": [
                    { "webAcmeTencentSecretKey": "tencent-secret" },
                    { "sessionName": "嵌套公开名称" }
                ]
            }
        });

        redact_sensitive_config(&mut value);

        assert_eq!(value["sessionName"], "公开名称");
        assert_eq!(value["serverPassword"], "");
        assert_eq!(value["adminPassword"], "");
        assert_eq!(value["nested"]["spectatorPassword"], "");
        assert_eq!(value["nested"]["webAdminPassword"], "");
        assert_eq!(value["nested"]["items"][0]["webAcmeTencentSecretKey"], "");
        assert_eq!(value["nested"]["items"][1]["sessionName"], "嵌套公开名称");
    }
}
