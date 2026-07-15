use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use clap::ValueEnum;
use serde::Serialize;
use walkdir::WalkDir;

pub const MODEL_INVENTORY_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InventoryTool {
    Auto,
    Ollama,
    Huggingface,
    Generic,
}

#[derive(Debug, Clone)]
pub struct InventoryOptions {
    pub root: Option<PathBuf>,
    pub tool: InventoryTool,
    pub max_depth: usize,
}

#[derive(Debug, Serialize)]
pub struct ModelInventoryReport {
    pub schema_version: u16,
    pub generated_at: DateTime<Local>,
    pub roots: Vec<InventoryRoot>,
    pub assets: Vec<ModelAsset>,
    pub nodes: Vec<ProvenanceNode>,
    pub edges: Vec<ProvenanceEdge>,
    pub summary: InventorySummary,
}

#[derive(Debug, Serialize)]
pub struct InventoryRoot {
    pub path: String,
    pub tool: String,
    pub exists: bool,
}

#[derive(Debug, Serialize)]
pub struct ModelAsset {
    pub id: String,
    pub logical_name: String,
    pub format: String,
    pub manager: String,
    pub revision: Option<String>,
    pub manifest: Option<String>,
    pub blob: Option<String>,
    pub snapshot: Option<String>,
    pub source: Option<String>,
    pub last_modified: Option<String>,
    pub last_accessed: Option<String>,
    pub logical_size_bytes: u64,
    pub exclusive_physical_size_bytes: u64,
    pub shared_physical_size_bytes: u64,
    pub recoverability: String,
    pub suspected_custom_model: bool,
    pub state: String,
    pub action: String,
    pub reclaim_confidence: u8,
    pub positive_evidence: Vec<String>,
    pub risk_evidence: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InventorySummary {
    pub total_assets: usize,
    pub logical_bytes: u64,
    pub exclusive_physical_bytes: u64,
    pub shared_physical_bytes: u64,
    pub managed_assets: usize,
    pub unknown_custom_assets: usize,
    pub report_only_assets: usize,
}

#[derive(Debug, Serialize)]
pub struct ProvenanceNode {
    pub id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
pub struct ProvenanceEdge {
    pub from: String,
    pub relation: String,
    pub to: String,
}

#[derive(Debug, Clone, Copy)]
struct PhysicalPath {
    size_bytes: u64,
    references: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DetectedTool {
    Ollama,
    Huggingface,
    Generic,
}

impl DetectedTool {
    fn label(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::Huggingface => "huggingface",
            Self::Generic => "generic",
        }
    }

    fn is_managed(self) -> bool {
        !matches!(self, Self::Generic)
    }
}

pub fn build_inventory(options: &InventoryOptions) -> Result<ModelInventoryReport> {
    let root_paths = options
        .root
        .clone()
        .map(|root| vec![root])
        .unwrap_or_else(default_model_roots);
    let mut roots = Vec::new();
    let mut assets = Vec::new();
    let mut physical_paths: HashMap<String, PhysicalPath> = HashMap::new();

    for root in root_paths {
        let detected = detect_tool(&root, options.tool);
        roots.push(InventoryRoot {
            path: root.display().to_string(),
            tool: detected.label().to_string(),
            exists: root.is_dir(),
        });

        if !root.is_dir() {
            continue;
        }

        for entry in WalkDir::new(&root)
            .follow_links(false)
            .max_depth(options.max_depth)
        {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            if entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();
            let metadata = match fs::metadata(path) {
                Ok(metadata) if metadata.is_file() => metadata,
                _ => continue,
            };
            if !is_model_asset(path, detected) {
                continue;
            }

            let physical_key = physical_key(path, &metadata);
            let logical_size_bytes = metadata.len();
            physical_paths
                .entry(physical_key.clone())
                .and_modify(|physical| physical.references += 1)
                .or_insert(PhysicalPath {
                    size_bytes: logical_size_bytes,
                    references: 1,
                });
            assets.push((
                build_asset(path, detected, logical_size_bytes),
                physical_key,
            ));
        }
    }

    for (asset, physical_key) in &mut assets {
        let physical = physical_paths
            .get(physical_key)
            .expect("every asset should have physical accounting");
        let is_shared = physical.references > 1;
        asset.exclusive_physical_size_bytes = if is_shared { 0 } else { physical.size_bytes };
        asset.shared_physical_size_bytes = if is_shared { physical.size_bytes } else { 0 };
    }

    let mut assets = assets
        .into_iter()
        .map(|(asset, _)| asset)
        .collect::<Vec<_>>();
    assets.sort_by(|left, right| {
        right
            .logical_size_bytes
            .cmp(&left.logical_size_bytes)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut node_ids = HashSet::new();
    for asset in &assets {
        let tool_id = format!("tool:{}", asset.manager);
        push_node(
            &mut nodes,
            &mut node_ids,
            ProvenanceNode {
                id: tool_id.clone(),
                kind: "tool".to_string(),
                label: asset.manager.clone(),
            },
        );
        push_node(
            &mut nodes,
            &mut node_ids,
            ProvenanceNode {
                id: asset.id.clone(),
                kind: "model".to_string(),
                label: asset.logical_name.clone(),
            },
        );
        edges.push(ProvenanceEdge {
            from: asset.id.clone(),
            relation: "managed-by".to_string(),
            to: tool_id,
        });

        if let Some(revision) = &asset.revision {
            let revision_id = format!("revision:{}@{}", asset.logical_name, revision);
            push_node(
                &mut nodes,
                &mut node_ids,
                ProvenanceNode {
                    id: revision_id.clone(),
                    kind: "revision".to_string(),
                    label: revision.clone(),
                },
            );
            edges.push(ProvenanceEdge {
                from: asset.id.clone(),
                relation: "belongs-to".to_string(),
                to: revision_id,
            });
        }
    }

    let logical_bytes = assets.iter().map(|asset| asset.logical_size_bytes).sum();
    let exclusive_physical_bytes = physical_paths
        .values()
        .filter(|physical| physical.references == 1)
        .map(|physical| physical.size_bytes)
        .sum();
    let shared_physical_bytes = physical_paths
        .values()
        .filter(|physical| physical.references > 1)
        .map(|physical| physical.size_bytes)
        .sum();

    Ok(ModelInventoryReport {
        schema_version: MODEL_INVENTORY_SCHEMA_VERSION,
        generated_at: Local::now(),
        roots,
        summary: InventorySummary {
            total_assets: assets.len(),
            logical_bytes,
            exclusive_physical_bytes,
            shared_physical_bytes,
            managed_assets: assets
                .iter()
                .filter(|asset| asset.manager != "generic")
                .count(),
            unknown_custom_assets: assets
                .iter()
                .filter(|asset| asset.suspected_custom_model)
                .count(),
            report_only_assets: assets
                .iter()
                .filter(|asset| asset.action == "report-only")
                .count(),
        },
        assets,
        nodes,
        edges,
    })
}

fn default_model_roots() -> Vec<PathBuf> {
    let Some(home) = env::var_os("USERPROFILE").or_else(|| env::var_os("HOME")) else {
        return Vec::new();
    };
    let home = PathBuf::from(home);
    vec![
        home.join(".ollama").join("models"),
        home.join(".cache").join("huggingface").join("hub"),
        home.join("Library")
            .join("Caches")
            .join("huggingface")
            .join("hub"),
    ]
}

fn detect_tool(path: &Path, requested: InventoryTool) -> DetectedTool {
    match requested {
        InventoryTool::Ollama => DetectedTool::Ollama,
        InventoryTool::Huggingface => DetectedTool::Huggingface,
        InventoryTool::Generic => DetectedTool::Generic,
        InventoryTool::Auto => {
            let normalized = normalize_path(path);
            if normalized.ends_with("/.ollama/models") || normalized.contains("/.ollama/models/") {
                DetectedTool::Ollama
            } else if normalized.contains("/.cache/huggingface")
                || normalized.contains("/library/caches/huggingface")
            {
                DetectedTool::Huggingface
            } else {
                DetectedTool::Generic
            }
        }
    }
}

fn is_model_asset(path: &Path, tool: DetectedTool) -> bool {
    let normalized = normalize_path(path);
    if tool == DetectedTool::Ollama {
        return normalized.contains("/blobs/sha256-");
    }

    let recognized_extension = matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("gguf" | "mlx" | "onnx" | "pt" | "pth" | "safetensors")
    );
    recognized_extension
        || (tool == DetectedTool::Huggingface
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("bin")))
}

fn build_asset(path: &Path, tool: DetectedTool, logical_size_bytes: u64) -> ModelAsset {
    let normalized = normalize_path(path);
    let logical_name = infer_logical_name(path, tool);
    let revision = path
        .components()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|parts| {
            parts[0]
                .as_os_str()
                .to_string_lossy()
                .eq_ignore_ascii_case("snapshots")
        })
        .map(|parts| parts[1].as_os_str().to_string_lossy().into_owned());
    let snapshot = revision.as_ref().map(|_| path.display().to_string());
    let blob = normalized
        .contains("/blobs/")
        .then(|| path.display().to_string());
    let source =
        (tool == DetectedTool::Huggingface).then(|| "huggingface-cache-layout".to_string());
    let suspected_custom_model = tool == DetectedTool::Generic;
    let state = if suspected_custom_model {
        "unknown-custom"
    } else if tool == DetectedTool::Ollama && blob.is_some() {
        "managed-cache-unresolved"
    } else {
        "managed-cache"
    };
    let recoverability = if tool.is_managed() {
        "likely-redownloadable"
    } else {
        "unknown"
    };
    let mut positive_evidence = vec!["matched recognized model format".to_string()];
    let mut risk_evidence = vec!["content and official indexes were not read".to_string()];
    if tool.is_managed() {
        positive_evidence.push(format!("located under {} cache layout", tool.label()));
        risk_evidence.push("logical model references were not resolved".to_string());
    } else {
        risk_evidence.push("path may contain a private or custom model".to_string());
    }

    ModelAsset {
        id: format!("asset:{}", path.display()),
        logical_name,
        format: infer_format(path, tool),
        manager: tool.label().to_string(),
        revision,
        manifest: find_named_ancestor(path, "manifests"),
        blob,
        snapshot,
        source,
        last_modified: metadata_time(path, |metadata| metadata.modified().ok()),
        last_accessed: metadata_time(path, |metadata| metadata.accessed().ok()),
        logical_size_bytes,
        exclusive_physical_size_bytes: 0,
        shared_physical_size_bytes: 0,
        recoverability: recoverability.to_string(),
        suspected_custom_model,
        state: state.to_string(),
        action: "report-only".to_string(),
        reclaim_confidence: if tool.is_managed() { 35 } else { 0 },
        positive_evidence,
        risk_evidence,
    }
}

fn infer_logical_name(path: &Path, tool: DetectedTool) -> String {
    let components = path
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    if tool == DetectedTool::Huggingface {
        if let Some(repository) = components
            .iter()
            .rev()
            .find(|component| component.starts_with("models--"))
        {
            return repository.trim_start_matches("models--").replace("--", "/");
        }
    }
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown-model")
        .to_string()
}

fn infer_format(path: &Path, tool: DetectedTool) -> String {
    if tool == DetectedTool::Ollama && normalize_path(path).contains("/blobs/sha256-") {
        return "ollama-blob".to_string();
    }
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

fn find_named_ancestor(path: &Path, name: &str) -> Option<String> {
    path.ancestors()
        .find(|ancestor| {
            ancestor
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(name))
        })
        .map(|ancestor| ancestor.display().to_string())
}

fn metadata_time<F>(path: &Path, read: F) -> Option<String>
where
    F: FnOnce(&fs::Metadata) -> Option<SystemTime>,
{
    let metadata = fs::metadata(path).ok()?;
    let time = read(&metadata)?;
    Some(DateTime::<Utc>::from(time).to_rfc3339())
}

fn physical_key(path: &Path, metadata: &fs::Metadata) -> String {
    #[cfg(not(unix))]
    let _ = metadata;

    #[cfg(unix)]
    {
        return format!("unix:{}:{}", metadata.dev(), metadata.ino());
    }

    #[cfg(windows)]
    {
        if let Some(file_id) = windows_file_id(path) {
            return file_id;
        }
    }

    fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

#[cfg(windows)]
fn windows_file_id(path: &Path) -> Option<String> {
    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct ByHandleFileInformation {
        file_attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(
            file: *mut std::ffi::c_void,
            information: *mut ByHandleFileInformation,
        ) -> i32;
    }

    let file = fs::File::open(path).ok()?;
    let mut information = ByHandleFileInformation {
        file_attributes: 0,
        creation_time: FileTime { low: 0, high: 0 },
        last_access_time: FileTime { low: 0, high: 0 },
        last_write_time: FileTime { low: 0, high: 0 },
        volume_serial_number: 0,
        file_size_high: 0,
        file_size_low: 0,
        number_of_links: 0,
        file_index_high: 0,
        file_index_low: 0,
    };
    let succeeded =
        unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) != 0 };
    if !succeeded {
        return None;
    }

    let file_index =
        (u64::from(information.file_index_high) << 32) | u64::from(information.file_index_low);
    Some(format!(
        "windows:{}:{file_index}",
        information.volume_serial_number
    ))
}

fn push_node(
    nodes: &mut Vec<ProvenanceNode>,
    node_ids: &mut HashSet<String>,
    node: ProvenanceNode,
) {
    if node_ids.insert(node.id.clone()) {
        nodes.push(node);
    }
}

fn normalize_path(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{build_inventory, InventoryOptions, InventoryTool};

    #[test]
    fn inventories_model_formats_without_reading_contents() {
        let temp = tempdir().expect("tempdir should exist");
        fs::create_dir_all(temp.path().join("models--demo--repo/snapshots/rev-1"))
            .expect("snapshot directory should exist");
        fs::write(
            temp.path()
                .join("models--demo--repo/snapshots/rev-1/model.safetensors"),
            b"not parsed",
        )
        .expect("model should be written");
        fs::write(temp.path().join("custom.gguf"), b"custom").expect("custom model should write");

        let report = build_inventory(&InventoryOptions {
            root: Some(temp.path().to_path_buf()),
            tool: InventoryTool::Huggingface,
            max_depth: 20,
        })
        .expect("inventory should succeed");

        assert_eq!(report.summary.total_assets, 2);
        assert_eq!(report.summary.logical_bytes, 16);
        assert!(report.assets.iter().any(|asset| {
            asset.logical_name == "demo/repo"
                && asset.format == "safetensors"
                && asset.revision.as_deref() == Some("rev-1")
        }));
        assert!(report
            .assets
            .iter()
            .all(|asset| asset.action == "report-only"));
        assert_eq!(report.summary.exclusive_physical_bytes, 16);
        assert_eq!(report.summary.shared_physical_bytes, 0);
        assert_eq!(
            report
                .nodes
                .iter()
                .filter(|node| node.kind == "revision")
                .count(),
            1
        );
    }

    #[test]
    fn generic_model_assets_are_marked_unknown_custom() {
        let temp = tempdir().expect("tempdir should exist");
        fs::write(temp.path().join("private.onnx"), b"model").expect("model should write");

        let report = build_inventory(&InventoryOptions {
            root: Some(temp.path().to_path_buf()),
            tool: InventoryTool::Auto,
            max_depth: 20,
        })
        .expect("inventory should succeed");

        assert_eq!(report.summary.unknown_custom_assets, 1);
        assert_eq!(report.assets[0].state, "unknown-custom");
        assert_eq!(report.assets[0].recoverability, "unknown");
        assert_eq!(report.assets[0].reclaim_confidence, 0);
    }

    #[test]
    fn shared_physical_paths_are_counted_once() {
        let temp = tempdir().expect("tempdir should exist");
        let snapshots = temp.path().join("models--demo--repo/snapshots/rev-1");
        fs::create_dir_all(&snapshots).expect("snapshot directory should exist");
        let first = snapshots.join("model-a.safetensors");
        let second = snapshots.join("model-b.safetensors");
        fs::write(&first, vec![0_u8; 10]).expect("model should write");
        fs::hard_link(&first, &second).expect("hard link should be created");

        let report = build_inventory(&InventoryOptions {
            root: Some(temp.path().to_path_buf()),
            tool: InventoryTool::Huggingface,
            max_depth: 20,
        })
        .expect("inventory should succeed");

        assert_eq!(report.summary.total_assets, 2);
        assert_eq!(report.summary.logical_bytes, 20);
        assert_eq!(report.summary.exclusive_physical_bytes, 0);
        assert_eq!(report.summary.shared_physical_bytes, 10);
        assert!(report
            .assets
            .iter()
            .all(|asset| asset.exclusive_physical_size_bytes == 0));
        assert_eq!(
            report
                .nodes
                .iter()
                .filter(|node| node.id == "revision:demo/repo@rev-1")
                .count(),
            1
        );
    }

    #[test]
    fn generic_bin_files_are_not_treated_as_models() {
        let temp = tempdir().expect("tempdir should exist");
        fs::write(temp.path().join("ordinary.bin"), b"cache").expect("cache should write");

        let report = build_inventory(&InventoryOptions {
            root: Some(temp.path().to_path_buf()),
            tool: InventoryTool::Generic,
            max_depth: 20,
        })
        .expect("inventory should succeed");

        assert_eq!(report.summary.total_assets, 0);
    }
}
