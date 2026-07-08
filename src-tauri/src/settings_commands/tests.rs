use super::*;
use crate::web_auth;

#[test]
fn public_settings_never_exposes_web_admin_password() {
    let settings = GlobalSettings {
        web_admin_username: "admin".to_string(),
        web_admin_password: "secret-password".to_string(),
        ..GlobalSettings::default()
    };

    let payload = public_settings(settings).expect("生成脱敏设置");

    assert_eq!(
        payload.get("webAdminPassword").and_then(Value::as_str),
        Some("")
    );
    assert_eq!(
        payload
            .get("webAdminPasswordConfigured")
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn public_settings_never_exposes_tencent_secret_key() {
    let settings = GlobalSettings {
        web_acme_tencent_secret_id: "AKIDTEST".to_string(),
        web_acme_tencent_secret_key: "secret-key".to_string(),
        ..GlobalSettings::default()
    };

    let payload = public_settings(settings).expect("生成脱敏设置");

    assert_eq!(
        payload
            .get("webAcmeTencentSecretKey")
            .and_then(Value::as_str),
        Some("")
    );
    assert_eq!(
        payload
            .get("webAcmeTencentSecretKeyConfigured")
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn save_settings_hashes_new_web_admin_password() {
    let current = GlobalSettings::default();
    let mut incoming = current.clone();
    incoming.web_admin_username = "admin".to_string();
    incoming.web_admin_password = "new-secret".to_string();

    let prepared = prepare_settings_for_save(&current, incoming).expect("准备保存设置");

    assert_ne!(prepared.web_admin_password, "new-secret");
    assert!(web_auth::is_web_admin_password_hash(
        &prepared.web_admin_password
    ));
    assert!(
        web_auth::verify_web_admin_password("new-secret", &prepared.web_admin_password)
            .expect("校验新密码哈希")
    );
}

#[test]
fn save_settings_blank_password_preserves_existing_hash() {
    let current = GlobalSettings {
        web_admin_username: "admin".to_string(),
        web_admin_password: web_auth::hash_web_admin_password("old-secret")
            .expect("生成旧密码哈希"),
        ..GlobalSettings::default()
    };
    let mut incoming = current.clone();
    incoming.web_admin_password.clear();

    let prepared = prepare_settings_for_save(&current, incoming).expect("准备保存空密码设置");

    assert_eq!(prepared.web_admin_password, current.web_admin_password);
}

#[test]
fn save_settings_blank_tencent_secret_key_preserves_existing_secret() {
    let current = GlobalSettings {
        web_acme_tencent_secret_id: "AKIDTEST".to_string(),
        web_acme_tencent_secret_key: "old-secret-key".to_string(),
        ..GlobalSettings::default()
    };
    let mut incoming = current.clone();
    incoming.web_acme_tencent_secret_key.clear();

    let prepared =
        prepare_settings_for_save(&current, incoming).expect("准备保存空 Secret Key 设置");

    assert_eq!(
        prepared.web_acme_tencent_secret_key,
        current.web_acme_tencent_secret_key
    );
}

#[test]
fn save_settings_empty_username_and_password_disables_web_auth() {
    let current = GlobalSettings {
        web_admin_username: "admin".to_string(),
        web_admin_password: web_auth::hash_web_admin_password("old-secret")
            .expect("生成旧密码哈希"),
        ..GlobalSettings::default()
    };
    let mut incoming = current.clone();
    incoming.web_admin_username.clear();
    incoming.web_admin_password.clear();

    let prepared =
        prepare_settings_for_save(&current, incoming).expect("准备保存禁用 Web 鉴权设置");

    assert!(prepared.web_admin_username.is_empty());
    assert!(prepared.web_admin_password.is_empty());
}
