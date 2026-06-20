use rustpython_ruff_python_ast::visitor::{
    walk_body, walk_comprehension, walk_except_handler, walk_expr, walk_stmt, Visitor,
};
use rustpython_ruff_python_ast::{Comprehension, ExceptHandler, Expr, ModModule, Stmt};
use rustpython_ruff_source_file::SourceCode;
use rustpython_ruff_text_size::Ranged;

use super::Rule;
use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::visitor::loc;

pub struct Complexity;

impl Complexity {
    pub const RULE_ID: &'static str = "SA005";
}

/// Own-scope (stops at nested FunctionDef/Lambda) complexity scorer.
struct ComplexityCounter {
    score: i64,
}

impl<'a> Visitor<'a> for ComplexityCounter {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if matches!(stmt, Stmt::FunctionDef(_)) {
            return;
        }
        match stmt {
            // CPython's ast represents each `elif` as its own nested `If`
            // node (each contributing +1), but ruff flattens elif/else into
            // `elif_else_clauses` with no separate node per elif — so count
            // the if itself plus one per elif clause (test: Some), excluding
            // a trailing plain `else` (test: None), which isn't a decision point.
            //
            // Traversal here is manual (not delegated to `walk_stmt`) because
            // ruff's generated `walk_stmt` for `Stmt::If` visits each elif
            // clause's test expression twice — once explicitly, then again
            // inside `walk_elif_else_clause` — which would double-count any
            // BoolOp/IfExp living in that test.
            Stmt::If(s) => {
                self.score += 1;
                self.visit_expr(&s.test);
                for inner in &s.body {
                    self.visit_stmt(inner);
                }
                for clause in &s.elif_else_clauses {
                    if let Some(test) = &clause.test {
                        self.score += 1;
                        self.visit_expr(test);
                    }
                    for inner in &clause.body {
                        self.visit_stmt(inner);
                    }
                }
                return;
            }
            Stmt::For(_) | Stmt::While(_) | Stmt::Assert(_) => self.score += 1,
            _ => {}
        }
        walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if matches!(expr, Expr::Lambda(_)) {
            return;
        }
        match expr {
            Expr::If(_) => self.score += 1,
            Expr::BoolOp(bool_op) => self.score += bool_op.values.len() as i64 - 1,
            _ => {}
        }
        walk_expr(self, expr);
    }

    fn visit_except_handler(&mut self, except_handler: &'a ExceptHandler) {
        self.score += 1;
        walk_except_handler(self, except_handler);
    }

    fn visit_comprehension(&mut self, comprehension: &'a Comprehension) {
        self.score += 1 + comprehension.ifs.len() as i64;
        walk_comprehension(self, comprehension);
    }
}

fn compute_complexity(body: &[Stmt]) -> i64 {
    let mut counter = ComplexityCounter { score: 1 };
    for stmt in body {
        counter.visit_stmt(stmt);
    }
    counter.score
}

struct FunctionCollector<'a, 'src, 'idx> {
    path: &'a str,
    source: &'a SourceCode<'src, 'idx>,
    threshold: i64,
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'src, 'idx> Visitor<'a> for FunctionCollector<'a, 'src, 'idx> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if let Stmt::FunctionDef(func) = stmt {
            let score = compute_complexity(&func.body);
            if score > self.threshold {
                let (line, col) = loc(self.source, func.range().start());
                self.diagnostics.push(Diagnostic {
                    path: self.path.to_string(),
                    line,
                    col,
                    rule_id: Complexity::RULE_ID,
                    message: format!(
                        "Function `{}` has cyclomatic complexity {} (threshold {})",
                        func.name.id.as_str(),
                        score,
                        self.threshold
                    ),
                });
            }
        }
        walk_stmt(self, stmt);
    }
}

impl Rule for Complexity {
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
        let mut collector = FunctionCollector {
            path,
            source,
            threshold: config.max_complexity,
            diagnostics: Vec::new(),
        };
        walk_body(&mut collector, &tree.body);
        collector.diagnostics
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
        Complexity.check(&tree, "example.py", &source_code, config)
    }

    #[test]
    fn simple_function_is_not_flagged() {
        let diagnostics = check("def add(a, b):\n    return a + b\n", &Config::default());
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn flags_high_complexity_function() {
        let config = Config {
            max_complexity: 3,
            ..Config::default()
        };
        let source = "def classify(x):\n    if x > 0:\n        if x > 10:\n            return \"big\"\n        return \"small\"\n    elif x < 0:\n        return \"negative\"\n    return \"zero\"\n";
        let diagnostics = check(source, &config);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "SA005");
    }

    #[test]
    fn nested_function_scored_independently() {
        let config = Config {
            max_complexity: 1,
            ..Config::default()
        };
        let source = "def outer():\n    def inner():\n        if True:\n            return 1\n        return 2\n    return inner()\n";
        let diagnostics = check(source, &config);
        let names: std::collections::HashSet<&str> = diagnostics
            .iter()
            .map(|d| d.message.split('`').nth(1).unwrap())
            .collect();
        assert_eq!(names, std::collections::HashSet::from(["inner"]));
    }
}
