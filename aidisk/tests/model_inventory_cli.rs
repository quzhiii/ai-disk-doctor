use std::fs;
use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_aidisk")
}

#[test]
fn models_inventory_reports_managed_and_unknown_assets_as_read_only() {
    let temp = tempdir().expect("tempdir should exist");
    let snapshot = temp
        .path()
        .join("models--org--demo")
        .join("snapshots")
        .join("rev-1");
    fs::create_dir_all(&snapshot).expect("snapshot should exist");
    fs::write(snapshot.join("model.safetensors"), vec![0_u8; 12])
        .expect("managed model should write");
    let output = Command::new(binary())
        .args([
            "models",
            "inventory",
            "--json",
            "--tool",
            "huggingface",
            "--root",
            temp.path().to_str().expect("temp path should be utf8"),
        ])
        .output()
        .expect("models inventory should run");

    assert!(output.status.success(), "inventory should succeed");
    assert!(
        output.stderr.is_empty(),
        "stderr should be empty on success"
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("inventory should be json");

    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["summary"]["total_assets"], 1);
    assert_eq!(report["summary"]["logical_bytes"], 12);
    assert_eq!(report["summary"]["report_only_assets"], 1);
    assert!(report["edges"]
        .as_array()
        .expect("edges should be an array")
        .iter()
        .any(|edge| edge["relation"] == "managed-by"));

    let assets = report["assets"]
        .as_array()
        .expect("assets should be an array");
    assert!(assets.iter().any(|asset| {
        asset["logical_name"] == "org/demo"
            && asset["revision"] == "rev-1"
            && asset["format"] == "safetensors"
    }));
    let custom_root = temp.path().join("custom");
    fs::create_dir_all(&custom_root).expect("custom root should exist");
    fs::write(custom_root.join("private.gguf"), vec![0_u8; 8]).expect("custom model should write");
    let custom_output = Command::new(binary())
        .args([
            "models",
            "inventory",
            "--json",
            "--tool",
            "generic",
            "--root",
            custom_root.to_str().expect("custom root should be utf8"),
        ])
        .output()
        .expect("custom inventory should run");
    assert!(custom_output.status.success());
    let custom_report: Value =
        serde_json::from_slice(&custom_output.stdout).expect("custom inventory should be json");
    assert!(custom_report["assets"]
        .as_array()
        .expect("assets should exist")
        .iter()
        .any(|asset| {
            asset["state"] == "unknown-custom"
                && asset["suspected_custom_model"] == true
                && asset["reclaim_confidence"] == 0
        }));
}

#[test]
fn models_inventory_supports_text_and_markdown_modes() {
    let temp = tempdir().expect("tempdir should exist");
    fs::write(temp.path().join("demo.gguf"), vec![0_u8; 4]).expect("model should write");

    for format in ["text", "markdown"] {
        let mut args = vec![
            "models",
            "inventory",
            "--tool",
            "generic",
            "--root",
            temp.path().to_str().expect("temp path should be utf8"),
        ];
        if format == "markdown" {
            args.push("--markdown");
        }
        let output = Command::new(binary())
            .args(args)
            .output()
            .expect("models inventory should run");
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Model Asset Inventory"));
        assert!(stdout.contains("demo"));
    }
}

#[test]
fn models_inventory_does_not_require_a_tool_or_modify_files() {
    let temp = tempdir().expect("tempdir should exist");
    let blob = temp.path().join("blobs").join("sha256-demo");
    fs::create_dir_all(blob.parent().expect("blob parent should exist"))
        .expect("blob parent should exist");
    fs::write(&blob, vec![1_u8; 7]).expect("blob should write");

    let output = Command::new(binary())
        .args([
            "models",
            "inventory",
            "--json",
            "--tool",
            "generic",
            "--root",
            temp.path().to_str().expect("temp path should be utf8"),
        ])
        .output()
        .expect("models inventory should run");

    assert!(output.status.success());
    assert!(blob.exists(), "inventory must not mutate source files");
    let report: Value = serde_json::from_slice(&output.stdout).expect("inventory should be json");
    assert_eq!(report["summary"]["total_assets"], 0);
}

#[test]
fn models_inventory_accepts_stale_cutoff_and_reports_new_summary_fields() {
    let temp = tempdir().expect("tempdir should exist");
    let snapshot = temp.path().join("models--org--demo/snapshots/rev-1");
    fs::create_dir_all(&snapshot).expect("snapshot should exist");
    fs::write(snapshot.join("model.safetensors"), b"model").expect("model should write");

    let output = Command::new(binary())
        .args([
            "models",
            "inventory",
            "--json",
            "--tool",
            "huggingface",
            "--stale-after-days",
            "0",
            "--root",
            temp.path().to_str().expect("temp path should be utf8"),
        ])
        .output()
        .expect("models inventory should run");

    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).expect("inventory should be json");
    assert_eq!(report["summary"]["stale_assets"], 0);
    assert_eq!(report["summary"]["duplicate_logical_model_assets"], 0);
    assert_eq!(report["assets"][0]["stale"], false);
    assert_eq!(report["assets"][0]["duplicate_logical_model"], false);
}

#[test]
fn models_adapters_report_local_capabilities_without_invoking_tools() {
    let temp = tempdir().expect("tempdir should exist");
    let hf_root = temp.path().join("models--org--demo");
    fs::create_dir_all(hf_root.join("refs")).expect("refs should exist");
    fs::write(hf_root.join("refs/main"), "rev-1\n").expect("ref should write");

    let hf_output = Command::new(binary())
        .args([
            "models",
            "adapters",
            "--json",
            "--tool",
            "huggingface",
            "--root",
            temp.path().to_str().expect("temp path should be utf8"),
        ])
        .output()
        .expect("adapter report should run");
    assert!(hf_output.status.success());
    let hf_report: Value =
        serde_json::from_slice(&hf_output.stdout).expect("adapter report should be json");
    assert_eq!(hf_report["schema_version"], 3);
    assert_eq!(hf_report["summary"]["total_adapters"], 1);
    assert_eq!(hf_report["summary"]["index_parseable_adapters"], 1);
    assert_eq!(hf_report["adapters"][0]["action"], "report-only");
    assert_eq!(
        hf_report["adapters"][0]["plan_mode"],
        "metadata-only-dry-run"
    );
    assert_eq!(hf_report["adapters"][0]["official_cli"]["probed"], false);
    assert!(hf_report["adapters"][0]["official_cli"]["available"].is_null());

    let ollama_root = temp.path().join("ollama");
    fs::create_dir_all(ollama_root.join("manifests/library"))
        .expect("ollama manifests should exist");
    fs::write(
        ollama_root.join("manifests/library/demo"),
        r#"{"layers":[{"digest":"sha256:aaaaaaaa"}]}"#,
    )
    .expect("ollama manifest should write");
    let ollama_output = Command::new(binary())
        .args([
            "models",
            "adapters",
            "--json",
            "--tool",
            "ollama",
            "--root",
            ollama_root.to_str().expect("ollama path should be utf8"),
        ])
        .output()
        .expect("ollama adapter report should run");
    assert!(ollama_output.status.success());
    let ollama_report: Value =
        serde_json::from_slice(&ollama_output.stdout).expect("ollama report should be json");
    assert_eq!(ollama_report["summary"]["index_parseable_adapters"], 1);
    assert_eq!(
        ollama_report["adapters"][0]["official_cli"]["probed"],
        false
    );
}

#[test]
fn models_adapters_probe_is_explicit_and_bounded() {
    let temp = tempdir().expect("tempdir should exist");
    let root = temp.path().join("ollama");
    fs::create_dir_all(root.join("manifests/library")).expect("manifests should exist");
    fs::write(
        root.join("manifests/library/demo"),
        r#"{"layers":[{"digest":"sha256:aaaaaaaa"}]}"#,
    )
    .expect("manifest should write");

    let output = Command::new(binary())
        .args([
            "models",
            "adapters",
            "--json",
            "--tool",
            "ollama",
            "--root",
            root.to_str().expect("root should be utf8"),
        ])
        .output()
        .expect("default adapter report should run");
    assert!(output.status.success());
    let report: Value = serde_json::from_slice(&output.stdout).expect("report should be json");
    assert_eq!(report["adapters"][0]["official_cli"]["probed"], false);
    assert_eq!(
        report["adapters"][0]["official_cli"]["version_status"],
        "not-probed"
    );
    assert_eq!(report["adapters"][0]["plan_mode"], "metadata-only-dry-run");

    let probe_output = Command::new(binary())
        .args([
            "models",
            "adapters",
            "--json",
            "--tool",
            "ollama",
            "--probe-official-cli",
            "--probe-timeout-ms",
            "1",
            "--probe-output-chars",
            "1",
            "--root",
            root.to_str().expect("root should be utf8"),
        ])
        .output()
        .expect("opt-in adapter report should run");
    assert!(probe_output.status.success());
    let probe_report: Value =
        serde_json::from_slice(&probe_output.stdout).expect("probe report should be json");
    let cli = &probe_report["adapters"][0]["official_cli"];
    assert_eq!(cli["probed"], true);
    assert_eq!(cli["timeout_ms"], 1);
    assert_eq!(cli["output_limit_chars"], 1);
    assert_eq!(probe_report["adapters"][0]["action"], "report-only");
    assert_eq!(
        probe_report["adapters"][0]["plan_mode"],
        "metadata-and-official-cli-capability-dry-run"
    );
}
