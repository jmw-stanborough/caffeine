use caffeine_lib::server_monitor::remote::validation::{
    resolve_port, sanitize_host, sanitize_username,
};

#[test]
fn test_sanitize_host_valid() {
    assert_eq!(sanitize_host("192.168.1.1").unwrap(), "192.168.1.1");
    assert_eq!(sanitize_host("server.local").unwrap(), "server.local");
    assert_eq!(sanitize_host("   node-01   ").unwrap(), "node-01");
}

#[test]
fn test_sanitize_host_rejects_injections() {
    assert!(sanitize_host("").is_err());
    assert!(sanitize_host("server; rm -rf").is_err());
    assert!(sanitize_host("host&calc").is_err());
    assert!(sanitize_host("host | ls").is_err());
    assert!(sanitize_host("host`id`").is_err());
}

#[test]
fn test_sanitize_username_valid() {
    assert_eq!(sanitize_username(None).unwrap(), None);
    assert_eq!(sanitize_username(Some("")).unwrap(), None);
    assert_eq!(sanitize_username(Some("root")).unwrap(), Some("root".to_string()));
    assert_eq!(sanitize_username(Some(" admin ")).unwrap(), Some("admin".to_string()));
}

#[test]
fn test_sanitize_username_rejects_injections() {
    assert!(sanitize_username(Some("root;whoami")).is_err());
    assert!(sanitize_username(Some("user name")).is_err());
}

#[test]
fn test_resolve_port() {
    assert_eq!(resolve_port(None, 22).unwrap(), 22);
    assert_eq!(resolve_port(Some(2222), 22).unwrap(), 2222);
    assert!(resolve_port(Some(0), 22).is_err());
}
