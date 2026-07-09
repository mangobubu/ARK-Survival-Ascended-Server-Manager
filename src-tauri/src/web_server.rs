mod routing;

use crate::{
    app_state::AppRuntime,
    command_events::emit_instance_log,
    models::GlobalSettings,
    web_auth_state::WebAuthState,
    web_http::{json_response, read_request, write_response},
};
use serde_json::json;
use std::{
    net::{SocketAddr, TcpListener as StdTcpListener},
    sync::Mutex,
};
use tauri::{AppHandle, Manager};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::oneshot,
};

#[derive(Default)]
pub struct WebServerManager {
    active: Mutex<Option<WebServerHandle>>,
}

struct WebServerHandle {
    port: u16,
    shutdown: oneshot::Sender<()>,
}

impl WebServerManager {
    pub fn apply_settings(
        &self,
        app: AppHandle,
        runtime: AppRuntime,
        settings: &GlobalSettings,
    ) -> Result<(), String> {
        if !settings.web_management_enabled {
            self.stop_current_best_effort(&app, Some(&runtime));
            return Ok(());
        }

        let desired_port = settings.web_server_port;
        let current_port = self
            .active
            .lock()
            .map_err(|_| "Web 管理服务状态锁已损坏".to_string())?
            .as_ref()
            .map(|active| active.port);

        if current_port == Some(desired_port) {
            return Ok(());
        }

        self.stop_current_best_effort(&app, Some(&runtime));
        let handle = spawn_web_server(app, runtime, desired_port)?;
        let mut active = self
            .active
            .lock()
            .map_err(|_| "Web 管理服务状态锁已损坏".to_string())?;
        *active = Some(handle);
        Ok(())
    }

    pub fn shutdown(&self, app: &AppHandle, runtime: Option<&AppRuntime>) {
        self.stop_current_best_effort(app, runtime);
    }

    fn stop_current_best_effort(&self, app: &AppHandle, runtime: Option<&AppRuntime>) {
        let current = self.active.lock().ok().and_then(|mut active| active.take());
        if let Some(handle) = current {
            let port = handle.port;
            let _ = handle.shutdown.send(());
            if let Some(runtime) = runtime {
                let _ = emit_instance_log(
                    app,
                    runtime,
                    "Web服务",
                    "info",
                    &format!("Web 管理已关闭，端口 {port} 停止监听"),
                );
            }
        }
    }
}

pub fn apply_settings_from_app(app: &AppHandle, settings: &GlobalSettings) -> Result<(), String> {
    let runtime = app
        .try_state::<AppRuntime>()
        .ok_or_else(|| "应用运行状态尚未初始化，无法应用 Web 管理服务设置".to_string())?;
    let manager = app
        .try_state::<WebServerManager>()
        .ok_or_else(|| "Web 管理服务状态尚未初始化".to_string())?;
    manager.apply_settings(app.clone(), runtime.inner().clone(), settings)
}

pub fn shutdown(app: &AppHandle) {
    let runtime = app.try_state::<AppRuntime>();
    if let Some(manager) = app.try_state::<WebServerManager>() {
        manager.shutdown(app, runtime.as_deref());
    }
}

fn spawn_web_server(
    app: AppHandle,
    runtime: AppRuntime,
    port: u16,
) -> Result<WebServerHandle, String> {
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = StdTcpListener::bind(address)
        .map_err(|error| format!("Web 管理启动失败，端口 {port} 不可用：{error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("Web 管理启动失败，无法切换端口 {port} 为非阻塞模式：{error}"))?;
    let (shutdown, mut shutdown_receiver) = oneshot::channel();

    tauri::async_runtime::spawn(async move {
        let listener = match TcpListener::from_std(listener) {
            Ok(listener) => listener,
            Err(error) => {
                let _ = emit_instance_log(
                    &app,
                    &runtime,
                    "Web服务",
                    "error",
                    &format!("Web 管理启动失败，无法接管端口 {port}：{error}"),
                );
                return;
            }
        };
        let auth_state = WebAuthState::default();
        let _ = emit_instance_log(
            &app,
            &runtime,
            "Web服务",
            "success",
            &format!("Web 管理已启动：http://127.0.0.1:{port}"),
        );

        loop {
            tokio::select! {
                _ = &mut shutdown_receiver => {
                    break;
                }
                accepted = listener.accept() => {
                    match accepted {
                        Ok((stream, peer_addr)) => {
                            let app = app.clone();
                            let runtime = runtime.clone();
                            let auth_state = auth_state.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = handle_connection(app, runtime, auth_state, stream, peer_addr).await;
                            });
                        }
                        Err(error) => {
                            let _ = emit_instance_log(
                                &app,
                                &runtime,
                                "Web服务",
                                "warn",
                                &format!("接受 Web 连接失败：{error}"),
                            );
                        }
                    }
                }
            }
        }
    });

    Ok(WebServerHandle { port, shutdown })
}

async fn handle_connection(
    app: AppHandle,
    runtime: AppRuntime,
    auth_state: WebAuthState,
    mut stream: TcpStream,
    client_addr: SocketAddr,
) -> Result<(), String> {
    let response = match read_request(&mut stream, client_addr).await {
        Ok(request) => routing::route_request(app, runtime, auth_state, request).await,
        Err(error) => json_response(
            400,
            json!({ "ok": false, "error": error })
                .to_string()
                .into_bytes(),
        ),
    };

    write_response(&mut stream, response).await
}
