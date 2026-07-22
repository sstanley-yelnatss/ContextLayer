//! Trace CI — validate PR artifacts against `.contextlayer/rules.yml`

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::redact::contains_secrets;
use crate::store::TraceStore;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TraceRules {
    #[serde(default = "default_true")]
    pub require_reasoning_export: bool,
    #[serde(default)]
    pub require_checkpoint: bool,
    #[serde(default = "default_true")]
    pub secrets_scan: bool,
    #[serde(default = "default_markers")]
    pub reasoning_markers: Vec<String>,
    #[serde(default = "default_reasoning_paths")]
    pub reasoning_paths: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_markers() -> Vec<String> {
    vec![
        "PR Reasoning:".into(),
        "Assumption:".into(),
        "Hypothesis:".into(),
        "Reasoning appendix by".into(),
        // Legacy footer text from exports before the site-link footer.
        "Exported from ContextLayer".into(),
    ]
}

fn default_reasoning_paths() -> Vec<String> {
    vec!["docs/reasoning/".into(), ".contextlayer/".into()]
}

impl Default for TraceRules {
    fn default() -> Self {
        Self {
            require_reasoning_export: true,
            require_checkpoint: false,
            secrets_scan: true,
            reasoning_markers: default_markers(),
            reasoning_paths: default_reasoning_paths(),
        }
    }
}

impl TraceRules {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_yaml::from_str(&text).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone, Default)]
pub struct TraceCheckInput {
    pub pr_body: String,
    pub repo_root: PathBuf,
    pub scan_globs: Vec<String>,
    pub workspace_id: Option<String>,
    pub trace_store: Option<TraceStore>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceCheckReport {
    pub passed: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn run_trace_check(rules: &TraceRules, input: &TraceCheckInput) -> TraceCheckReport {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if rules.secrets_scan {
        if contains_secrets(&input.pr_body) {
            errors.push("PR body matches secret patterns".into());
        }
    }

    let mut reasoning_found = false;

    if rules.require_reasoning_export {
        for marker in &rules.reasoning_markers {
            if input.pr_body.contains(marker.as_str()) {
                reasoning_found = true;
                break;
            }
        }

        if !reasoning_found {
            for prefix in &rules.reasoning_paths {
                if reasoning_file_exists(&input.repo_root, prefix) {
                    reasoning_found = true;
                    break;
                }
            }
        }

        if !reasoning_found {
            errors.push(
                "Missing reasoning export: add ContextLayer PR markdown to description or commit under docs/reasoning/".into(),
            );
        }
    }

    if rules.secrets_scan {
        for path in collect_scan_files(&input.repo_root, &input.scan_globs) {
            if let Ok(text) = fs::read_to_string(&path) {
                if contains_secrets(&text) {
                    errors.push(format!(
                        "Secret pattern in {}",
                        path.strip_prefix(&input.repo_root)
                            .unwrap_or(&path)
                            .display()
                    ));
                }
            }
        }
    }

    if rules.require_checkpoint {
        if let (Some(ws), Some(store)) = (&input.workspace_id, &input.trace_store) {
            match store.summary(ws) {
                Ok(s) if s.checkpoint_count > 0 => {}
                Ok(_) => errors.push(format!(
                    "require_checkpoint: no checkpoints in trace for workspace {ws}"
                )),
                Err(e) => errors.push(format!("trace read failed: {e}")),
            }
        } else {
            warnings.push(
                "require_checkpoint is true but workspace_id/trace_store not provided — skipped"
                    .into(),
            );
        }
    }

    TraceCheckReport {
        passed: errors.is_empty(),
        errors,
        warnings,
    }
}

fn reasoning_file_exists(repo_root: &Path, prefix: &str) -> bool {
    let dir = repo_root.join(prefix.trim_end_matches('/'));
    if !dir.is_dir() {
        return false;
    }
    fs::read_dir(&dir)
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                e.path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .is_some_and(|ext| ext == "md")
            })
        })
        .unwrap_or(false)
}

fn collect_scan_files(repo_root: &Path, globs: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if globs.is_empty() {
        scan_dir_recursive(&repo_root.join("docs/reasoning"), &mut out);
        return out;
    }
    for g in globs {
        let p = repo_root.join(g.trim_start_matches('/'));
        if p.is_file() {
            out.push(p);
        } else if p.is_dir() {
            scan_dir_recursive(&p, &mut out);
        }
    }
    out
}

fn scan_dir_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                scan_dir_recursive(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                out.push(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn fails_without_reasoning_marker() {
        let rules = TraceRules::default();
        let input = TraceCheckInput {
            pr_body: "Just a diff".into(),
            repo_root: PathBuf::from("."),
            ..Default::default()
        };
        let r = run_trace_check(&rules, &input);
        assert!(!r.passed);
    }

    #[test]
    fn passes_with_pr_reasoning_marker() {
        let rules = TraceRules::default();
        let input = TraceCheckInput {
            pr_body: "PR Reasoning: fix auth\n\nHypothesis: token stale".into(),
            repo_root: PathBuf::from("."),
            ..Default::default()
        };
        let r = run_trace_check(&rules, &input);
        assert!(r.passed);
    }

    #[test]
    fn passes_with_assumption_marker() {
        let rules = TraceRules::default();
        let input = TraceCheckInput {
            pr_body: "Assumption:\nRefresh token rotation is safe because tests pass".into(),
            repo_root: PathBuf::from("."),
            ..Default::default()
        };
        let r = run_trace_check(&rules, &input);
        assert!(r.passed);
    }

    #[test]
    fn passes_with_reasoning_file() {
        let dir = tempfile::tempdir().unwrap();
        let reasoning = dir.path().join("docs/reasoning");
        fs::create_dir_all(&reasoning).unwrap();
        let mut f = fs::File::create(reasoning.join("pr-1.md")).unwrap();
        write!(f, "Exported from ContextLayer").unwrap();

        let rules = TraceRules {
            require_reasoning_export: true,
            ..Default::default()
        };
        let input = TraceCheckInput {
            pr_body: String::new(),
            repo_root: dir.path().to_path_buf(),
            ..Default::default()
        };
        let r = run_trace_check(&rules, &input);
        assert!(r.passed);
    }
}
