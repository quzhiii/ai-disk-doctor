use std::fs;

use tempfile::tempdir;

#[test]
fn loads_example_rule_yaml() {
    let content = fs::read_to_string("rules/windows-system.yaml").expect("rule file should exist");
    assert!(content.contains("litesandbox-logs"));
}

#[test]
fn loads_playwright_glob_rule_yaml() {
    let content = fs::read_to_string("rules/playwright.yaml").expect("rule file should exist");
    assert!(content.contains(".playwright-browsers"));
}

#[test]
fn loads_sensitive_sample_rule_yaml() {
    let content = fs::read_to_string("rules/sensitive-samples.yaml").expect("rule file should exist");
    assert!(content.contains("Login Data"));
}

#[test]
fn example_glob_fixture_can_be_created() {
    let temp = tempdir().expect("tempdir should exist");
    let fixture = temp
        .path()
        .join("projects")
        .join("demo-app")
        .join(".playwright-browsers");

    fs::create_dir_all(&fixture).expect("fixture directory should be created");

    assert!(fixture.exists());
}
