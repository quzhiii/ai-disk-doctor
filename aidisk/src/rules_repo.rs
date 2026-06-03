use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

pub fn default_rules_repo_cache_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".aidisk")
        .join("rules-repos")
}

pub fn resolve_rules_repo(source: &str, cache_root: &Path) -> Result<PathBuf> {
    let source_path = PathBuf::from(source);
    let repo_root = if source_path.exists() {
        source_path
    } else {
        validate_rules_repo_url(source)?;
        clone_or_reuse_rules_repo(source, cache_root)?
    };

    resolve_rules_dir_from_repo(&repo_root)
}

pub fn validate_rules_repo_url(source: &str) -> Result<()> {
    if !source.starts_with("https://") {
        anyhow::bail!("rules repo URL must use https://");
    }

    let host = source
        .trim_start_matches("https://")
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let host = host.split('@').next_back().unwrap_or(&host);
    let host = host.split(':').next().unwrap_or(host);

    if host.is_empty()
        || matches!(host, "localhost" | "127.0.0.1" | "0.0.0.0" | "::1")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || is_private_172_host(host)
        || host.starts_with("169.254.")
    {
        anyhow::bail!("rules repo URL host is not allowed");
    }

    Ok(())
}

fn clone_or_reuse_rules_repo(source: &str, cache_root: &Path) -> Result<PathBuf> {
    fs::create_dir_all(cache_root)
        .with_context(|| format!("failed to create rules repo cache {}", cache_root.display()))?;
    let repo_root = cache_root.join(cache_key(source));
    if repo_root.exists() {
        return Ok(repo_root);
    }

    let status = Command::new("git")
        .args(["clone", "--depth", "1", source])
        .arg(&repo_root)
        .status()
        .with_context(|| "failed to run git clone for rules repo")?;

    if !status.success() {
        anyhow::bail!("git clone failed for rules repo {source}");
    }

    Ok(repo_root)
}

fn resolve_rules_dir_from_repo(repo_root: &Path) -> Result<PathBuf> {
    if !repo_root.is_dir() {
        anyhow::bail!(
            "rules repo source is not a directory: {}",
            repo_root.display()
        );
    }

    let nested_rules = repo_root.join("rules");
    if nested_rules.is_dir() {
        return Ok(nested_rules);
    }

    Ok(repo_root.to_path_buf())
}

fn cache_key(source: &str) -> String {
    let mut key = source
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    while key.contains("--") {
        key = key.replace("--", "-");
    }
    key.trim_matches('-').to_ascii_lowercase()
}

fn is_private_172_host(host: &str) -> bool {
    let Some(rest) = host.strip_prefix("172.") else {
        return false;
    };
    let Some(octet) = rest
        .split('.')
        .next()
        .and_then(|value| value.parse::<u8>().ok())
    else {
        return false;
    };
    (16..=31).contains(&octet)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{resolve_rules_repo, validate_rules_repo_url};

    #[test]
    fn resolves_local_repo_rules_subdirectory() {
        let temp = tempdir().expect("tempdir should exist");
        let repo = temp.path().join("community-rules");
        let rules_dir = repo.join("rules");
        fs::create_dir_all(&rules_dir).expect("rules dir should be created");

        let resolved = resolve_rules_repo(repo.to_str().unwrap(), temp.path())
            .expect("local repo should resolve");

        assert_eq!(resolved, rules_dir);
    }

    #[test]
    fn resolves_local_repo_root_when_rules_subdirectory_is_absent() {
        let temp = tempdir().expect("tempdir should exist");
        let repo = temp.path().join("community-rules");
        fs::create_dir_all(&repo).expect("repo dir should be created");
        fs::write(repo.join("sample.yaml"), "id: demo").expect("sample yaml should be written");

        let resolved = resolve_rules_repo(repo.to_str().unwrap(), temp.path())
            .expect("local repo root should resolve");

        assert_eq!(resolved, repo);
    }

    #[test]
    fn validates_https_rules_repo_urls() {
        validate_rules_repo_url("https://github.com/example/rules.git")
            .expect("https github url should be allowed");
    }

    #[test]
    fn rejects_unsafe_rules_repo_urls() {
        for url in [
            "http://example.com/rules.git",
            "file:///C:/secret/rules.git",
            "https://localhost/rules.git",
            "https://127.0.0.1/rules.git",
            "https://192.168.1.2/rules.git",
        ] {
            assert!(
                validate_rules_repo_url(url).is_err(),
                "{url} should be rejected"
            );
        }
    }
}
