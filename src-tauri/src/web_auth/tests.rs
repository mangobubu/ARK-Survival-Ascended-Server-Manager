use super::*;

#[test]
fn hashes_password_with_salt_and_verifies_it() {
    let hash = hash_web_admin_password("secret").expect("生成密码哈希");

    assert_ne!(hash, "secret");
    assert!(is_web_admin_password_hash(&hash));
    assert!(verify_web_admin_password("secret", &hash).expect("校验正确密码"));
    assert!(!verify_web_admin_password("wrong", &hash).expect("校验错误密码"));
}

#[test]
fn generated_hashes_use_random_salt() {
    let first = hash_web_admin_password("secret").expect("生成第一个哈希");
    let second = hash_web_admin_password("secret").expect("生成第二个哈希");

    assert_ne!(first, second);
}

#[test]
fn migrates_legacy_plaintext_once() {
    let mut stored = "legacy-secret".to_string();

    assert!(migrate_web_admin_password_hash(&mut stored).expect("迁移明文密码"));
    assert_ne!(stored, "legacy-secret");
    assert!(verify_web_admin_password("legacy-secret", &stored).expect("校验迁移后密码"));

    let migrated = stored.clone();
    assert!(!migrate_web_admin_password_hash(&mut stored).expect("已迁移密码不重复迁移"));
    assert_eq!(stored, migrated);
}
