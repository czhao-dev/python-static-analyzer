use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn scan_clean_file_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("clean.py");
    std::fs::write(&file, "def add(a, b):\n    return a + b\n").unwrap();

    Command::cargo_bin("static-analyzer")
        .unwrap()
        .args(["scan", file.to_str().unwrap(), "--no-config"])
        .assert()
        .code(0)
        .stdout(predicate::str::is_empty());
}

#[test]
fn scan_file_with_issues_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bad.py");
    std::fs::write(
        &file,
        "def add_item(item, items=[]):\n    items.append(item)\n    return items\n",
    )
    .unwrap();

    Command::cargo_bin("static-analyzer")
        .unwrap()
        .args(["scan", file.to_str().unwrap(), "--no-config"])
        .assert()
        .code(1)
        .stdout(predicate::str::contains("SA001"))
        .stdout(predicate::str::contains(file.to_str().unwrap()));
}

#[test]
fn select_filters_rules() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bad.py");
    std::fs::write(
        &file,
        "def add_item(item, items=[]):\n    items.append(item)\n    return items\n",
    )
    .unwrap();

    Command::cargo_bin("static-analyzer")
        .unwrap()
        .args([
            "scan",
            file.to_str().unwrap(),
            "--no-config",
            "--select",
            "SA005",
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::is_empty());
}

#[test]
fn missing_path_exits_two() {
    Command::cargo_bin("static-analyzer")
        .unwrap()
        .args(["scan", "/no/such/path.py", "--no-config"])
        .assert()
        .code(2);
}
