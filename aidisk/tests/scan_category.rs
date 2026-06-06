use std::fs;
use std::path::Path;

use tempfile::tempdir;

#[test]
fn playwright_fixture_matches_glob_pattern_shape() {
    let temp = tempdir().expect("tempdir should exist");
    let fixture = temp
        .path()
        .join("workspace")
        .join("agent-project")
        .join(".playwright-browsers");

    fs::create_dir_all(&fixture).expect("fixture directory should be created");
    fs::write(fixture.join("chromium.zip"), vec![1_u8; 1024]).expect("fixture file should be written");

    assert!(fixture.exists());
    assert!(fixture.join("chromium.zip").exists());
}

#[test]
fn repository_fixtures_cover_core_storage_shapes() {
    let root = Path::new("tests/fixtures/windows-user");

    assert!(root.join("AppData/Local/Packages/DistroA/LocalState/ext4.vhdx").exists());
    assert!(root.join("AppData/Local/Docker/build-cache/cache.db").exists());
    assert!(root.join("AppData/Local/Docker/wsl/disk/docker_data.vhdx").exists());
    assert!(root.join("projects/demo-app/.playwright-browsers/chromium.zip").exists());
    assert!(root.join(".cache/huggingface/hub/models--demo/model.bin").exists());
    assert!(root.join(".ollama/models/blobs/sha256-demo").exists());
}

#[test]
fn repository_fixtures_cover_common_dev_artifacts() {
    let root = Path::new("tests/fixtures/windows-user/projects/dev-artifacts-demo");

    assert!(root.join("node_modules/pkg/index.js").exists());
    assert!(root.join("target/debug/app.exe").exists());
    assert!(root.join(".gradle/caches/modules-2/module.bin").exists());
    assert!(root.join("src/__pycache__/module.pyc").exists());
    assert!(root.join("dist/app.js").exists());
    assert!(root.join(".next/cache/build.bin").exists());
    assert!(root.join(".turbo/cache/hash.bin").exists());
}
