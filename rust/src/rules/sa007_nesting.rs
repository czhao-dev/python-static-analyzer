use rustpython_ruff_python_ast::visitor::{walk_body, walk_stmt, Visitor};
use rustpython_ruff_python_ast::{ExceptHandler, ModModule, Stmt};
use rustpython_ruff_source_file::SourceCode;
use rustpython_ruff_text_size::{Ranged, TextSize};

use super::Rule;
use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::visitor::loc;

pub struct Nesting;

impl Nesting {
    pub const RULE_ID: &'static str = "SA007";
}

struct Scanner<'a, 'src, 'idx> {
    path: &'a str,
    source: &'a SourceCode<'src, 'idx>,
    threshold: i64,
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'src, 'idx> Scanner<'a, 'src, 'idx> {
    fn maybe_report(&mut self, offset: TextSize, depth: i64, reported: &mut bool) {
        if depth > self.threshold && !*reported {
            *reported = true;
            let (line, col) = loc(self.source, offset);
            self.diagnostics.push(Diagnostic {
                path: self.path.to_string(),
                line,
                col,
                rule_id: Nesting::RULE_ID,
                message: format!(
                    "Control flow nested {depth} levels deep (threshold {})",
                    self.threshold
                ),
            });
        }
    }

    /// `Stmt::FunctionDef` is intentionally not skipped here: with no match
    /// arm for it, it (and anything else that isn't If/For/While/With/Try —
    /// e.g. ClassDef) simply isn't recursed into, matching nesting.py's
    /// `_NESTED_SCOPES = (FunctionDef, AsyncFunctionDef, Lambda)` walk_block,
    /// which only recurses through control-flow statement kinds.
    fn walk_block(&mut self, stmts: &[Stmt], depth: i64, reported: &mut bool) {
        for stmt in stmts {
            match stmt {
                Stmt::If(s) => {
                    let new_depth = depth + 1;
                    self.maybe_report(s.range().start(), new_depth, reported);
                    self.walk_block(&s.body, new_depth, reported);
                    for clause in &s.elif_else_clauses {
                        // Ruff already distinguishes elif (test: Some) from a
                        // genuine else (test: None) structurally, so every
                        // clause's body is walked at the same `new_depth` as
                        // the if's own body — no col_offset heuristic needed.
                        self.walk_block(&clause.body, new_depth, reported);
                    }
                }
                Stmt::For(s) => {
                    let new_depth = depth + 1;
                    self.maybe_report(s.range().start(), new_depth, reported);
                    self.walk_block(&s.body, new_depth, reported);
                    self.walk_block(&s.orelse, new_depth, reported);
                }
                Stmt::While(s) => {
                    let new_depth = depth + 1;
                    self.maybe_report(s.range().start(), new_depth, reported);
                    self.walk_block(&s.body, new_depth, reported);
                    self.walk_block(&s.orelse, new_depth, reported);
                }
                Stmt::With(s) => {
                    let new_depth = depth + 1;
                    self.maybe_report(s.range().start(), new_depth, reported);
                    self.walk_block(&s.body, new_depth, reported);
                }
                Stmt::Try(s) => {
                    let new_depth = depth + 1;
                    self.maybe_report(s.range().start(), new_depth, reported);
                    self.walk_block(&s.body, new_depth, reported);
                    for handler in &s.handlers {
                        let ExceptHandler::ExceptHandler(handler) = handler;
                        self.walk_block(&handler.body, new_depth, reported);
                    }
                    self.walk_block(&s.orelse, new_depth, reported);
                    self.walk_block(&s.finalbody, new_depth, reported);
                }
                _ => {}
            }
        }
    }
}

struct FunctionFinder<'b, 'a, 'src, 'idx> {
    scanner: &'b mut Scanner<'a, 'src, 'idx>,
}

impl<'s, 'b, 'a, 'src, 'idx> Visitor<'s> for FunctionFinder<'b, 'a, 'src, 'idx> {
    fn visit_stmt(&mut self, stmt: &'s Stmt) {
        if let Stmt::FunctionDef(func) = stmt {
            let mut reported = false;
            self.scanner.walk_block(&func.body, 0, &mut reported);
        }
        walk_stmt(self, stmt);
    }
}

impl Rule for Nesting {
    fn id(&self) -> &'static str {
        Self::RULE_ID
    }

    fn check(
        &self,
        tree: &ModModule,
        path: &str,
        source: &SourceCode<'_, '_>,
        config: &Config,
    ) -> Vec<Diagnostic> {
        let mut scanner = Scanner {
            path,
            source,
            threshold: config.max_nesting,
            diagnostics: Vec::new(),
        };
        let mut finder = FunctionFinder {
            scanner: &mut scanner,
        };
        walk_body(&mut finder, &tree.body);
        scanner.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use rustpython_ruff_source_file::LineIndex;

    fn check(source: &str, config: &Config) -> Vec<Diagnostic> {
        let tree = rustpython_ruff_python_parser::parse_module(source)
            .unwrap()
            .into_syntax();
        let index = LineIndex::from_source_text(source);
        let source_code = SourceCode::new(source, &index);
        Nesting.check(&tree, "example.py", &source_code, config)
    }

    #[test]
    fn shallow_nesting_is_not_flagged() {
        let diagnostics = check(
            "def f(x):\n    if x:\n        return 1\n    return 0\n",
            &Config::default(),
        );
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn flags_deep_nesting() {
        let config = Config {
            max_nesting: 2,
            ..Config::default()
        };
        let source =
            "def f(x):\n    if x:\n        for i in range(x):\n            while i > 0:\n                i -= 1\n";
        let diagnostics = check(source, &config);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "SA007");
    }

    #[test]
    fn elif_chain_does_not_count_as_nesting() {
        let config = Config {
            max_nesting: 1,
            ..Config::default()
        };
        let source = "def f(x):\n    if x == 1:\n        return \"one\"\n    elif x == 2:\n        return \"two\"\n    elif x == 3:\n        return \"three\"\n    else:\n        return \"other\"\n";
        let diagnostics = check(source, &config);
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn else_with_nested_if_does_count() {
        let config = Config {
            max_nesting: 1,
            ..Config::default()
        };
        let source = "def f(x, y):\n    if x:\n        return 1\n    else:\n        if y:\n            return 2\n";
        let diagnostics = check(source, &config);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn reports_only_once_per_function() {
        let config = Config {
            max_nesting: 1,
            ..Config::default()
        };
        let source =
            "def f(x):\n    if x:\n        if x:\n            if x:\n                return 1\n";
        let diagnostics = check(source, &config);
        assert_eq!(diagnostics.len(), 1);
    }
}
