use assert_cmd::Command;

/// Captured from `c-static-analyzer scan examples/sample_issues.c
/// --no-config` against the Python implementation — verifies the Rust port
/// is byte-for-byte equivalent on the project's reference fixture.
#[test]
fn matches_python_output_on_sample_issues() {
    let expected_stdout = "\
examples/sample_issues.c:3: SA001 Function `complex_calc` has cyclomatic complexity 12 (threshold 10)
examples/sample_issues.c:18: SA004 Function `classify` may not return a value on all code paths
examples/sample_issues.c:31: SA003 Control flow nested 5 levels deep (threshold 4)
examples/sample_issues.c:41: SA002 Local variable `unused` is assigned but never used
examples/sample_issues.c:45: SA005 Unreachable code after `return`
";
    let expected_stderr = "\n5 issue(s) found.\n";

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let assert = Command::cargo_bin("c-static-analyzer")
        .unwrap()
        .current_dir(manifest_dir)
        .args(["scan", "examples/sample_issues.c", "--no-config"])
        .assert()
        .code(1);

    let output = assert.get_output();
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected_stdout);
    assert_eq!(String::from_utf8_lossy(&output.stderr), expected_stderr);
}
