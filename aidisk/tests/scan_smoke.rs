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
fn loads_expanded_platform_rule_yamls() {
    let wsl = fs::read_to_string("rules/wsl.yaml").expect("wsl rule should exist");
    let docker_build = fs::read_to_string("rules/docker-build-cache.yaml").expect("docker build cache rule should exist");
    let docker_volume = fs::read_to_string("rules/docker-volumes.yaml").expect("docker volumes rule should exist");
    let models = fs::read_to_string("rules/models.yaml").expect("models rule should exist");
    let huggingface = fs::read_to_string("rules/huggingface.yaml").expect("huggingface rule should exist");

    assert!(wsl.contains("ext4.vhdx"));
    assert!(docker_build.contains("build-cache"));
    assert!(docker_volume.contains("docker_data.vhdx"));
    assert!(models.contains(".ollama"));
    assert!(huggingface.contains("huggingface"));
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
