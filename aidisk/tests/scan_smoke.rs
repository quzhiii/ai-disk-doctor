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
    let agents = fs::read_to_string("rules/ai-agents.yaml").expect("ai agents rule should exist");
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
    assert!(agents.contains("~/.claude"));
    assert!(agents.contains("platforms: [windows, linux, macos]"));

    // Verify multi-platform format is present in updated ai-* rules
    assert!(ide.contains("platforms: [windows, linux, macos]"));
    assert!(cli.contains("platforms: [windows, linux, macos]"));
    assert!(cache.contains("platforms: [windows, linux, macos]"));
    assert!(installers.contains("platforms: [windows, linux, macos]"));
    assert!(installed_apps.contains("platforms: [windows, linux, macos]"));

    // Verify linux/macos path sections exist
    assert!(ide.contains("~/.config/Cursor"), "ai-ides should include linux Cursor path");
    assert!(ide.contains("~/Library/Application Support/Cursor"), "ai-ides should include macos Cursor path");
    assert!(cli.contains("~/.aider*"), "ai-clis should include unix aider path");
    assert!(cli.contains("~/.config/opencode"), "ai-clis should include unix opencode path");
    assert!(cache.contains("~/.cache/transformers"), "ai-caches should include unix transformers cache");
    assert!(cache.contains("~/Library/Caches/promptfoo"), "ai-caches should include macos promptfoo cache");
    assert!(installers.contains("AppImage"), "ai-installers should include linux AppImage patterns");
    assert!(installers.contains(".dmg"), "ai-installers should include macos dmg patterns");
    assert!(installed_apps.contains("/Applications/"), "ai-installed-apps should include macos /Applications paths");
    assert!(installed_apps.contains("/opt/"), "ai-installed-apps should include linux /opt paths");
}

#[test]
fn loads_cross_platform_rule_paths() {
    let models = fs::read_to_string("rules/models.yaml").expect("models rule should exist");
    let huggingface = fs::read_to_string("rules/huggingface.yaml").expect("huggingface rule should exist");
    let docker = fs::read_to_string("rules/docker.yaml").expect("docker rule should exist");

    assert!(models.contains("~/.ollama"), "models should include unix ollama path");
    assert!(huggingface.contains("~/.cache/huggingface"), "huggingface should include unix path");
    assert!(docker.contains("~/.docker"), "docker should include unix path");

    // Verify ai-* rules include cross-platform paths
    let agents = fs::read_to_string("rules/ai-agents.yaml").expect("ai agents rule should exist");
    let ide = fs::read_to_string("rules/ai-ides.yaml").expect("ai ide rule should exist");
    let cli = fs::read_to_string("rules/ai-clis.yaml").expect("ai cli rule should exist");
    let cache = fs::read_to_string("rules/ai-caches.yaml").expect("ai cache rule should exist");
    assert!(agents.contains("~/.claude"), "ai-agents should include linux claude path");
    assert!(agents.contains("~/Library/Application Support/Claude"), "ai-agents should include macos claude path");
    assert!(agents.contains("~/.codex"), "ai-agents should include linux codex path");
    assert!(ide.contains("~/.config/Cursor"), "ai-ides should include linux Cursor path");
    assert!(ide.contains("~/Library/Application Support/Cursor"), "ai-ides should include macos Cursor path");
    assert!(cli.contains("~/.aider*"), "ai-clis should include unix aider path");
    assert!(cli.contains("~/.config/opencode"), "ai-clis should include unix opencode path");
    assert!(cache.contains("~/.cache/transformers"), "ai-caches should include unix transformers cache");
    assert!(cache.contains("~/Library/Caches/promptfoo"), "ai-caches should include macos promptfoo cache");
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

#[test]
fn loads_gpu_runner_rule_yaml() {
    let content = fs::read_to_string("rules/gpu-runners.yaml").expect("gpu runner rule should exist");
    assert!(content.contains("lm-studio"));
    assert!(content.contains("llama.cpp"));
    assert!(content.contains("category: ai-model"));
}

#[test]
fn loads_ai_coding_agent_rule_yaml() {
    let content = fs::read_to_string("rules/ai-coding-agents.yaml").expect("ai coding agent rule should exist");
    assert!(content.contains("Claude Code"));
    assert!(content.contains("Codex CLI"));
    assert!(content.contains("Gemini CLI"));
}

#[test]
fn loads_mcp_server_rule_yaml() {
    let content = fs::read_to_string("rules/mcp-servers.yaml").expect("mcp server rule should exist");
    assert!(content.contains(".mcp"));
    assert!(content.contains("category: ai-agent"));
}

#[test]
fn loads_ai_ides_next_rule_yaml() {
    let content = fs::read_to_string("rules/ai-ides-next.yaml").expect("ai ides next rule should exist");
    assert!(content.contains("Roo Code"));
    assert!(content.contains("Codeium"));
}

#[test]
fn loads_model_files_rule_yaml() {
    let content = fs::read_to_string("rules/model-files.yaml").expect("model files rule should exist");
    assert!(content.contains(".gguf"));
    assert!(content.contains(".safetensors"));
    assert!(content.contains(".onnx"));
    assert!(content.contains("category: ai-model"));
}
