use crate::{
    app_state::AppRuntime,
    commands,
    sync_events::SyncEventBus,
    web_auth_state::WebAuthState,
    web_command_security,
    web_http::{
        HttpRequest, HttpResponse, StreamingBody, embedded_asset_count, json_response, serve_asset,
    },
    web_request_security, web_server_auth,
};
use serde::Deserialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager};

#[derive(Deserialize)]
struct InvokeRequest {
    command: String,
    #[serde(default)]
    args: Value,
}

pub(super) async fn route_request(
    app: AppHandle,
    runtime: AppRuntime,
    auth_state: WebAuthState,
    request: HttpRequest,
) -> HttpResponse {
    if let Err(response) = web_request_security::require_allowed_host(&runtime, &request) {
        return response;
    }

    if request.method == "OPTIONS" {
        return HttpResponse::empty(204, "No Content");
    }

    let request_path = request.path.split('?').next().unwrap_or("/");
    if request.method == "POST"
        && request_path.starts_with("/api/")
        && let Err(response) = web_request_security::require_same_origin_api_post(&request)
    {
        return response;
    }

    if request.method == "GET" && request_path == "/api/health" {
        return json_response(
            200,
            json!({
                "ok": true,
                "name": "ASA Server Manager Web",
                "assets": embedded_asset_count(),
            })
            .to_string()
            .into_bytes(),
        );
    }

    if request.method == "GET" && request_path == "/api/commands/security" {
        return json_response(
            200,
            json!({
                "ok": true,
                "data": web_command_security::web_command_policies(),
            })
            .to_string()
            .into_bytes(),
        );
    }

    if request.method == "GET" && request_path == "/api/auth/status" {
        return web_server_auth::handle_auth_status(runtime, &auth_state, &request);
    }

    if request.method == "GET" && request_path == "/api/auth/captcha" {
        return web_server_auth::handle_captcha(runtime, &auth_state, &request);
    }

    if request.method == "POST" && request_path == "/api/auth/login" {
        return web_server_auth::handle_login(runtime, auth_state, request).await;
    }

    if request.method == "POST" && request_path == "/api/auth/logout" {
        return web_server_auth::handle_logout(&auth_state, &request);
    }

    if request.method == "POST" && request_path == "/api/risk/confirm" {
        if let Err(response) = web_server_auth::require_auth(&runtime, &auth_state, &request) {
            return response;
        }
        return web_server_auth::handle_risk_confirmation(runtime, &auth_state, request);
    }

    if request.method == "POST" && request_path == "/api/invoke" {
        if let Err(response) = web_server_auth::require_auth(&runtime, &auth_state, &request) {
            return response;
        }
        return handle_invoke(app, runtime, auth_state, request).await;
    }

    if request.method == "GET" && request_path == "/api/events" {
        if let Err(response) = web_server_auth::require_auth(&runtime, &auth_state, &request) {
            return response;
        }
        return stream_events(app).await;
    }

    if request.method == "GET" || request.method == "HEAD" {
        return serve_asset(request_path, request.method == "HEAD");
    }

    json_response(
        405,
        json!({ "ok": false, "error": "不支持的 HTTP 方法" })
            .to_string()
            .into_bytes(),
    )
}

async fn stream_events(app: AppHandle) -> HttpResponse {
    let Some(bus) = app.try_state::<SyncEventBus>() else {
        return json_response(
            503,
            json!({ "ok": false, "error": "同步事件总线尚未初始化" })
                .to_string()
                .into_bytes(),
        );
    };

    let mut receiver = bus.subscribe();
    let (mut writer, body) = StreamingBody::new();

    tauri::async_runtime::spawn(async move {
        if writer
            .write_all(b": ASA Server Manager sync stream\n\n")
            .await
            .is_err()
        {
            return;
        }

        loop {
            let event = match receiver.recv().await {
                Ok(event) => event,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            };
            let data = match serde_json::to_string(&event.payload) {
                Ok(data) => data,
                Err(_) => continue,
            };
            let frame = format!(
                "id: {}\nevent: {}\ndata: {}\n\n",
                event.id,
                event.name,
                data.replace('\n', "\\n"),
            );
            if writer.write_all(frame.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    let mut response = HttpResponse::stream(200, "OK", body);
    response.header("Content-Type", "text/event-stream; charset=utf-8");
    response.header("Cache-Control", "no-cache");
    response.header("X-Accel-Buffering", "no");
    response
}

async fn handle_invoke(
    app: AppHandle,
    runtime: AppRuntime,
    auth_state: WebAuthState,
    request: HttpRequest,
) -> HttpResponse {
    let payload = match serde_json::from_slice::<InvokeRequest>(&request.body) {
        Ok(payload) => payload,
        Err(error) => {
            return json_response(
                400,
                json!({ "ok": false, "error": format!("Web API 请求 JSON 无效：{error}") })
                    .to_string()
                    .into_bytes(),
            );
        }
    };

    let risk = match web_command_security::web_command_policy(&payload.command) {
        Ok(risk) => risk,
        Err(error) => {
            return json_response(
                400,
                json!({ "ok": false, "error": error })
                    .to_string()
                    .into_bytes(),
            );
        }
    };
    let high_risk_confirmed = if risk == web_command_security::WebCommandRisk::High {
        match web_server_auth::validate_high_risk_confirmation(
            &runtime,
            &auth_state,
            &request,
            &payload.command,
            &payload.args,
        ) {
            Ok(()) => true,
            Err(response) => return response,
        }
    } else {
        false
    };

    match commands::handle_web_invoke(
        app,
        runtime,
        payload.command,
        payload.args,
        high_risk_confirmed,
    )
    .await
    {
        Ok(data) => json_response(
            200,
            json!({ "ok": true, "data": data }).to_string().into_bytes(),
        ),
        Err(error) => json_response(
            500,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        ),
    }
}
