use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

pub const MODEL_INVENTORY_SCHEMA_VERSION: u16 = 1;
pub const MODEL_ADAPTER_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InventoryTool {
    Auto,
    Ollama,
    Huggingface,
    Generic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterTool {
    Auto,
    Ollama,
    Huggingface,
}

#[derive(Debug, Clone)]
pub struct AdapterOptions {
    pub root: Option<PathBuf>,
    pub tool: AdapterTool,
    pub max_depth: usize,
}

#[derive(Debug, Serialize)]
pub struct ModelAdapterReport {
    pub schema_version: u16,
    pub generated_at: DateTime<Local>,
    pub adapters: Vec<ModelAdapterStatus>,
    pub summary: ModelAdapterSummary,
}

#[derive(Debug, Serialize)]
pub struct ModelAdapterStatus {
    pub tool: String,
    pub root: Option<String>,
    pub root_exists: bool,
    pub index_present: bool,
    pub index_parseable: bool,
    pub official_cli: Option<OfficialCliStatus>,
    pub capabilities: Vec<String>,
    pub plan_mode: String,
    pub action: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct OfficialCliStatus {
    pub command: String,
    pub available: Option<bool>,
    pub version: Option<String>,
    pub probed: bool,
}

#[derive(Debug, Serialize)]
pub struct ModelAdapterSummary {
    pub total_adapters: usize,
    pub available_adapters: usize,
    pub index_parseable_adapters: usize,
    pub dry_run_capable_adapters: usize,
    pub report_only_adapters: usize,
}

#[derive(Debug, Clone)]
pub struct InventoryOptions {
    pub root: Option<PathBuf>,
    pub tool: InventoryTool,
    pub max_depth: usize,
    pub stale_after_days: u64,
}

#[derive(Debug, Serialize)]
pub struct ModelInventoryReport {
    pub schema_version: u16,
    pub generated_at: DateTime<Local>,
    pub stale_after_days: u64,
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
    pub stale: bool,
    pub duplicate_logical_model: bool,
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
    pub referenced_assets: usize,
    pub detached_revision_assets: usize,
    pub orphan_blob_assets: usize,
    pub incomplete_download_assets: usize,
    pub stale_assets: usize,
    pub duplicate_logical_model_assets: usize,
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

#[derive(Debug)]
struct AssetRecord {
    asset: ModelAsset,
    path: PathBuf,
    physical_key: String,
    activity_time: Option<SystemTime>,
}

#[derive(Debug, Deserialize)]
struct OllamaManifest {
    #[serde(default)]
    config: Option<OllamaDescriptor>,
    #[serde(default)]
    layers: Vec<OllamaDescriptor>,
}

#[derive(Debug, Deserialize)]
struct OllamaDescriptor {
    digest: Option<String>,
}

#[derive(Debug, Default)]
struct InventoryAnalysis {
    referenced_asset_ids: HashSet<String>,
    referenced_physical_keys: HashSet<String>,
    detached_revision_asset_ids: HashSet<String>,
    orphan_blob_asset_ids: HashSet<String>,
    incomplete_download_asset_ids: HashSet<String>,
    stale_asset_ids: HashSet<String>,
    duplicate_logical_model_asset_ids: HashSet<String>,
    nodes: Vec<ProvenanceNode>,
    edges: Vec<ProvenanceEdge>,
    node_ids: HashSet<String>,
    edge_ids: HashSet<String>,
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

    for root in &root_paths {
        let detected = detect_tool(root, options.tool);
        roots.push(InventoryRoot {
            path: root.display().to_string(),
            tool: detected.label().to_string(),
            exists: root.is_dir(),
        });

        if !root.is_dir() {
            continue;
        }

        for entry in WalkDir::new(root)
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
            assets.push(AssetRecord {
                asset: build_asset(path, detected, logical_size_bytes),
                path: path.to_path_buf(),
                physical_key,
                activity_time: latest_activity_time(&metadata),
            });
        }
    }

    let mut analysis = InventoryAnalysis::default();
    mark_stale_assets(&assets, options.stale_after_days, &mut analysis);
    mark_duplicate_logical_models(&assets, &mut analysis);
    for root in &root_paths {
        let detected = detect_tool(root, options.tool);
        match detected {
            DetectedTool::Huggingface => {
                analyze_huggingface(root, options.max_depth, &mut assets, &mut analysis)
            }
            DetectedTool::Ollama => {
                analyze_ollama(root, options.max_depth, &mut assets, &mut analysis)
            }
            DetectedTool::Generic => {}
        }
    }

    for record in &mut assets {
        let asset = &mut record.asset;
        let physical_key = &record.physical_key;
        let physical = physical_paths
            .get(physical_key)
            .expect("every asset should have physical accounting");
        let is_shared = physical.references > 1;
        asset.exclusive_physical_size_bytes = if is_shared { 0 } else { physical.size_bytes };
        asset.shared_physical_size_bytes = if is_shared { physical.size_bytes } else { 0 };

        if analysis.incomplete_download_asset_ids.contains(&asset.id) {
            asset.state = "incomplete-download".to_string();
        } else if analysis.orphan_blob_asset_ids.contains(&asset.id) {
            asset.state = "orphan-blob".to_string();
        } else if analysis.detached_revision_asset_ids.contains(&asset.id) {
            asset.state = "detached-revision".to_string();
        } else if analysis.referenced_asset_ids.contains(&asset.id) {
            asset.state = "managed-cache-referenced".to_string();
        }

        match asset.state.as_str() {
            "managed-cache-referenced" => {
                asset
                    .positive_evidence
                    .push("resolved from a local model reference index".to_string());
                asset
                    .risk_evidence
                    .retain(|evidence| evidence != "logical model references were not resolved");
            }
            "detached-revision" => asset
                .risk_evidence
                .push("snapshot revision is not selected by a parsed ref".to_string()),
            "orphan-blob" => asset
                .risk_evidence
                .push("no parsed local index references this blob".to_string()),
            "incomplete-download" => asset
                .risk_evidence
                .push("filename indicates an incomplete download".to_string()),
            _ => {}
        }
        asset.stale = analysis.stale_asset_ids.contains(&asset.id);
        asset.duplicate_logical_model = analysis
            .duplicate_logical_model_asset_ids
            .contains(&asset.id);
        if asset.stale {
            asset.risk_evidence.push(format!(
                "last activity is older than {} days",
                options.stale_after_days
            ));
        }
        if asset.duplicate_logical_model {
            asset.risk_evidence.push(
                "same logical model identity appears in multiple revisions or paths".to_string(),
            );
        }
        asset.reclaim_confidence = reclaim_confidence(asset);
    }

    let mut assets = assets
        .into_iter()
        .map(|record| record.asset)
        .collect::<Vec<_>>();
    assets.sort_by(|left, right| {
        right
            .logical_size_bytes
            .cmp(&left.logical_size_bytes)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut nodes = analysis.nodes;
    let mut edges = analysis.edges;
    let mut node_ids = analysis.node_ids;
    let mut edge_ids = analysis.edge_ids;
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
        push_edge(
            &mut edges,
            &mut edge_ids,
            ProvenanceEdge {
                from: asset.id.clone(),
                relation: "managed-by".to_string(),
                to: tool_id,
            },
        );

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
            push_edge(
                &mut edges,
                &mut edge_ids,
                ProvenanceEdge {
                    from: asset.id.clone(),
                    relation: "belongs-to".to_string(),
                    to: revision_id,
                },
            );
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
        stale_after_days: options.stale_after_days,
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
            referenced_assets: assets
                .iter()
                .filter(|asset| asset.state == "managed-cache-referenced")
                .count(),
            detached_revision_assets: assets
                .iter()
                .filter(|asset| asset.state == "detached-revision")
                .count(),
            orphan_blob_assets: assets
                .iter()
                .filter(|asset| asset.state == "orphan-blob")
                .count(),
            incomplete_download_assets: assets
                .iter()
                .filter(|asset| asset.state == "incomplete-download")
                .count(),
            stale_assets: assets.iter().filter(|asset| asset.stale).count(),
            duplicate_logical_model_assets: assets
                .iter()
                .filter(|asset| asset.duplicate_logical_model)
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

pub fn build_adapter_report(options: &AdapterOptions) -> Result<ModelAdapterReport> {
    let adapter_roots = adapter_roots(options);
    let adapters = adapter_roots
        .into_iter()
        .map(|(tool, root)| build_adapter_status(tool, root, options.max_depth))
        .collect::<Vec<_>>();

    Ok(ModelAdapterReport {
        schema_version: MODEL_ADAPTER_SCHEMA_VERSION,
        generated_at: Local::now(),
        summary: ModelAdapterSummary {
            total_adapters: adapters.len(),
            available_adapters: adapters
                .iter()
                .filter(|adapter| adapter.root_exists)
                .count(),
            index_parseable_adapters: adapters
                .iter()
                .filter(|adapter| adapter.index_parseable)
                .count(),
            dry_run_capable_adapters: adapters
                .iter()
                .filter(|adapter| adapter.root_exists)
                .count(),
            report_only_adapters: adapters
                .iter()
                .filter(|adapter| adapter.action == "report-only")
                .count(),
        },
        adapters,
    })
}

fn adapter_roots(options: &AdapterOptions) -> Vec<(DetectedTool, PathBuf)> {
    if let Some(root) = &options.root {
        return match options.tool {
            AdapterTool::Ollama => vec![(DetectedTool::Ollama, root.clone())],
            AdapterTool::Huggingface => vec![(DetectedTool::Huggingface, root.clone())],
            AdapterTool::Auto => match detect_tool(root, InventoryTool::Auto) {
                DetectedTool::Ollama => vec![(DetectedTool::Ollama, root.clone())],
                DetectedTool::Huggingface => vec![(DetectedTool::Huggingface, root.clone())],
                DetectedTool::Generic => vec![
                    (DetectedTool::Ollama, root.clone()),
                    (DetectedTool::Huggingface, root.clone()),
                ],
            },
        };
    }

    match options.tool {
        AdapterTool::Ollama => vec![(DetectedTool::Ollama, default_ollama_root())],
        AdapterTool::Huggingface => vec![(DetectedTool::Huggingface, default_huggingface_root())],
        AdapterTool::Auto => vec![
            (DetectedTool::Ollama, default_ollama_root()),
            (DetectedTool::Huggingface, default_huggingface_root()),
        ],
    }
}

fn build_adapter_status(tool: DetectedTool, root: PathBuf, max_depth: usize) -> ModelAdapterStatus {
    let root_exists = root.is_dir();
    let index = match tool {
        DetectedTool::Ollama => probe_ollama_index(&root, max_depth),
        DetectedTool::Huggingface => probe_huggingface_index(&root, max_depth),
        DetectedTool::Generic => IndexProbe::default(),
    };
    let mut capabilities = vec![
        "metadata-index-inspection".to_string(),
        "report-only-dry-run".to_string(),
    ];
    if index.parseable {
        capabilities.push("provenance-resolution".to_string());
    }
    capabilities.push("official-cli-dry-run-pending".to_string());

    let mut evidence = index.evidence;
    evidence.push("external official CLI was not invoked".to_string());
    evidence.push("no cleanup or index mutation is performed".to_string());

    ModelAdapterStatus {
        tool: tool.label().to_string(),
        root: Some(root.display().to_string()),
        root_exists,
        index_present: index.present,
        index_parseable: index.parseable,
        official_cli: Some(OfficialCliStatus {
            command: match tool {
                DetectedTool::Ollama => "ollama".to_string(),
                DetectedTool::Huggingface => "hf".to_string(),
                DetectedTool::Generic => "".to_string(),
            },
            available: None,
            version: None,
            probed: false,
        }),
        capabilities,
        plan_mode: "metadata-only-dry-run".to_string(),
        action: "report-only".to_string(),
        evidence,
    }
}

#[derive(Debug, Default)]
struct IndexProbe {
    present: bool,
    parseable: bool,
    evidence: Vec<String>,
}

fn probe_huggingface_index(root: &Path, max_depth: usize) -> IndexProbe {
    let mut probe = IndexProbe::default();
    if !root.is_dir() {
        probe
            .evidence
            .push("Hugging Face cache root is not present".to_string());
        return probe;
    }

    for entry in WalkDir::new(root).follow_links(false).max_depth(max_depth) {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(parent) = entry.path().parent() else {
            continue;
        };
        if !parent
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("refs"))
        {
            continue;
        }
        probe.present = true;
        let Ok(metadata) = fs::metadata(entry.path()) else {
            continue;
        };
        if metadata.len() > 1024 * 1024 {
            continue;
        }
        let Ok(content) = fs::read_to_string(entry.path()) else {
            continue;
        };
        if !content.trim().is_empty() {
            probe.parseable = true;
            probe
                .evidence
                .push("parsed at least one Hugging Face refs entry".to_string());
            break;
        }
    }
    if !probe.present {
        probe
            .evidence
            .push("Hugging Face refs index was not found".to_string());
    } else if !probe.parseable {
        probe
            .evidence
            .push("Hugging Face refs index was present but not parseable".to_string());
    }
    probe
}

fn probe_ollama_index(root: &Path, max_depth: usize) -> IndexProbe {
    let mut probe = IndexProbe::default();
    if !root.is_dir() {
        probe
            .evidence
            .push("Ollama models root is not present".to_string());
        return probe;
    }

    for entry in WalkDir::new(root).follow_links(false).max_depth(max_depth) {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_file() || !is_ollama_manifest(entry.path()) {
            continue;
        }
        probe.present = true;
        let Ok(metadata) = fs::metadata(entry.path()) else {
            continue;
        };
        if metadata.len() > 1024 * 1024 {
            continue;
        }
        let Ok(content) = fs::read_to_string(entry.path()) else {
            continue;
        };
        if serde_json::from_str::<OllamaManifest>(&content).is_ok() {
            probe.parseable = true;
            probe
                .evidence
                .push("parsed at least one Ollama manifest".to_string());
            break;
        }
    }
    if !probe.present {
        probe
            .evidence
            .push("Ollama manifest index was not found".to_string());
    } else if !probe.parseable {
        probe
            .evidence
            .push("Ollama manifest index was present but not parseable".to_string());
    }
    probe
}

fn default_ollama_root() -> PathBuf {
    default_model_roots()
        .into_iter()
        .next()
        .unwrap_or_else(|| PathBuf::from(".ollama/models"))
}

fn default_huggingface_root() -> PathBuf {
    default_model_roots()
        .into_iter()
        .find(|root| normalize_path(root).contains("huggingface"))
        .unwrap_or_else(|| PathBuf::from(".cache/huggingface/hub"))
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

fn mark_stale_assets(
    assets: &[AssetRecord],
    stale_after_days: u64,
    analysis: &mut InventoryAnalysis,
) {
    if stale_after_days == 0 {
        return;
    }
    let cutoff =
        SystemTime::now().checked_sub(Duration::from_secs(stale_after_days.saturating_mul(86_400)));
    let Some(cutoff) = cutoff else {
        return;
    };
    for record in assets {
        if record.asset.manager != "generic" && is_stale(record.activity_time, cutoff) {
            analysis.stale_asset_ids.insert(record.asset.id.clone());
        }
    }
}

fn is_stale(activity: Option<SystemTime>, cutoff: SystemTime) -> bool {
    activity.is_some_and(|activity| activity < cutoff)
}

fn mark_duplicate_logical_models(assets: &[AssetRecord], analysis: &mut InventoryAnalysis) {
    let mut identities: HashMap<(String, String, String), Vec<&AssetRecord>> = HashMap::new();
    for record in assets {
        if record.asset.manager == "generic" {
            continue;
        }
        let revision = record.asset.revision.clone().unwrap_or_default();
        identities
            .entry((
                record.asset.manager.clone(),
                record.asset.logical_name.clone(),
                revision,
            ))
            .or_default()
            .push(record);
    }

    let mut logical_revisions: HashMap<(String, String), HashSet<String>> = HashMap::new();
    for ((manager, logical_name, revision), records) in identities {
        let key = (manager, logical_name);
        logical_revisions.entry(key).or_default().insert(revision);
        let _ = records;
    }

    for record in assets {
        if record.asset.manager == "generic" {
            continue;
        }
        let key = (
            record.asset.manager.clone(),
            record.asset.logical_name.clone(),
        );
        if logical_revisions
            .get(&key)
            .is_some_and(|revisions| revisions.len() > 1 && !revisions.contains(""))
        {
            analysis
                .duplicate_logical_model_asset_ids
                .insert(record.asset.id.clone());
        }
    }
}

fn reclaim_confidence(asset: &ModelAsset) -> u8 {
    if asset.suspected_custom_model || asset.state == "incomplete-download" {
        return 0;
    }
    let mut score = if asset.manager == "generic" { 0 } else { 35 };
    if asset.state == "orphan-blob" {
        score += 25;
    }
    if asset.state == "detached-revision" {
        score += 15;
    }
    if asset.stale {
        score += 15;
    }
    if asset.duplicate_logical_model {
        score += 10;
    }
    score.min(100)
}

fn analyze_huggingface(
    root: &Path,
    max_depth: usize,
    assets: &mut [AssetRecord],
    analysis: &mut InventoryAnalysis,
) {
    let mut indexed_models = HashSet::new();
    let mut referenced_snapshots = HashSet::new();
    let mut parsed_ref_index = false;

    for entry in WalkDir::new(root).follow_links(false).max_depth(max_depth) {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(refs_dir) = entry.path().parent() else {
            continue;
        };
        if !refs_dir
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("refs"))
        {
            continue;
        }
        let Some(model_dir) = refs_dir.parent() else {
            continue;
        };
        let Some(model_name) = huggingface_model_name(model_dir) else {
            continue;
        };
        let Ok(metadata) = fs::metadata(entry.path()) else {
            continue;
        };
        if metadata.len() > 1024 * 1024 {
            continue;
        }
        let Ok(revision) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let revision = revision.trim();
        if revision.is_empty() {
            continue;
        }

        parsed_ref_index = true;
        indexed_models.insert(model_name.clone());
        referenced_snapshots.insert((model_name.clone(), revision.to_string()));
        let ref_id = format!("ref:{}", entry.path().display());
        let revision_id = format!("revision:{}@{}", model_name, revision);
        push_node(
            &mut analysis.nodes,
            &mut analysis.node_ids,
            ProvenanceNode {
                id: ref_id.clone(),
                kind: "ref".to_string(),
                label: entry.path().display().to_string(),
            },
        );
        push_node(
            &mut analysis.nodes,
            &mut analysis.node_ids,
            ProvenanceNode {
                id: revision_id.clone(),
                kind: "revision".to_string(),
                label: revision.to_string(),
            },
        );
        push_edge(
            &mut analysis.edges,
            &mut analysis.edge_ids,
            ProvenanceEdge {
                from: ref_id,
                relation: "resolves-to".to_string(),
                to: revision_id,
            },
        );
    }

    for record in assets.iter() {
        if record.asset.manager != "huggingface" {
            continue;
        }
        if is_incomplete_download_path(&record.path) {
            analysis
                .incomplete_download_asset_ids
                .insert(record.asset.id.clone());
            continue;
        }
        let Some(revision) = record.asset.revision.as_deref() else {
            continue;
        };
        let Some(model_name) = huggingface_model_name(&record.path) else {
            continue;
        };
        let snapshot_path = huggingface_snapshot_path(&record.path);
        if let Some(snapshot_path) = &snapshot_path {
            let snapshot_id = format!("snapshot:{}", snapshot_path.display());
            push_node(
                &mut analysis.nodes,
                &mut analysis.node_ids,
                ProvenanceNode {
                    id: snapshot_id.clone(),
                    kind: "snapshot".to_string(),
                    label: snapshot_path.display().to_string(),
                },
            );
            push_edge(
                &mut analysis.edges,
                &mut analysis.edge_ids,
                ProvenanceEdge {
                    from: record.asset.id.clone(),
                    relation: "part-of".to_string(),
                    to: snapshot_id.clone(),
                },
            );

            let revision_id = format!("revision:{}@{}", model_name, revision);
            push_edge(
                &mut analysis.edges,
                &mut analysis.edge_ids,
                ProvenanceEdge {
                    from: snapshot_id.clone(),
                    relation: "belongs-to".to_string(),
                    to: revision_id,
                },
            );

            if let Some(blob_record) = assets.iter().find(|candidate| {
                candidate.asset.manager == "huggingface"
                    && is_under_directory(&candidate.path, "blobs")
                    && candidate.physical_key == record.physical_key
            }) {
                let blob_id = blob_record.asset.id.clone();
                analysis
                    .referenced_physical_keys
                    .insert(blob_record.physical_key.clone());
                push_node(
                    &mut analysis.nodes,
                    &mut analysis.node_ids,
                    ProvenanceNode {
                        id: blob_id.clone(),
                        kind: "blob".to_string(),
                        label: blob_record.path.display().to_string(),
                    },
                );
                push_edge(
                    &mut analysis.edges,
                    &mut analysis.edge_ids,
                    ProvenanceEdge {
                        from: snapshot_id,
                        relation: "references".to_string(),
                        to: blob_id,
                    },
                );
            }
        } else if parsed_ref_index {
            analysis
                .detached_revision_asset_ids
                .insert(record.asset.id.clone());
        }

        let key = (model_name, revision.to_string());
        if indexed_models.contains(&key.0) {
            if referenced_snapshots.contains(&key) {
                analysis
                    .referenced_asset_ids
                    .insert(record.asset.id.clone());
            } else {
                analysis
                    .detached_revision_asset_ids
                    .insert(record.asset.id.clone());
            }
        }
    }

    if parsed_ref_index {
        for record in assets.iter() {
            if record.asset.manager != "huggingface" || !is_under_directory(&record.path, "blobs") {
                continue;
            }
            if is_incomplete_download_path(&record.path) {
                analysis
                    .incomplete_download_asset_ids
                    .insert(record.asset.id.clone());
            } else if analysis
                .referenced_physical_keys
                .contains(&record.physical_key)
            {
                analysis
                    .referenced_asset_ids
                    .insert(record.asset.id.clone());
            } else {
                analysis
                    .orphan_blob_asset_ids
                    .insert(record.asset.id.clone());
            }
        }
    }
}

fn analyze_ollama(
    root: &Path,
    max_depth: usize,
    assets: &mut [AssetRecord],
    analysis: &mut InventoryAnalysis,
) {
    let mut parsed_manifest = false;

    for entry in WalkDir::new(root).follow_links(false).max_depth(max_depth) {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry.file_type().is_file() || !is_ollama_manifest(entry.path()) {
            continue;
        }
        let Ok(metadata) = fs::metadata(entry.path()) else {
            continue;
        };
        if metadata.len() > 1024 * 1024 {
            continue;
        }
        let Ok(content) = fs::read_to_string(entry.path()) else {
            continue;
        };
        let Ok(manifest) = serde_json::from_str::<OllamaManifest>(&content) else {
            continue;
        };
        parsed_manifest = true;

        let manifest_id = format!("manifest:{}", entry.path().display());
        let model_name = ollama_model_name(entry.path());
        let model_id = format!("ollama-model:{model_name}");
        push_node(
            &mut analysis.nodes,
            &mut analysis.node_ids,
            ProvenanceNode {
                id: model_id.clone(),
                kind: "model".to_string(),
                label: model_name,
            },
        );
        push_node(
            &mut analysis.nodes,
            &mut analysis.node_ids,
            ProvenanceNode {
                id: manifest_id.clone(),
                kind: "manifest".to_string(),
                label: entry.path().display().to_string(),
            },
        );
        push_edge(
            &mut analysis.edges,
            &mut analysis.edge_ids,
            ProvenanceEdge {
                from: model_id,
                relation: "has-manifest".to_string(),
                to: manifest_id.clone(),
            },
        );

        let descriptors = manifest
            .config
            .into_iter()
            .chain(manifest.layers.into_iter())
            .collect::<Vec<_>>();
        for descriptor in descriptors {
            let Some(blob_name) = descriptor.digest.as_deref().and_then(ollama_blob_name) else {
                continue;
            };
            let blob_id = format!("blob:ollama:{blob_name}");
            push_node(
                &mut analysis.nodes,
                &mut analysis.node_ids,
                ProvenanceNode {
                    id: blob_id.clone(),
                    kind: "blob".to_string(),
                    label: blob_name.clone(),
                },
            );
            push_edge(
                &mut analysis.edges,
                &mut analysis.edge_ids,
                ProvenanceEdge {
                    from: manifest_id.clone(),
                    relation: "references".to_string(),
                    to: blob_id,
                },
            );

            if let Some(record) = assets.iter().find(|record| {
                record.asset.manager == "ollama"
                    && record.path.file_name().and_then(|name| name.to_str())
                        == Some(blob_name.as_str())
            }) {
                analysis
                    .referenced_asset_ids
                    .insert(record.asset.id.clone());
            }
        }
    }

    if parsed_manifest {
        for record in assets.iter() {
            if record.asset.manager != "ollama" {
                continue;
            }
            if is_incomplete_download_path(&record.path) {
                analysis
                    .incomplete_download_asset_ids
                    .insert(record.asset.id.clone());
            } else if !analysis.referenced_asset_ids.contains(&record.asset.id) {
                analysis
                    .orphan_blob_asset_ids
                    .insert(record.asset.id.clone());
            }
        }
    }
}

fn huggingface_model_name(path: &Path) -> Option<String> {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .find(|component| component.starts_with("models--"))
        .map(|component| component.trim_start_matches("models--").replace("--", "/"))
}

fn huggingface_snapshot_path(path: &Path) -> Option<PathBuf> {
    let snapshots = path.ancestors().find(|ancestor| {
        ancestor
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("snapshots"))
    })?;
    let revision = path
        .strip_prefix(snapshots)
        .ok()?
        .components()
        .next()?
        .as_os_str();
    Some(snapshots.join(revision))
}

fn is_under_directory(path: &Path, directory_name: &str) -> bool {
    path.ancestors().any(|ancestor| {
        ancestor
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case(directory_name))
    })
}

fn is_ollama_manifest(path: &Path) -> bool {
    is_under_directory(path, "manifests")
}

fn ollama_model_name(path: &Path) -> String {
    let components = path
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    let Some(manifests) = components
        .iter()
        .position(|component| component.eq_ignore_ascii_case("manifests"))
    else {
        return path.display().to_string();
    };
    components[manifests + 1..].join("/")
}

fn ollama_blob_name(digest: &str) -> Option<String> {
    let digest = digest.strip_prefix("sha256:")?;
    if digest.is_empty()
        || !digest
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return None;
    }
    Some(format!("sha256-{digest}"))
}

fn is_incomplete_download_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(".incomplete"))
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.to_ascii_lowercase().ends_with("-incomplete"))
}

fn is_model_asset(path: &Path, tool: DetectedTool) -> bool {
    let normalized = normalize_path(path);
    if tool.is_managed() && is_incomplete_download_path(path) {
        return true;
    }
    if tool == DetectedTool::Huggingface && is_under_directory(path, "blobs") {
        return true;
    }
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
    let source = match tool {
        DetectedTool::Huggingface => Some("huggingface-cache-layout".to_string()),
        DetectedTool::Ollama => Some("ollama-cache-layout".to_string()),
        DetectedTool::Generic => None,
    };
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
    let mut risk_evidence =
        vec!["model contents and external tool state were not read".to_string()];
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
        stale: false,
        duplicate_logical_model: false,
        action: "report-only".to_string(),
        reclaim_confidence: 0,
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
    if normalize_path(path).contains("/blobs/sha256-") {
        return format!("{}-blob", tool.label());
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

fn latest_activity_time(metadata: &fs::Metadata) -> Option<SystemTime> {
    [metadata.accessed().ok(), metadata.modified().ok()]
        .into_iter()
        .flatten()
        .max()
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

fn push_edge(
    edges: &mut Vec<ProvenanceEdge>,
    edge_ids: &mut HashSet<String>,
    edge: ProvenanceEdge,
) {
    let edge_id = format!("{}|{}|{}", edge.from, edge.relation, edge.to);
    if edge_ids.insert(edge_id) {
        edges.push(edge);
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

    use super::{build_inventory, is_stale, InventoryOptions, InventoryTool};
    use std::time::{Duration, SystemTime};

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
            stale_after_days: 30,
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
            stale_after_days: 30,
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
            stale_after_days: 30,
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
            stale_after_days: 30,
        })
        .expect("inventory should succeed");

        assert_eq!(report.summary.total_assets, 0);
    }

    #[test]
    fn huggingface_refs_and_blobs_resolve_asset_states() {
        let temp = tempdir().expect("tempdir should exist");
        let model_root = temp.path().join("models--org--demo");
        let snapshot = model_root.join("snapshots").join("rev-live");
        let detached_snapshot = model_root.join("snapshots").join("rev-old");
        let blobs = model_root.join("blobs");
        let refs = model_root.join("refs");
        fs::create_dir_all(&snapshot).expect("snapshot should exist");
        fs::create_dir_all(&detached_snapshot).expect("detached snapshot should exist");
        fs::create_dir_all(&blobs).expect("blobs should exist");
        fs::create_dir_all(&refs).expect("refs should exist");

        let live_blob = blobs.join("livehash");
        fs::write(&live_blob, vec![0_u8; 11]).expect("live blob should write");
        fs::hard_link(&live_blob, snapshot.join("model.safetensors"))
            .expect("snapshot link should be created");
        fs::write(detached_snapshot.join("detached.safetensors"), b"detached")
            .expect("detached snapshot should write");
        fs::write(blobs.join("orphanhash"), b"orphan").expect("orphan blob should write");
        fs::write(blobs.join("partial.incomplete"), b"partial").expect("partial blob should write");
        fs::write(refs.join("main"), "rev-live\n").expect("ref should write");

        let report = build_inventory(&InventoryOptions {
            root: Some(temp.path().to_path_buf()),
            tool: InventoryTool::Huggingface,
            max_depth: 20,
            stale_after_days: 30,
        })
        .expect("inventory should succeed");

        assert_eq!(report.summary.referenced_assets, 2);
        assert_eq!(report.summary.detached_revision_assets, 1);
        assert_eq!(report.summary.orphan_blob_assets, 1);
        assert_eq!(report.summary.incomplete_download_assets, 1);
        assert!(report
            .edges
            .iter()
            .any(|edge| { edge.relation == "resolves-to" && edge.from.starts_with("ref:") }));
        assert!(report
            .edges
            .iter()
            .any(|edge| { edge.relation == "references" && edge.from.starts_with("snapshot:") }));
    }

    #[test]
    fn ollama_manifests_resolve_referenced_and_orphan_blobs() {
        let temp = tempdir().expect("tempdir should exist");
        let manifests = temp.path().join("manifests/library");
        let blobs = temp.path().join("blobs");
        fs::create_dir_all(&manifests).expect("manifests should exist");
        fs::create_dir_all(&blobs).expect("blobs should exist");
        fs::write(
            manifests.join("demo"),
            r#"{"config":{"digest":"sha256:aaaaaaaa"},"layers":[{"digest":"sha256:bbbbbbbb"}]}"#,
        )
        .expect("manifest should write");
        fs::write(blobs.join("sha256-aaaaaaaa"), b"config").expect("config blob should write");
        fs::write(blobs.join("sha256-bbbbbbbb"), b"layer").expect("layer blob should write");
        fs::write(blobs.join("sha256-cccccccc"), b"orphan").expect("orphan blob should write");
        fs::write(blobs.join("sha256-dddddddd.incomplete"), b"partial")
            .expect("partial blob should write");

        let report = build_inventory(&InventoryOptions {
            root: Some(temp.path().to_path_buf()),
            tool: InventoryTool::Ollama,
            max_depth: 20,
            stale_after_days: 30,
        })
        .expect("inventory should succeed");

        assert_eq!(report.summary.referenced_assets, 2);
        assert_eq!(report.summary.orphan_blob_assets, 1);
        assert_eq!(report.summary.incomplete_download_assets, 1);
        assert!(report.edges.iter().any(|edge| {
            edge.relation == "has-manifest" && edge.from.starts_with("ollama-model:")
        }));
        assert!(report.edges.iter().any(|edge| {
            edge.relation == "references" && edge.to == "blob:ollama:sha256-bbbbbbbb"
        }));
    }

    #[test]
    fn stale_and_duplicate_assets_are_reported_without_enabling_cleanup() {
        let temp = tempdir().expect("tempdir should exist");
        let first = temp.path().join("models--org--demo/snapshots/rev-a");
        let second = temp.path().join("models--org--demo/snapshots/rev-b");
        fs::create_dir_all(&first).expect("first snapshot should exist");
        fs::create_dir_all(&second).expect("second snapshot should exist");
        fs::write(first.join("model.safetensors"), b"first").expect("first model should write");
        fs::write(second.join("model.safetensors"), b"second").expect("second model should write");

        let report = build_inventory(&InventoryOptions {
            root: Some(temp.path().to_path_buf()),
            tool: InventoryTool::Huggingface,
            max_depth: 20,
            stale_after_days: 1,
        })
        .expect("inventory should succeed");

        assert_eq!(report.summary.duplicate_logical_model_assets, 2);
        assert!(report
            .assets
            .iter()
            .all(|asset| asset.duplicate_logical_model));
        assert!(report
            .assets
            .iter()
            .all(|asset| asset.action == "report-only"));
        assert_eq!(report.summary.stale_assets, 0);
    }

    #[test]
    fn stale_assets_use_metadata_cutoff_and_confidence_is_explanatory() {
        let temp = tempdir().expect("tempdir should exist");
        let root = temp.path().join("models--org--demo/snapshots/rev-a");
        fs::create_dir_all(&root).expect("snapshot should exist");
        let model = root.join("model.safetensors");
        fs::write(&model, b"model").expect("model should write");

        let report = build_inventory(&InventoryOptions {
            root: Some(temp.path().to_path_buf()),
            tool: InventoryTool::Huggingface,
            max_depth: 20,
            stale_after_days: 0,
        })
        .expect("inventory should succeed");

        assert_eq!(report.summary.stale_assets, 0);
        assert_eq!(report.assets[0].reclaim_confidence, 35);
        assert_eq!(report.assets[0].action, "report-only");
    }

    #[test]
    fn stale_cutoff_is_strictly_metadata_based() {
        let now = SystemTime::now();
        let old = now
            .checked_sub(Duration::from_secs(91 * 86_400))
            .expect("old timestamp should exist");

        assert!(is_stale(Some(old), now));
        assert!(!is_stale(Some(now), now));
        assert!(!is_stale(None, now));
    }
}
