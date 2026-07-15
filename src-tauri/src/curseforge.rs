use crate::app_state::AppRuntime;
use reqwest::{Client, StatusCode, Url, header::ACCEPT};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::State;

const CURSEFORGE_API_BASE_URL: &str = "https://api.curseforge.com/v1/mods/search";
const ASA_GAME_ID: &str = "83374";
const MAX_QUERY_CHARS: usize = 100;
const MAX_PAGE_SIZE: u32 = 50;
const MAX_RESULT_WINDOW: u32 = 10_000;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeModSearchResult {
    pub items: Vec<CurseForgeModSummary>,
    pub total_count: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeModSummary {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub author: String,
    pub version: String,
    pub size: String,
    pub download_count: u64,
    pub date_modified: String,
    pub thumbnail_url: Option<String>,
    pub website_url: String,
}

#[derive(Debug, Deserialize)]
struct ApiSearchResponse {
    data: Vec<ApiMod>,
    pagination: ApiPagination,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiMod {
    id: u64,
    name: String,
    slug: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    authors: Vec<ApiAuthor>,
    #[serde(default)]
    latest_files: Vec<ApiFile>,
    #[serde(default)]
    download_count: u64,
    #[serde(default)]
    date_modified: String,
    logo: Option<ApiLogo>,
}

#[derive(Debug, Deserialize)]
struct ApiAuthor {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiFile {
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    file_length: u64,
    #[serde(default)]
    file_date: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiLogo {
    thumbnail_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiPagination {
    total_count: u32,
}

pub(crate) async fn search_curseforge_mods_for_runtime(
    runtime: &AppRuntime,
    query: String,
    index: u32,
    page_size: u32,
) -> Result<CurseForgeModSearchResult, String> {
    validate_search(&query, index, page_size)?;
    let settings = runtime.settings()?;
    let api_key = configured_api_key(&settings.curseforge_api_key)?;
    let url = build_search_url(&query, index, page_size)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| format!("无法初始化 CurseForge API 客户端：{error}"))?;
    let response = client
        .get(url)
        .header(ACCEPT, "application/json")
        .header("x-api-key", api_key)
        .send()
        .await
        .map_err(|error| format!("连接 CurseForge 官方 API 失败：{error}"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| format!("读取 CurseForge 官方 API 响应失败：{error}"))?;

    if !status.is_success() {
        return Err(api_error(status, &body));
    }

    parse_search_response(&body)
}

#[tauri::command]
pub async fn search_curseforge_mods(
    runtime: State<'_, AppRuntime>,
    query: String,
    index: u32,
    page_size: u32,
) -> Result<CurseForgeModSearchResult, String> {
    search_curseforge_mods_for_runtime(runtime.inner(), query, index, page_size).await
}

fn validate_search(query: &str, index: u32, page_size: u32) -> Result<(), String> {
    if query.chars().count() > MAX_QUERY_CHARS {
        return Err(format!("MOD 搜索词不能超过 {MAX_QUERY_CHARS} 个字符"));
    }
    if !(1..=MAX_PAGE_SIZE).contains(&page_size) {
        return Err(format!("CurseForge 每页数量必须在 1-{MAX_PAGE_SIZE} 之间"));
    }
    if index.saturating_add(page_size) > MAX_RESULT_WINDOW {
        return Err(format!(
            "CurseForge 仅允许查询前 {MAX_RESULT_WINDOW} 条结果"
        ));
    }
    Ok(())
}

fn configured_api_key(stored_key: &str) -> Result<String, String> {
    let stored_key = stored_key.trim();
    if !stored_key.is_empty() {
        return Ok(stored_key.to_string());
    }
    if let Ok(environment_key) = std::env::var("CURSEFORGE_API_KEY")
        && !environment_key.trim().is_empty()
    {
        return Ok(environment_key.trim().to_string());
    }
    Err("尚未配置 CurseForge API Key，请先在全局设置的 CurseForge 接入中填写".to_string())
}

fn build_search_url(query: &str, index: u32, page_size: u32) -> Result<Url, String> {
    let mut url = Url::parse(CURSEFORGE_API_BASE_URL)
        .map_err(|error| format!("CurseForge API 地址无效：{error}"))?;
    {
        let mut params = url.query_pairs_mut();
        params
            .append_pair("gameId", ASA_GAME_ID)
            .append_pair("index", &index.to_string())
            .append_pair("pageSize", &page_size.to_string())
            .append_pair("sortField", "2")
            .append_pair("sortOrder", "desc");
        let query = query.trim();
        if !query.is_empty() {
            params.append_pair("searchFilter", query);
        }
    }
    Ok(url)
}

fn parse_search_response(body: &str) -> Result<CurseForgeModSearchResult, String> {
    let response: ApiSearchResponse = serde_json::from_str(body)
        .map_err(|error| format!("解析 CurseForge 官方 MOD 数据失败：{error}"))?;
    Ok(CurseForgeModSearchResult {
        items: response.data.into_iter().map(mod_summary).collect(),
        total_count: response.pagination.total_count,
    })
}

fn mod_summary(item: ApiMod) -> CurseForgeModSummary {
    let latest_file = item
        .latest_files
        .iter()
        .max_by(|left, right| left.file_date.cmp(&right.file_date));
    let version = latest_file
        .map(|file| {
            let display_name = file.display_name.trim();
            if display_name.is_empty() {
                file.file_name.trim()
            } else {
                display_name
            }
        })
        .filter(|value| !value.is_empty())
        .unwrap_or("暂无发布文件")
        .to_string();
    let size = latest_file
        .map(|file| format_file_size(file.file_length))
        .unwrap_or_else(|| "—".to_string());
    let author = item
        .authors
        .first()
        .map(|author| author.name.trim())
        .filter(|name| !name.is_empty())
        .unwrap_or("未知作者")
        .to_string();

    CurseForgeModSummary {
        id: item.id.to_string(),
        name: item.name,
        summary: item.summary,
        author,
        version,
        size,
        download_count: item.download_count,
        date_modified: item.date_modified,
        thumbnail_url: item.logo.and_then(|logo| logo.thumbnail_url),
        website_url: format!(
            "https://www.curseforge.com/ark-survival-ascended/mods/{}",
            item.slug
        ),
    }
}

fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let bytes = bytes as f64;
    if bytes >= GB {
        format!("{:.2} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{} B", bytes as u64)
    }
}

fn api_error(status: StatusCode, body: &str) -> String {
    if matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN) {
        return "CurseForge API Key 无效或无权访问 ARK: Survival Ascended MOD 数据".to_string();
    }
    let detail = body.trim().chars().take(200).collect::<String>();
    if detail.is_empty() {
        format!("CurseForge 官方 API 请求失败（HTTP {status}）")
    } else {
        format!("CurseForge 官方 API 请求失败（HTTP {status}）：{detail}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 搜索地址固定为_asa_并正确编码搜索词() {
        let url = build_search_url("Awesome Spyglass!", 20, 20).expect("生成 URL");
        let params = url
            .query_pairs()
            .collect::<std::collections::HashMap<_, _>>();
        assert_eq!(
            params.get("gameId").map(|value| value.as_ref()),
            Some(ASA_GAME_ID)
        );
        assert_eq!(
            params.get("searchFilter").map(|value| value.as_ref()),
            Some("Awesome Spyglass!")
        );
        assert_eq!(params.get("index").map(|value| value.as_ref()), Some("20"));
        assert_eq!(
            params.get("pageSize").map(|value| value.as_ref()),
            Some("20")
        );
    }

    #[test]
    fn 官方响应转换为_dialog_所需字段() {
        let result = parse_search_response(
            r#"{
              "data": [{
                "id": 928708,
                "name": "Awesome Spyglass!",
                "slug": "awesomespyglass",
                "summary": "Shows advanced creature information.",
                "authors": [{"name": "ChrisMods"}],
                "latestFiles": [
                  {"displayName": "Old", "fileName": "old.zip", "fileLength": 1024, "fileDate": "2025-01-01T00:00:00Z"},
                  {"displayName": "Release 2.0", "fileName": "new.zip", "fileLength": 1572864, "fileDate": "2026-07-01T00:00:00Z"}
                ],
                "downloadCount": 9200000,
                "dateModified": "2026-07-01T00:00:00Z",
                "logo": {"thumbnailUrl": "https://media.example/mod.png"}
              }],
              "pagination": {"totalCount": 1}
            }"#,
        )
        .expect("解析官方响应");

        assert_eq!(result.total_count, 1);
        let item = &result.items[0];
        assert_eq!(item.id, "928708");
        assert_eq!(item.author, "ChrisMods");
        assert_eq!(item.version, "Release 2.0");
        assert_eq!(item.size, "1.5 MB");
        assert!(item.website_url.ends_with("/awesomespyglass"));
    }

    #[test]
    fn 拒绝超出官方限制的分页() {
        assert!(validate_search("", 0, 0).is_err());
        assert!(validate_search("", 9_980, 50).is_err());
        assert!(validate_search("", 9_950, 50).is_ok());
    }
}
