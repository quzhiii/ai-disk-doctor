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
fn loads_expanded_ai_tooling_rule_yamls() {
    let ide = fs::read_to_string("rules/ai-ides.yaml").expect("ai ide rule should exist");
    let cli = fs::read_to_string("rules/ai-clis.yaml").expect("ai cli rule should exist");
    let cache = fs::read_to_string("rules/ai-caches.yaml").expect("ai cache rule should exist");
    let installers = fs::read_to_string("rules/ai-installers.yaml").expect("ai installer rule should exist");
    let installed_apps = fs::read_to_string("rules/ai-installed-apps.yaml").expect("ai installed app rule should exist");
    let test_artifacts = fs::read_to_string("rules/ai-test-artifacts.yaml").expect("ai test artifact rule should exist");

    assert!(ide.contains("Cursor"));
    assert!(ide.contains("Windsurf"));
    assert!(ide.contains("Trae"));
    assert!(cli.contains("aider"));
    assert!(cli.contains("Continue"));
    assert!(cache.contains("promptfoo"));
    assert!(cache.contains("evals"));
    assert!(installers.contains("Downloads"));
    assert!(installed_apps.contains("Programs"));
    assert!(installed_apps.contains("LM Studio"));
    assert!(test_artifacts.contains("playwright-report"));
    assert!(test_artifacts.contains("test-results"));
}

#[test]
fn loads_cross_platform_rule_paths() {
    let models = fs::read_to_string("rules/models.yaml").expect("models rule should exist");
    let huggingface = fs::read_to_string("rules/huggingface.yaml").expect("huggingface rule should exist");
    let docker = fs::read_to_string("rules/docker.yaml").expect("docker rule should exist");

    assert!(models.contains("~/.ollama"), "models should include unix ollama path");
    assert!(huggingface.contains("~/.cache/huggingface"), "huggingface should include unix path");
    assert!(docker.contains("~/.docker"), "docker should include unix path");
}

#[test]
fn loads_common_dev_artifact_rule_yaml() {
    let content = fs::read_to_string("rules/dev-artifacts.yaml")
        .expect("dev artifact rule should exist");

    for term in [
        "node_modules",
        "target",
        ".gradle",
        "__pycache__",
        "dist",
        ".next",
        ".turbo",
        "category: dev-artifact",
    ] {
        assert!(content.contains(term), "dev artifact rules should include {term}");
    }
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
