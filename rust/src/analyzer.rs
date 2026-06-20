use std::path::{Path, PathBuf};

use rustpython_ruff_source_file::{LineIndex, SourceCode};
use rustpython_ruff_text_size::Ranged;

use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::fnmatch::fnmatch;
use crate::rules::ALL_RULES;
use crate::visitor::loc;

pub const DEFAULT_EXCLUDE_DIRS: &[&str] = &[
    ".venv",
    "venv",
    ".git",
    "__pycache__",
    "build",
    "dist",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
    "node_modules",
];

pub fn is_excluded(path: &Path, patterns: &[String]) -> bool {
    let in_default_excluded_dir = path.components().any(|component| {
        let std::path::Component::Normal(part) = component else {
            return false;
        };
        let part = part.to_string_lossy();
        DEFAULT_EXCLUDE_DIRS.contains(&part.as_ref()) || part.ends_with(".egg-info")
    });
    if in_default_excluded_dir {
        return true;
    }
    let posix = path.to_string_lossy().replace('\\', "/");
    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();
    patterns
        .iter()
        .any(|pattern| fnmatch(&posix, pattern) || fnmatch(&filename, pattern))
}

fn collect_py_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_py_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "py") {
            out.push(path);
        }
    }
}

pub fn iter_python_files(paths: &[PathBuf], exclude: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for path in paths {
        if path.is_file() {
            if path.extension().is_some_and(|ext| ext == "py") && !is_excluded(path, exclude) {
                out.push(path.clone());
            }
            continue;
        }
        let mut found = Vec::new();
        collect_py_files(path, &mut found);
        found.sort();
        for candidate in found {
            if !is_excluded(&candidate, exclude) {
                out.push(candidate);
            }
        }
    }
    out
}

pub fn analyze_file(path: &Path, config: &Config) -> Vec<Diagnostic> {
    let path_str = path.to_string_lossy().to_string();

    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => {
            return vec![Diagnostic {
                path: path_str,
                line: 1,
                col: 0,
                rule_id: "SA000",
                message: format!("Could not read file: {err}"),
            }];
        }
    };

    let line_index = LineIndex::from_source_text(&source);
    let source_code = SourceCode::new(&source, &line_index);

    let tree = match rustpython_ruff_python_parser::parse_module(&source) {
        Ok(parsed) => parsed.into_syntax(),
        Err(err) => {
            let (line, col) = loc(&source_code, err.range().start());
            return vec![Diagnostic {
                path: path_str,
                line,
                col: col + 1,
                rule_id: "SA000",
                message: format!("Syntax error: {}", err.error),
            }];
        }
    };

    let mut diagnostics = Vec::new();
    for rule in ALL_RULES {
        if !config.is_enabled(rule.id()) {
            continue;
        }
        diagnostics.extend(rule.check(&tree, &path_str, &source_code, config));
    }
    diagnostics
}

pub fn analyze_paths(paths: &[PathBuf], config: &Config) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for file_path in iter_python_files(paths, &config.exclude) {
        diagnostics.extend(analyze_file(&file_path, config));
    }
    diagnostics.sort();
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_default_dirs() {
        let path = PathBuf::from("project/.venv/lib/site-packages/foo.py");
        assert!(is_excluded(&path, &[]));
    }

    #[test]
    fn excludes_user_glob_patterns() {
        // fnmatch is a full (anchored) match, so the pattern must match the
        // entire posix path or the bare filename, not just a substring.
        let path = PathBuf::from("tests/test_foo.py");
        assert!(is_excluded(&path, &["tests/*".to_string()]));
        assert!(!is_excluded(&path, &["other/*".to_string()]));
        assert!(is_excluded(&path, &["test_*.py".to_string()]));
    }

    #[test]
    fn iter_python_files_finds_py_files_sorted() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("b.py"), "").unwrap();
        std::fs::write(dir.path().join("a.py"), "").unwrap();
        std::fs::write(dir.path().join("c.txt"), "").unwrap();
        let found = iter_python_files(&[dir.path().to_path_buf()], &[]);
        assert_eq!(
            found,
            vec![dir.path().join("a.py"), dir.path().join("b.py")]
        );
    }
}
