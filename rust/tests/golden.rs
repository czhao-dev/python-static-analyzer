use assert_cmd::Command;

/// Captured from `python -m static_analyzer scan examples/sample_issues.py
/// --no-config` against the original Python implementation — verifies the
/// Rust port is byte-for-byte equivalent on the project's reference fixture.
#[test]
fn matches_python_output_on_sample_issues() {
    let expected_stdout = "\
examples/sample_issues.py:1: SA002 Unused import `json`
examples/sample_issues.py:2: SA002 Unused import `os`
examples/sample_issues.py:5: SA001 Mutable default argument `values=[]`
examples/sample_issues.py:9: SA003 Broad exception handler `except Exception`
examples/sample_issues.py:13: SA004 Parameter `list` shadows a built-in name
examples/sample_issues.py:14: SA006 Local variable `unused` is assigned but never used
examples/sample_issues.py:18: SA008 Function `classify` may not return a value on all code paths
examples/sample_issues.py:23: SA008 Function `first_even` may not return a value on all code paths
examples/sample_issues.py:27: SA009 Unreachable code after `return`
";
    let expected_stderr = "\n9 issue(s) found.\n";

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let assert = Command::cargo_bin("static-analyzer")
        .unwrap()
        .current_dir(manifest_dir)
        .args(["scan", "examples/sample_issues.py", "--no-config"])
        .assert()
        .code(1);

    let output = assert.get_output();
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected_stdout);
    assert_eq!(String::from_utf8_lossy(&output.stderr), expected_stderr);
}
