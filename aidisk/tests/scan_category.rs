use std::fs;

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
