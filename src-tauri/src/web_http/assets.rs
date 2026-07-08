use super::{HttpResponse, text_response};

struct WebAsset {
    path: &'static str,
    content_type: &'static str,
    content: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/web_assets.rs"));

pub(crate) fn embedded_asset_count() -> usize {
    WEB_ASSETS.len()
}

pub(crate) fn serve_asset(raw_path: &str, head_only: bool) -> HttpResponse {
    let Some(path) = normalize_asset_path(raw_path) else {
        return text_response(400, "非法资源路径");
    };

    let requested = if path.is_empty() {
        "index.html".to_string()
    } else {
        path
    };

    let asset = find_asset(&requested).or_else(|| {
        if requested.contains('.') {
            None
        } else {
            find_asset("index.html")
        }
    });

    match asset {
        Some(asset) => {
            let body = if head_only {
                Vec::new()
            } else {
                asset.content.to_vec()
            };
            let mut response = HttpResponse::new(200, "OK", body);
            response.header("Content-Type", asset.content_type);
            response.header(
                "Cache-Control",
                if asset.path == "index.html" {
                    "no-cache"
                } else {
                    "public, max-age=31536000, immutable"
                },
            );
            response
        }
        None if WEB_ASSETS.is_empty() => text_response(
            503,
            "Web 静态资源尚未嵌入，请先执行 npm run build 后重新启动应用。",
        ),
        None => text_response(404, "资源不存在"),
    }
}

fn find_asset(path: &str) -> Option<&'static WebAsset> {
    WEB_ASSETS.iter().find(|asset| asset.path == path)
}

fn normalize_asset_path(raw_path: &str) -> Option<String> {
    let trimmed = raw_path.trim_start_matches('/');
    let decoded = percent_decode(trimmed)?;
    if decoded
        .split('/')
        .any(|part| part == ".." || part.contains('\\') || part.contains(':'))
    {
        return None;
    }
    Some(decoded)
}

fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            let high = hex_value(bytes[index + 1])?;
            let low = hex_value(bytes[index + 2])?;
            output.push(high * 16 + low);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).ok()
}

fn hex_value(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_asset_path;

    #[test]
    fn 静态资源路径会拒绝目录穿越和_windows_路径片段() {
        assert_eq!(normalize_asset_path("/assets/%2e%2e/secret.js"), None);
        assert_eq!(normalize_asset_path("/assets\\secret.js"), None);
        assert_eq!(normalize_asset_path("/C:/secret.js"), None);
    }

    #[test]
    fn 静态资源路径会解码合法_percent_编码() {
        assert_eq!(
            normalize_asset_path("/assets/index%2Ejs").as_deref(),
            Some("assets/index.js")
        );
    }
}
