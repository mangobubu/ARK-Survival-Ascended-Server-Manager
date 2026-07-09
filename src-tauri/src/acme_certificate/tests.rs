use super::*;
use crate::{
    acme_crypto::{base64_url, dns01_txt_value, jwk_thumbprint},
    acme_persistence::{AcmeCertificateStatus, STATUS_FILE_NAME},
    tencent_dns::TencentDnsRecord,
};
use rand::rngs::OsRng;
use rsa::RsaPrivateKey;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread::JoinHandle,
    time::Instant,
};

#[derive(Default)]
struct MockAcmeDnsProvider {
    created: Mutex<Vec<TencentDnsRecord>>,
    deleted: Mutex<Vec<(String, u64)>>,
}

impl AcmeDnsProvider for MockAcmeDnsProvider {
    fn create_txt_record(
        &self,
        domain: &str,
        sub_domain: &str,
        value: &str,
        _ttl: u32,
    ) -> Result<TencentDnsRecord, String> {
        let mut created = self.created.lock().expect("读取 mock DNS 创建记录");
        let record = TencentDnsRecord::created(
            created.len() as u64 + 1,
            domain,
            sub_domain,
            value,
            600,
            "默认",
        );
        created.push(record.clone());
        Ok(record)
    }

    fn cleanup_txt_record(&self, record: &TencentDnsRecord) -> Result<(), String> {
        self.deleted
            .lock()
            .expect("读取 mock DNS 删除记录")
            .push((record.domain.clone(), record.id));
        Ok(())
    }
}

#[derive(Default)]
struct MockAcmeState {
    challenge_accepted: bool,
    finalized: bool,
    done: bool,
    nonce_counter: usize,
    requests: Vec<String>,
}

struct MockAcmeServer {
    base_url: String,
    state: Arc<Mutex<MockAcmeState>>,
    handle: Option<JoinHandle<()>>,
}

impl MockAcmeServer {
    fn spawn() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("绑定 mock ACME 服务端");
        listener
            .set_nonblocking(true)
            .expect("设置 mock ACME 非阻塞监听");
        let address = listener.local_addr().expect("读取 mock ACME 地址");
        let base_url = format!("http://{address}");
        let state = Arc::new(Mutex::new(MockAcmeState::default()));
        let thread_state = Arc::clone(&state);
        let thread_base_url = base_url.clone();
        let handle = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(60);
            loop {
                if thread_state.lock().expect("读取 mock ACME 状态").done {
                    break;
                }
                if Instant::now() > deadline {
                    break;
                }
                match listener.accept() {
                    Ok((stream, _)) => {
                        handle_mock_acme_stream(stream, &thread_base_url, &thread_state);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            base_url,
            state,
            handle: Some(handle),
        }
    }

    fn directory_url(&self) -> String {
        format!("{}/directory", self.base_url)
    }

    fn requests(&self) -> Vec<String> {
        self.state
            .lock()
            .expect("读取 mock ACME 请求记录")
            .requests
            .clone()
    }

    fn finish(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("mock ACME 服务端线程退出");
        }
    }
}

fn handle_mock_acme_stream(
    mut stream: TcpStream,
    base_url: &str,
    state: &Arc<Mutex<MockAcmeState>>,
) {
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut buffer = Vec::new();
    let mut temp = [0_u8; 4096];
    let (method, path) = loop {
        let read = stream.read(&mut temp).expect("读取 mock ACME 请求");
        if read == 0 {
            return;
        }
        buffer.extend_from_slice(&temp[..read]);
        let Some(header_end) = http_header_end(&buffer) else {
            continue;
        };
        let header = String::from_utf8_lossy(&buffer[..header_end]);
        let content_length = content_length_from_header(&header);
        if buffer.len() < header_end + 4 + content_length {
            continue;
        }
        let request_line = header.lines().next().unwrap_or_default();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or_default().to_string();
        let path = parts.next().unwrap_or_default().to_string();
        break (method, path);
    };

    let mut state = state.lock().expect("写入 mock ACME 状态");
    state.requests.push(format!("{method} {path}"));
    state.nonce_counter += 1;
    let nonce = format!("mock-nonce-{}", state.nonce_counter);

    let response = match (method.as_str(), path.as_str()) {
        ("GET", "/directory") => MockAcmeResponse::json(
            200,
            json!({
                "newNonce": format!("{base_url}/new-nonce"),
                "newAccount": format!("{base_url}/new-account"),
                "newOrder": format!("{base_url}/new-order")
            }),
        ),
        ("HEAD", "/new-nonce") => MockAcmeResponse::empty(200),
        ("POST", "/new-account") => MockAcmeResponse::json_with_location(
            201,
            json!({ "status": "valid" }),
            format!("{base_url}/account/1"),
        ),
        ("POST", "/new-order") => MockAcmeResponse::json_with_location(
            201,
            json!({
                "status": "pending",
                "authorizations": [format!("{base_url}/authz/1")],
                "finalize": format!("{base_url}/finalize/1")
            }),
            format!("{base_url}/order/1"),
        ),
        ("POST", "/authz/1") => {
            let status = if state.challenge_accepted {
                "valid"
            } else {
                "pending"
            };
            MockAcmeResponse::json(
                200,
                json!({
                    "status": status,
                    "challenges": [{
                        "type": "dns-01",
                        "url": format!("{base_url}/challenge/1"),
                        "token": "mock-dns-token",
                        "status": status
                    }]
                }),
            )
        }
        ("POST", "/challenge/1") => {
            state.challenge_accepted = true;
            MockAcmeResponse::json(200, json!({ "status": "processing" }))
        }
        ("POST", "/finalize/1") => {
            state.finalized = true;
            MockAcmeResponse::json(
                200,
                json!({
                    "status": "processing",
                    "authorizations": [format!("{base_url}/authz/1")],
                    "finalize": format!("{base_url}/finalize/1")
                }),
            )
        }
        ("POST", "/order/1") if state.finalized => MockAcmeResponse::json(
            200,
            json!({
                "status": "valid",
                "authorizations": [format!("{base_url}/authz/1")],
                "finalize": format!("{base_url}/finalize/1"),
                "certificate": format!("{base_url}/cert/1")
            }),
        ),
        ("POST", "/cert/1") => {
            state.done = true;
            MockAcmeResponse::pem(
                200,
                "-----BEGIN CERTIFICATE-----\nTEST\n-----END CERTIFICATE-----\n",
            )
        }
        _ => MockAcmeResponse::json(404, json!({ "error": "not found" })),
    };
    drop(state);

    write_mock_response(&mut stream, response, &nonce, method == "HEAD");
}

struct MockAcmeResponse {
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
    location: Option<String>,
}

impl MockAcmeResponse {
    fn empty(status: u16) -> Self {
        Self {
            status,
            content_type: "application/json",
            body: Vec::new(),
            location: None,
        }
    }

    fn json(status: u16, body: Value) -> Self {
        Self {
            status,
            content_type: "application/json",
            body: body.to_string().into_bytes(),
            location: None,
        }
    }

    fn json_with_location(status: u16, body: Value, location: String) -> Self {
        Self {
            status,
            content_type: "application/json",
            body: body.to_string().into_bytes(),
            location: Some(location),
        }
    }

    fn pem(status: u16, body: &str) -> Self {
        Self {
            status,
            content_type: "application/pem-certificate-chain",
            body: body.as_bytes().to_vec(),
            location: None,
        }
    }
}

fn write_mock_response(
    stream: &mut TcpStream,
    response: MockAcmeResponse,
    nonce: &str,
    head_only: bool,
) {
    let reason = match response.status {
        200 => "OK",
        201 => "Created",
        404 => "Not Found",
        _ => "OK",
    };
    let mut head = format!(
        "HTTP/1.1 {} {reason}\r\nReplay-Nonce: {nonce}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status,
        response.content_type,
        if head_only { 0 } else { response.body.len() }
    );
    if let Some(location) = response.location {
        head.push_str(&format!("Location: {location}\r\n"));
    }
    head.push_str("\r\n");
    stream
        .write_all(head.as_bytes())
        .expect("写入 mock ACME 响应头");
    if !head_only {
        stream
            .write_all(&response.body)
            .expect("写入 mock ACME 响应体");
    }
}

fn http_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn content_length_from_header(header: &str) -> usize {
    header
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.trim().eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}

#[test]
fn 证书路径使用_rsa2048_后缀() {
    let paths = certificate_paths(Path::new("certs"), "ark.example.com");
    assert!(
        paths
            .private_key_pem
            .to_string_lossy()
            .contains("RSA2048.key.pem")
    );
    assert!(
        paths
            .fullchain_pem
            .to_string_lossy()
            .ends_with(".fullchain.pem")
    );
}

#[test]
fn dns01_txt_value_使用_key_authorization_sha256() {
    let value = dns01_txt_value("token", "thumbprint");
    assert_eq!(value, base64_url(&Sha256::digest(b"token.thumbprint")));
}

#[test]
fn rsa_jwk_thumbprint_使用稳定字段顺序() {
    let jwk = json!({
        "kty": "RSA",
        "n": "modulus",
        "e": "AQAB",
    });
    let thumbprint = jwk_thumbprint(&jwk).expect("计算 thumbprint");
    assert_eq!(
        thumbprint,
        base64_url(&Sha256::digest(
            br#"{"e":"AQAB","kty":"RSA","n":"modulus"}"#
        ))
    );
}

#[test]
fn csr_der_包含域名_san_和_rsa2048_公钥() {
    let mut rng = OsRng;
    let key = RsaPrivateKey::new(&mut rng, 2048).expect("生成测试私钥");
    let csr = build_csr_der("ark.example.com", &key).expect("生成 CSR");
    assert_eq!(csr.first().copied(), Some(0x30));
    assert!(
        csr.windows("ark.example.com".len())
            .any(|window| window == b"ark.example.com")
    );
    assert!(csr.len() > 600);
}

#[test]
fn 证书状态到达续期时间会触发续期() {
    let temp = tempfile::tempdir().expect("创建临时目录");
    let paths = certificate_paths(temp.path(), "ark.example.com");
    let now = unix_timestamp().expect("当前时间");
    let status = AcmeCertificateStatus {
        domain: "ark.example.com".to_string(),
        directory_url: LETS_ENCRYPT_DIRECTORY_URL.to_string(),
        key_algorithm: ACME_KEY_ALGORITHM.to_string(),
        dns_provider: "tencent-cloud-dnspod".to_string(),
        issued_at_unix: now.saturating_sub(60),
        renew_after_unix: now.saturating_sub(1),
        expires_at_unix: now + CERTIFICATE_RENEW_BEFORE_SECONDS,
        fullchain_pem: paths.fullchain_pem,
        private_key_pem: paths.private_key_pem,
    };
    let content = serde_json::to_vec_pretty(&status).expect("序列化状态");
    fs::write(temp.path().join(STATUS_FILE_NAME), content).expect("写入状态");

    assert!(certificate_renewal_due(temp.path(), "ark.example.com").expect("判断续期"));
}

#[test]
fn 缺少证书状态时自动签发应触发续期重建状态() {
    let temp = tempfile::tempdir().expect("创建临时目录");

    assert!(certificate_renewal_due(temp.path(), "ark.example.com").expect("判断续期"));
}

#[test]
fn mock_acme_dns01_完整签发流程会创建并清理腾讯云_txt_记录() {
    let mut server = MockAcmeServer::spawn();
    let temp = tempfile::tempdir().expect("创建临时证书目录");
    let domain = "ark.example.com";
    let paths = certificate_paths(temp.path(), domain);
    let settings = GlobalSettings {
        web_https_enabled: true,
        web_acme_auto_issue_enabled: true,
        web_acme_directory_url: server.directory_url(),
        web_acme_account_email: "admin@example.com".to_string(),
        web_acme_tencent_secret_id: "AKIDMOCK".to_string(),
        web_acme_tencent_secret_key: "SECRETMOCK".to_string(),
        ..GlobalSettings::default()
    };
    let dns_provider = MockAcmeDnsProvider::default();

    write_pending_acme_manifest(temp.path(), domain, &settings, &paths).expect("写入待签发清单");
    issue_certificate_with_dns_provider(
        temp.path(),
        domain,
        &settings,
        &paths,
        &dns_provider,
        Duration::ZERO,
        None,
    )
    .expect("mock ACME 签发成功");
    server.finish();

    let fullchain = fs::read_to_string(&paths.fullchain_pem).expect("读取证书链");
    assert!(fullchain.contains("-----BEGIN CERTIFICATE-----"));
    assert!(paths.private_key_pem.is_file());
    assert!(!temp.path().join(PENDING_MANIFEST_FILE_NAME).exists());

    let status = read_certificate_status(temp.path())
        .expect("读取证书状态")
        .expect("存在证书状态");
    assert_eq!(status.domain, domain);
    assert_eq!(status.key_algorithm, ACME_KEY_ALGORITHM);
    assert_eq!(status.directory_url, settings.web_acme_directory_url);

    let created = dns_provider.created.lock().expect("读取创建记录");
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].domain, "example.com");
    assert_eq!(created[0].sub_domain, "_acme-challenge.ark");
    assert!(!created[0].value.trim().is_empty());
    drop(created);

    let deleted = dns_provider.deleted.lock().expect("读取删除记录");
    assert_eq!(deleted.as_slice(), &[("example.com".to_string(), 1)]);
    drop(deleted);

    let requests = server.requests();
    for expected in [
        "GET /directory",
        "HEAD /new-nonce",
        "POST /new-account",
        "POST /new-order",
        "POST /authz/1",
        "POST /challenge/1",
        "POST /finalize/1",
        "POST /order/1",
        "POST /cert/1",
    ] {
        assert!(
            requests.iter().any(|request| request == expected),
            "缺少请求 {expected}"
        );
    }
}
