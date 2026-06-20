use static_analyzer::analyzer::iter_python_files;
use static_analyzer::config::Config;

#[test]
fn default_excludes_skip_venv_and_vcs_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("src")).unwrap();
    std::fs::write(tmp.path().join("src").join("app.py"), "x = 1\n").unwrap();
    let venv_pkg = tmp.path().join(".venv").join("lib").join("somepkg");
    std::fs::create_dir_all(&venv_pkg).unwrap();
    std::fs::write(venv_pkg.join("module.py"), "y = 2\n").unwrap();

    let config = Config::default();
    let mut found: Vec<String> = iter_python_files(&[tmp.path().to_path_buf()], &config.exclude)
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    found.sort();

    assert_eq!(found, vec!["app.py".to_string()]);
}

#[test]
fn custom_exclude_pattern() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("keep.py"), "x = 1\n").unwrap();
    std::fs::write(tmp.path().join("generated.py"), "x = 1\n").unwrap();

    let mut found: Vec<String> =
        iter_python_files(&[tmp.path().to_path_buf()], &["*generated*".to_string()])
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
    found.sort();

    assert_eq!(found, vec!["keep.py".to_string()]);
}
