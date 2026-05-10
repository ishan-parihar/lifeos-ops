#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::fs;

/// Parsed vault file with frontmatter + body
pub struct VaultFile {
    pub frontmatter: serde_yaml::Mapping,
    pub body: String,
}

/// Diff kind for frontmatter/body changes
#[derive(Debug, Clone, PartialEq)]
pub enum DiffKind {
    Added,
    Removed,
    Changed,
    Unchanged,
    Conflict,
}

/// A single diff hunk
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub kind: DiffKind,
    pub location: String,    // property name or "body:block-N"
    pub base: Option<String>,
    pub local: Option<String>,
    pub remote: Option<String>,
}

/// Merge conflict
#[derive(Debug, Clone)]
pub struct Conflict {
    pub location: String,
    pub base: String,
    pub local: String,
    pub remote: String,
}

/// Merge result
pub struct MergeResult {
    pub has_conflicts: bool,
    pub merged_frontmatter: serde_yaml::Mapping,
    pub merged_body: String,
    pub conflicts: Vec<Conflict>,
}

/// Parse a vault file into frontmatter + body
pub fn parse_vault_file(content: &str) -> Result<(serde_yaml::Mapping, String), String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---\n") && !trimmed.starts_with("---\r\n") {
        return Ok((serde_yaml::Mapping::new(), trimmed.to_string()));
    }

    let after_first = trimmed.strip_prefix("---\n").or_else(|| trimmed.strip_prefix("---\r\n")).unwrap();
    let end = after_first.find("\n---\n")
        .or_else(|| after_first.find("\n---\r\n"))
        .ok_or_else(|| "Malformed frontmatter: no closing ---".to_string())?;

    let yaml_str = &after_first[..end];
    let body = after_first[end + 5..].to_string();

    let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str)
        .map_err(|e| format!("Parse frontmatter YAML: {e}"))?;

    let mapping = match yaml {
        serde_yaml::Value::Mapping(m) => m,
        _ => return Err("Frontmatter is not a mapping".to_string()),
    };

    Ok((mapping, body.trim_start().to_string()))
}

/// Rebuild vault file content from frontmatter + body
pub fn make_vault_content(frontmatter: &serde_yaml::Mapping, body: &str) -> String {
    let yaml_str = serde_yaml::to_string(frontmatter).unwrap_or_default();
    format!("---\n{}---\n\n{}\n", yaml_str, body.trim())
}

/// Read a vault file from disk
pub fn read_vault_file(path: &Path) -> Result<(serde_yaml::Mapping, String), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Read {}: {e}", path.display()))?;
    parse_vault_file(&content)
}

/// Base snapshot path for a page
pub fn base_snapshot_path(vault_dir: &Path, page_id: &str) -> PathBuf {
    let base_dir = vault_dir.join(".vault.base");
    base_dir.join(format!("{}.md", page_id))
}

/// Store a base snapshot before overwriting vault file
pub fn store_base_snapshot(vault_dir: &Path, page_id: &str, content: &str) -> Result<(), String> {
    let path = base_snapshot_path(vault_dir, page_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Create base dir: {e}"))?;
    }
    fs::write(&path, content).map_err(|e| format!("Write base snapshot: {e}"))?;
    Ok(())
}

/// Read a base snapshot
pub fn read_base_snapshot(vault_dir: &Path, page_id: &str) -> Result<String, String> {
    let path = base_snapshot_path(vault_dir, page_id);
    fs::read_to_string(&path).map_err(|e| format!("Read base snapshot: {e}"))
}

/// Delete a base snapshot after successful push
pub fn delete_base_snapshot(vault_dir: &Path, page_id: &str) -> Result<(), String> {
    let path = base_snapshot_path(vault_dir, page_id);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Delete base snapshot: {e}"))?;
    }
    Ok(())
}

// ── Frontmatter diff ─────────────────────────────────────────────

/// Property-by-property diff between two YAML mappings
pub fn diff_frontmatter(
    base: &serde_yaml::Mapping,
    local: &serde_yaml::Mapping,
) -> Vec<DiffHunk> {
    let mut hunks = Vec::new();

    for (key, base_val) in base {
        let loc = key_as_string(key);
        match local.get(key) {
            None => {
                hunks.push(DiffHunk {
                    kind: DiffKind::Removed,
                    location: loc,
                    base: Some(val_as_string(base_val)),
                    local: None,
                    remote: None,
                });
            }
            Some(lv) if lv != base_val => {
                hunks.push(DiffHunk {
                    kind: DiffKind::Changed,
                    location: loc,
                    base: Some(val_as_string(base_val)),
                    local: Some(val_as_string(lv)),
                    remote: None,
                });
            }
            _ => {} // unchanged
        }
    }

    for (key, local_val) in local {
        if !base.contains_key(key) {
            let loc = key_as_string(key);
            hunks.push(DiffHunk {
                kind: DiffKind::Added,
                location: loc,
                base: None,
                local: Some(val_as_string(local_val)),
                remote: None,
            });
        }
    }

    hunks
}

// ── Body diff via LCS ────────────────────────────────────────────

/// Block-level diff of body content (split on double newline)
pub fn diff_body(base: &str, local: &str) -> Vec<DiffHunk> {
    let base_blocks: Vec<&str> = split_blocks(base);
    let local_blocks: Vec<&str> = split_blocks(local);
    let lcs = compute_lcs(&base_blocks, &local_blocks);

    let mut hunks = Vec::new();
    let mut bi = 0usize;
    let mut li = 0usize;

    for &lcs_block in &lcs {
        while bi < base_blocks.len() && base_blocks[bi] != lcs_block {
            hunks.push(DiffHunk {
                kind: DiffKind::Removed,
                location: format!("body:block-{}", bi + 1),
                base: Some(base_blocks[bi].to_string()),
                local: None,
                remote: None,
            });
            bi += 1;
        }
        while li < local_blocks.len() && local_blocks[li] != lcs_block {
            hunks.push(DiffHunk {
                kind: DiffKind::Added,
                location: format!("body:block-{}", li + 1),
                base: None,
                local: Some(local_blocks[li].to_string()),
                remote: None,
            });
            li += 1;
        }
        if bi < base_blocks.len() {
            hunks.push(DiffHunk {
                kind: DiffKind::Unchanged,
                location: format!("body:block-{}", bi + 1),
                base: Some(base_blocks[bi].to_string()),
                local: None,
                remote: None,
            });
            bi += 1;
            li += 1;
        }
    }

    while bi < base_blocks.len() {
        hunks.push(DiffHunk {
            kind: DiffKind::Removed,
            location: format!("body:block-{}", bi + 1),
            base: Some(base_blocks[bi].to_string()),
            local: None,
            remote: None,
        });
        bi += 1;
    }
    while li < local_blocks.len() {
        hunks.push(DiffHunk {
            kind: DiffKind::Added,
            location: format!("body:block-{}", li + 1),
            base: None,
            local: Some(local_blocks[li].to_string()),
            remote: None,
        });
        li += 1;
    }

    hunks
}

fn split_blocks(s: &str) -> Vec<&str> {
    // Split on double newline to get Notion-block-aligned segments
    s.split("\n\n").collect()
}

fn compute_lcs<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<&'a str> {
    let m = a.len();
    let n = b.len();
    if m == 0 || n == 0 {
        return Vec::new();
    }
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1] + 1
            } else {
                dp[i - 1][j].max(dp[i][j - 1])
            };
        }
    }

    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(a[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}

// ── Three-way merge: frontmatter ─────────────────────────────────

/// 3-way merge for YAML frontmatter (property-by-property)
pub fn merge_frontmatter(
    base: &serde_yaml::Mapping,
    local: &serde_yaml::Mapping,
    remote: &serde_yaml::Mapping,
) -> (serde_yaml::Mapping, Vec<Conflict>) {
    let mut merged = serde_yaml::Mapping::new();
    let mut conflicts = Vec::new();

    let mut all_keys: Vec<&serde_yaml::Value> = Vec::new();
    for k in base.keys().chain(local.keys()).chain(remote.keys()) {
        if !all_keys.contains(&k) {
            all_keys.push(k);
        }
    }

    for key in all_keys {
        let b = base.get(key);
        let l = local.get(key);
        let r = remote.get(key);

        let loc = format!("frontmatter.{}", key_as_string(key));

        match (b, l, r) {
            // Only in one place
            (None, Some(lv), _) | (_, Some(lv), None) if b.is_none() || r.is_none() => {
                merged.insert(key.clone(), lv.clone());
            }
            (None, None, Some(rv)) => {
                merged.insert(key.clone(), rv.clone());
            }
            // All three same
            (Some(_bv), Some(lv), Some(rv)) if lv == rv => {
                merged.insert(key.clone(), lv.clone());
            }
            // Only one side diverged from base
            (Some(bv), Some(lv), Some(rv)) if lv == bv && rv != bv => {
                merged.insert(key.clone(), rv.clone());
            }
            (Some(bv), Some(lv), Some(rv)) if rv == bv && lv != bv => {
                merged.insert(key.clone(), lv.clone());
            }
            // Both changed differently → conflict
            (Some(bv), Some(lv), Some(rv)) => {
                conflicts.push(Conflict {
                    location: loc,
                    base: val_as_string(bv),
                    local: val_as_string(lv),
                    remote: val_as_string(rv),
                });
                merged.insert(key.clone(), lv.clone());
            }
            // Deleted in one, changed in other
            (Some(bv), None, Some(rv)) if rv != bv => {
                conflicts.push(Conflict {
                    location: loc,
                    base: val_as_string(bv),
                    local: "(deleted)".to_string(),
                    remote: val_as_string(rv),
                });
            }
            (Some(_), Some(lv), None) => {
                merged.insert(key.clone(), lv.clone());
            }
            // Deleted in both
            (Some(_), None, None) => {}
            // Catch-all: prefer local
            (_, Some(lv), _) => {
                merged.insert(key.clone(), lv.clone());
            }
            (_, _, Some(rv)) => {
                merged.insert(key.clone(), rv.clone());
            }
            _ => {}
        }
    }

    (merged, conflicts)
}

// ── Three-way merge: body via git merge-file ─────────────────────

/// 3-way merge for body content using `git merge-file`
pub fn merge_body(base: &str, local: &str, remote: &str) -> Result<(String, Vec<Conflict>), String> {
    let tmp_dir = std::env::temp_dir().join(format!("lifeos-merge-{}", std::process::id()));
    fs::create_dir_all(&tmp_dir).map_err(|e| format!("Create tmp dir: {e}"))?;

    let base_path = tmp_dir.join("base.md");
    let local_path = tmp_dir.join("local.md");
    let remote_path = tmp_dir.join("remote.md");

    fs::write(&base_path, base).map_err(|e| format!("Write base tmp: {e}"))?;
    fs::write(&local_path, local).map_err(|e| format!("Write local tmp: {e}"))?;
    fs::write(&remote_path, remote).map_err(|e| format!("Write remote tmp: {e}"))?;

    let local_str = local_path.to_string_lossy().to_string();
    let base_str = base_path.to_string_lossy().to_string();
    let remote_str = remote_path.to_string_lossy().to_string();

    let output = std::process::Command::new("git")
        .args(["merge-file", "--diff3", &local_str, &base_str, &remote_str])
        .output()
        .map_err(|e| format!("git merge-file failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            tracing::warn!("git merge-file stderr: {stderr}");
        }
        // Non-zero exit is normal when conflicts exist
    }

    let result = fs::read_to_string(&local_path).map_err(|e| format!("Read merged: {e}"))?;
    let conflicts = extract_conflicts(&result);

    // Cleanup
    let _ = fs::remove_file(&base_path);
    let _ = fs::remove_file(&local_path);
    let _ = fs::remove_file(&remote_path);
    let _ = fs::remove_dir(&tmp_dir);

    Ok((result, conflicts))
}

fn extract_conflicts(content: &str) -> Vec<Conflict> {
    let mut conflicts = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].starts_with("<<<<<<<") {
            let conflict_start = i;
            i += 1;

            let mut local_lines: Vec<&str> = Vec::new();
            let mut base_lines: Vec<&str> = Vec::new();
            let mut remote_lines: Vec<&str> = Vec::new();

            while i < lines.len() && !lines[i].starts_with("|||||||") {
                local_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1; // skip |||||||
            }

            while i < lines.len() && !lines[i].starts_with("=======") {
                base_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1; // skip =======
            }

            while i < lines.len() && !lines[i].starts_with(">>>>>>>") {
                remote_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1; // skip >>>>>>>
            }

            conflicts.push(Conflict {
                location: format!("body:lines {}-{}", conflict_start + 1, i),
                base: base_lines.join("\n"),
                local: local_lines.join("\n"),
                remote: remote_lines.join("\n"),
            });
        } else {
            i += 1;
        }
    }

    conflicts
}

// ── Full 3-way merge ─────────────────────────────────────────────

/// Run full 3-way merge on a vault file
pub fn three_way_merge(
    base: &VaultFile,
    local: &VaultFile,
    remote: &VaultFile,
) -> Result<MergeResult, String> {
    let (merged_fm, fm_conflicts) = merge_frontmatter(&base.frontmatter, &local.frontmatter, &remote.frontmatter);
    let (merged_body, body_conflicts) = merge_body(&base.body, &local.body, &remote.body)?;

    let mut all_conflicts = fm_conflicts;
    all_conflicts.extend(body_conflicts);

    Ok(MergeResult {
        has_conflicts: !all_conflicts.is_empty(),
        merged_frontmatter: merged_fm,
        merged_body,
        conflicts: all_conflicts,
    })
}

// ── Helpers ──────────────────────────────────────────────────────

fn key_as_string(key: &serde_yaml::Value) -> String {
    key.as_str().map(|s| s.to_string()).unwrap_or_else(|| format!("{key:?}"))
}

fn val_as_string(val: &serde_yaml::Value) -> String {
    match val {
        serde_yaml::Value::Null => "null".to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Sequence(seq) => {
            let items: Vec<String> = seq.iter().map(val_as_string).collect();
            format!("[{}]", items.join(", "))
        }
        serde_yaml::Value::Mapping(m) => {
            let items: Vec<String> = m.iter().map(|(k, v)| {
                format!("{}: {}", val_as_string(k), val_as_string(v))
            }).collect();
            format!("{{{}}}", items.join(", "))
        }
        _ => format!("{val:?}"),
    }
}

/// Format diff hunks as human-readable text
pub fn format_diff(hunks: &[DiffHunk]) -> String {
    let mut out = String::new();
    for hunk in hunks {
        match hunk.kind {
            DiffKind::Added => {
                out.push_str(&format!("+ ADDED    {}\n", hunk.location));
                if let Some(ref l) = hunk.local {
                    for line in l.lines() {
                        out.push_str(&format!("  + {line}\n"));
                    }
                }
            }
            DiffKind::Removed => {
                out.push_str(&format!("- REMOVED  {}\n", hunk.location));
                if let Some(ref b) = hunk.base {
                    for line in b.lines() {
                        out.push_str(&format!("  - {line}\n"));
                    }
                }
            }
            DiffKind::Changed => {
                out.push_str(&format!("~ CHANGED  {}\n", hunk.location));
                if let Some(ref b) = hunk.base {
                    out.push_str("  - (was)\n");
                    for line in b.lines() {
                        out.push_str(&format!("    {line}\n"));
                    }
                }
                if let Some(ref l) = hunk.local {
                    out.push_str("  + (now)\n");
                    for line in l.lines() {
                        out.push_str(&format!("    {line}\n"));
                    }
                }
            }
            DiffKind::Unchanged => {}
            DiffKind::Conflict => {
                out.push_str(&format!("! CONFLICT {}\n", hunk.location));
                if let Some(ref b) = hunk.base {
                    out.push_str("  base:\n");
                    for line in b.lines() {
                        out.push_str(&format!("    {line}\n"));
                    }
                }
                if let Some(ref l) = hunk.local {
                    out.push_str("  local:\n");
                    for line in l.lines() {
                        out.push_str(&format!("    {line}\n"));
                    }
                }
                if let Some(ref r) = hunk.remote {
                    out.push_str("  remote:\n");
                    for line in r.lines() {
                        out.push_str(&format!("    {line}\n"));
                    }
                }
            }
        }
    }
    out
}

/// Format conflicts as human-readable text
pub fn format_conflicts(conflicts: &[Conflict]) -> String {
    if conflicts.is_empty() {
        return "  No conflicts.\n".to_string();
    }
    let mut out = format!("! {} conflict(s):\n", conflicts.len());
    for (i, c) in conflicts.iter().enumerate() {
        out.push_str(&format!("\n  Conflict #{}: {}\n", i + 1, c.location));
        out.push_str(&format!("    base:   {}\n", truncate(&c.base, 80)));
        out.push_str(&format!("    local:  {}\n", truncate(&c.local, 80)));
        out.push_str(&format!("    remote: {}\n", truncate(&c.remote, 80)));
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
