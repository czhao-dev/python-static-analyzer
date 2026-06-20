use rustpython_ruff_python_ast::visitor::{walk_body, walk_stmt, Visitor};
use rustpython_ruff_python_ast::{ExceptHandler, Expr, ModModule, Stmt};
use rustpython_ruff_source_file::SourceCode;
use rustpython_ruff_text_size::Ranged;

use super::Rule;
use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::visitor::{loc, own_scope_exprs, own_scope_stmts};

pub struct MissingReturn;

impl MissingReturn {
    pub const RULE_ID: &'static str = "SA008";
}

/// Whether a `break` targeting THIS loop appears in `stmts` (not crossing
/// nested loops/function scopes).
fn contains_break(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        match stmt {
            Stmt::Break(_) => return true,
            Stmt::FunctionDef(_) | Stmt::For(_) | Stmt::While(_) => continue,
            Stmt::If(s) => {
                if contains_break(&s.body) {
                    return true;
                }
                for clause in &s.elif_else_clauses {
                    if contains_break(&clause.body) {
                        return true;
                    }
                }
            }
            Stmt::ClassDef(s) => {
                if contains_break(&s.body) {
                    return true;
                }
            }
            Stmt::With(s) => {
                if contains_break(&s.body) {
                    return true;
                }
            }
            Stmt::Try(s) => {
                if contains_break(&s.body)
                    || contains_break(&s.orelse)
                    || contains_break(&s.finalbody)
                {
                    return true;
                }
                for handler in &s.handlers {
                    let ExceptHandler::ExceptHandler(handler) = handler;
                    if contains_break(&handler.body) {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }
    false
}

/// Whether executing this statement list always returns or raises (never falls through).
fn always_exits(stmts: &[Stmt]) -> bool {
    match stmts.last() {
        Some(stmt) => stmt_always_exits(stmt),
        None => false,
    }
}

fn stmt_always_exits(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Return(_) | Stmt::Raise(_) => true,
        Stmt::If(s) => {
            // Unlike CPython's nested `orelse: [If]` shape (where a missing
            // trailing `else` naturally falls through to an empty `orelse`),
            // ruff's flattened `elif_else_clauses` needs an explicit check
            // that the chain actually ends in a real `else` (test: None) —
            // an if/elif with no trailing else is not exhaustive.
            if s.elif_else_clauses.is_empty() {
                return false;
            }
            let ends_in_else = s.elif_else_clauses.last().is_some_and(|c| c.test.is_none());
            ends_in_else
                && always_exits(&s.body)
                && s.elif_else_clauses.iter().all(|c| always_exits(&c.body))
        }
        Stmt::With(s) => always_exits(&s.body),
        Stmt::Try(s) => {
            if !s.finalbody.is_empty() && always_exits(&s.finalbody) {
                return true;
            }
            let try_exits = if !s.orelse.is_empty() {
                always_exits(&s.orelse)
            } else {
                always_exits(&s.body)
            };
            if !s.handlers.is_empty() {
                try_exits
                    && s.handlers.iter().all(|handler| {
                        let ExceptHandler::ExceptHandler(handler) = handler;
                        always_exits(&handler.body)
                    })
            } else {
                try_exits
            }
        }
        Stmt::While(s) => {
            let is_infinite = matches!(s.test.as_ref(), Expr::BooleanLiteral(b) if b.value);
            is_infinite && !contains_break(&s.body)
        }
        _ => false,
    }
}

fn is_generator(body: &[Stmt]) -> bool {
    own_scope_exprs(body)
        .into_iter()
        .any(|expr| matches!(expr, Expr::Yield(_) | Expr::YieldFrom(_)))
}

fn returns_value(body: &[Stmt]) -> bool {
    own_scope_stmts(body)
        .into_iter()
        .any(|stmt| matches!(stmt, Stmt::Return(r) if r.value.is_some()))
}

struct Collector<'a, 'src, 'idx> {
    path: &'a str,
    source: &'a SourceCode<'src, 'idx>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'src, 'idx> Visitor<'a> for Collector<'a, 'src, 'idx> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if let Stmt::FunctionDef(func) = stmt {
            if !is_generator(&func.body) && returns_value(&func.body) && !always_exits(&func.body) {
                let (line, col) = loc(self.source, func.range().start());
                self.diagnostics.push(Diagnostic {
                    path: self.path.to_string(),
                    line,
                    col,
                    rule_id: MissingReturn::RULE_ID,
                    message: format!(
                        "Function `{}` may not return a value on all code paths",
                        func.name.id.as_str()
                    ),
                });
            }
        }
        walk_stmt(self, stmt);
    }
}

impl Rule for MissingReturn {
    fn id(&self) -> &'static str {
        Self::RULE_ID
    }

    fn check(
        &self,
        tree: &ModModule,
        path: &str,
        source: &SourceCode<'_, '_>,
        _config: &Config,
    ) -> Vec<Diagnostic> {
        let mut collector = Collector {
            path,
            source,
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

    fn check(source: &str) -> Vec<Diagnostic> {
        let tree = rustpython_ruff_python_parser::parse_module(source)
            .unwrap()
            .into_syntax();
        let index = LineIndex::from_source_text(source);
        let source_code = SourceCode::new(source, &index);
        MissingReturn.check(&tree, "example.py", &source_code, &Config::default())
    }

    #[test]
    fn flags_missing_else_branch() {
        let diagnostics = check("def classify(x):\n    if x > 0:\n        return \"positive\"\n");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "SA008");
    }

    #[test]
    fn ignores_complete_if_else() {
        let diagnostics = check(
            "def classify(x):\n    if x > 0:\n        return \"positive\"\n    else:\n        return \"non-positive\"\n",
        );
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn ignores_function_without_return_value() {
        let diagnostics = check("def log(message):\n    print(message)\n");
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn ignores_try_except_that_always_exits() {
        let diagnostics = check(
            "def parse(value):\n    try:\n        return int(value)\n    except ValueError:\n        return None\n",
        );
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn ignores_generator_functions() {
        let diagnostics = check(
            "def gen(items):\n    for item in items:\n        if item:\n            yield item\n",
        );
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn ignores_infinite_loop_without_break() {
        let diagnostics = check(
            "def serve():\n    while True:\n        if should_stop():\n            return \"done\"\n        handle()\n",
        );
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn flags_elif_chain_without_trailing_else() {
        // Regression test for ruff's flattened elif_else_clauses: an
        // if/elif with no final `else` must NOT be treated as exhaustive.
        let diagnostics =
            check("def f(x):\n    if x == 1:\n        return \"one\"\n    elif x == 2:\n        return \"two\"\n");
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_elif_chain_with_trailing_else_all_returning() {
        let diagnostics = check(
            "def f(x):\n    if x == 1:\n        return \"one\"\n    elif x == 2:\n        return \"two\"\n    else:\n        return \"other\"\n",
        );
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn ignores_infinite_loop_with_unconditional_return() {
        let diagnostics = check("def f():\n    while True:\n        return 1\n");
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn flags_infinite_loop_with_break() {
        let diagnostics = check(
            "def f(x):\n    while True:\n        if x:\n            break\n        return 1\n",
        );
        assert_eq!(diagnostics.len(), 1);
    }
}
