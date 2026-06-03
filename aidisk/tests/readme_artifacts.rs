use std::fs;

#[test]
fn readme_english_exists_and_has_required_sections() {
    let readme = fs::read_to_string("../README.md").expect("README.md should exist");
    assert!(readme.contains("# AI Disk Doctor"), "Should have English title");
    assert!(readme.contains("## Features"), "Should have Features section");
    assert!(readme.contains("## Architecture"), "Should have Architecture section");
    assert!(readme.contains("## Quick Start"), "Should have Quick Start section");
    assert!(readme.contains("## Command Reference"), "Should have Command Reference section");
    assert!(readme.contains("[中文](./README.zh-CN.md)"), "Should link to Chinese readme");
    assert!(readme.contains("![Version]"), "Should have version badge");
}

#[test]
fn readme_chinese_exists_and_has_required_sections() {
    let readme = fs::read_to_string("../README.zh-CN.md").expect("README.zh-CN.md should exist");
    assert!(readme.contains("# AI Disk Doctor"), "Should have Chinese title");
    assert!(readme.contains("## 功能特性"), "Should have Features section");
    assert!(readme.contains("## 架构设计"), "Should have Architecture section");
    assert!(readme.contains("## 快速开始"), "Should have Quick Start section");
    assert!(readme.contains("## 命令参考"), "Should have Command Reference section");
    assert!(readme.contains("[English](./README.md)"), "Should link to English readme");
}
