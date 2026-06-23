# C Static Analyzer

A lightweight C static analyzer for finding common code quality, correctness, and maintainability issues before runtime.

It parses `.c`/`.h` files with [tree-sitter](https://tree-sitter.github.io/tree-sitter/) and the `tree-sitter-c` grammar (no compilation or execution of your code), reports file-and-line diagnostics with stable rule IDs, and exits non-zero when it finds something — so it works as a local check or a CI gate. It builds to a single, dependency-free binary.

## Implemented Checks

| Rule    | Description |
|---------|--------------|
| `SA001` | Function with high cyclomatic complexity. |
| `SA002` | Unused local variable. |
| `SA003` | Deeply nested control flow. |
| `SA004` | Missing return path in a non-void function. |
| `SA005` | Unreachable code after `return`, `break`, `continue`, or `goto`. |

## Example

Given this C file:

```c
const char *classify(int x) {
    if (x > 0) {
        return "positive";
    } else if (x < 0) {
        return "negative";
    }
}
```

The analyzer reports:

```text
example.c:1: SA004 Function `classify` may not return a value on all code paths
```

## Installation

```bash
cargo build --release
# binary at ./target/release/c-static-analyzer
```

## Usage

```bash
c-static-analyzer scan path/to/project
```

Example output:

```text
src/app.c:12: SA001 Function `parse_request` has cyclomatic complexity 14 (threshold 10)
src/app.c:34: SA002 Local variable `unused` is assigned but never used
src/util.c:48: SA004 Function `convert` may not return a value on all code paths
```

The command exits `0` when no issues are found, `1` when diagnostics are reported, and `2` on a usage error (e.g. a path that doesn't exist) — making it suitable as a CI gate.

By default the scanner skips common non-project directories (`.git`, `build`, `dist`, `cmake-build-debug`, `cmake-build-release`, `CMakeFiles`, `out`, `vendor`, `third_party`).

### CLI options

```text
c-static-analyzer scan [paths ...]
  --max-complexity N     Cyclomatic complexity threshold (default: 10)
  --max-nesting N        Control flow nesting depth threshold (default: 4)
  --select SA001,SA002   Only run these rule IDs (default: all rules)
  --exclude PATTERN       Glob pattern to exclude; repeatable
  --no-config             Ignore .c-static-analyzer.toml configuration
```

## Configuration

Settings can be set on the command line or in a `.c-static-analyzer.toml` file in (or above) the directory you're scanning from:

```toml
exclude = ["tests/fixtures/*"]
max_complexity = 10
max_nesting = 4
enabled_rules = ["SA001", "SA002", "SA004"]
```

`enabled_rules` defaults to an empty list, which means all rules are enabled. CLI flags override the values loaded from `.c-static-analyzer.toml`.

## Development

Project structure:

```text
c-static-analyzer/
├── README.md
├── Cargo.toml
├── examples/
│   └── sample_issues.c
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli.rs
│   ├── analyzer.rs
│   ├── visitor.rs
│   ├── config.rs
│   ├── diagnostics.rs
│   ├── fnmatch.rs
│   └── rules/
│       ├── mod.rs
│       ├── sa001_complexity.rs
│       ├── sa002_unused_variables.rs
│       ├── sa003_nesting.rs
│       ├── sa004_missing_return.rs
│       └── sa005_unreachable_code.rs
└── tests/
    ├── analyzer.rs
    ├── cli.rs
    └── golden.rs
```

Development commands:

```bash
cargo build --release
cargo test
./target/release/c-static-analyzer scan examples/
```

## Design

The analyzer uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/) with the `tree-sitter-c` grammar via the [`tree-sitter`](https://crates.io/crates/tree-sitter) and [`tree-sitter-c`](https://crates.io/crates/tree-sitter-c) crates:

1. Parse each `.c`/`.h` file into a concrete syntax tree.
2. Run each enabled rule over the tree, walking `tree_sitter::Node`s with `child_by_field_name`.
3. Collect diagnostics with rule IDs, messages, file paths, and line numbers.
4. Sort and render results in a human-readable CLI format.
5. Exit with a non-zero status when findings are present, making the tool usable in CI.

Each rule lives in its own module under `src/rules/` and exposes a rule ID and a check function, so adding a new rule means adding one file and registering it in `rules/mod.rs`.

## Test Results

44 tests pass — 36 unit tests (rule logic, config loading, fnmatch, file discovery) plus 8 integration tests (CLI behavior and a byte-for-byte golden-output comparison against the project's reference fixture):

```text
$ cargo test
...
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (tests/analyzer.rs)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (tests/cli.rs)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (tests/golden.rs)
```

Running the analyzer on [examples/sample_issues.c](examples/sample_issues.c), a file written specifically to trigger every rule, confirms end-to-end behavior:

```text
$ c-static-analyzer scan examples/sample_issues.c
examples/sample_issues.c:3: SA001 Function `complex_calc` has cyclomatic complexity 12 (threshold 10)
examples/sample_issues.c:18: SA004 Function `classify` may not return a value on all code paths
examples/sample_issues.c:31: SA003 Control flow nested 5 levels deep (threshold 4)
examples/sample_issues.c:41: SA002 Local variable `unused` is assigned but never used
examples/sample_issues.c:45: SA005 Unreachable code after `return`

5 issue(s) found.
```

## Roadmap

- [x] Add project packaging with `Cargo.toml`.
- [x] Implement the CLI entry point.
- [x] Implement tree-sitter parsing for C files.
- [x] Add diagnostic formatting.
- [x] Add unit tests for each rule.
- [x] Add configuration support.
- [x] Add CI-friendly exit codes.
- [ ] Add JSON output for editor and automation integrations.

## License

This project is licensed under the terms in [LICENSE](LICENSE).
