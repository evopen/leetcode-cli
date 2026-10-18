use leetcode_cli::Config;

#[test]
fn clearance_configuration_and_environment() {
    let base = r#"
[code]
lang = 'rust'
[cookies]
csrf = 'csrf'
session = 'session'
site = 'leetcode.com'
[storage]
code = 'code'
root = '~/.leetcode'
scripts = 'scripts'
"#;
    for name in [
        "LEETCODE_CSRF",
        "LEETCODE_SESSION",
        "LEETCODE_CF_CLEARANCE",
        "LEETCODE_USER_AGENT",
    ] {
        unsafe { std::env::remove_var(name) };
    }
    let config: Config = base.parse().unwrap();
    assert!(config.cookies.cf_clearance.is_empty());
    assert!(config.cookies.user_agent.is_empty());
    assert_eq!(
        config.cookies.to_string(),
        "LEETCODE_SESSION=session;csrftoken=csrf;"
    );

    let configured = base.replace(
        "site = 'leetcode.com'",
        "site = 'leetcode.com'\ncf_clearance = 'configured'\nuser_agent = 'browser'",
    );
    let config: Config = configured.parse().unwrap();
    assert_eq!(config.cookies.user_agent, "browser");
    assert_eq!(
        config.cookies.to_string(),
        "LEETCODE_SESSION=session;csrftoken=csrf;cf_clearance=configured;"
    );
    unsafe {
        std::env::set_var("LEETCODE_CF_CLEARANCE", "override");
        std::env::set_var("LEETCODE_USER_AGENT", "override-browser");
    }
    let config: Config = configured.parse().unwrap();
    assert_eq!(config.cookies.cf_clearance, "override");
    assert_eq!(config.cookies.user_agent, "override-browser");
}
