use std::collections::HashMap;
use std::collections::HashSet;

use rustpython_ruff_python_ast::visitor::{walk_body, walk_expr, walk_stmt, Visitor};
use rustpython_ruff_python_ast::{Expr, ExprContext, ExprName, ModModule, Stmt};
use rustpython_ruff_source_file::SourceCode;
use rustpython_ruff_text_size::{Ranged, TextSize};

use super::Rule;
use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::visitor::loc;

pub struct UnusedVariables;

impl UnusedVariables {
    pub const RULE_ID: &'static str = "SA006";
}

fn assigned_names<'a>(target: &'a Expr, out: &mut Vec<&'a ExprName>) {
    match target {
        Expr::Name(name) => out.push(name),
        Expr::Tuple(tuple) => {
            for elt in &tuple.elts {
                assigned_names(elt, out);
            }
        }
        Expr::List(list) => {
            for elt in &list.elts {
                assigned_names(elt, out);
            }
        }
        Expr::Starred(starred) => assigned_names(&starred.value, out),
        _ => {}
    }
}

/// Unrestricted walk (crosses into nested scopes) — a variable used inside a
/// nested closure still counts as "used" in the enclosing function, matching
/// Python's `ast.walk(func)` in `_collect_usage`.
struct UsageCollector {
    used: HashSet<String>,
    declared_global: HashSet<String>,
}

impl<'a> Visitor<'a> for UsageCollector {
    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Name(name) = expr {
            if name.ctx == ExprContext::Load {
                self.used.insert(name.id.as_str().to_string());
            }
        }
        walk_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Global(g) => {
                self.declared_global
                    .extend(g.names.iter().map(|n| n.id.as_str().to_string()));
            }
            Stmt::Nonlocal(n) => {
                self.declared_global
                    .extend(n.names.iter().map(|n| n.id.as_str().to_string()));
            }
            Stmt::AugAssign(aug) => {
                if let Expr::Name(name) = aug.target.as_ref() {
                    self.used.insert(name.id.as_str().to_string());
                }
            }
            _ => {}
        }
        walk_stmt(self, stmt);
    }
}

/// Own-scope walk that, unlike `crate::visitor::own_scope_stmts`, also stops
/// at `ClassDef` — matches `unused_variables.py`'s `_NESTED_SCOPES` exactly
/// (the one rule whose nested-scope set includes classes).
fn collect_first_assignments(body: &[Stmt]) -> HashMap<String, TextSize> {
    struct Collector {
        first: HashMap<String, TextSize>,
    }
    impl<'a> Visitor<'a> for Collector {
        fn visit_stmt(&mut self, stmt: &'a Stmt) {
            if matches!(stmt, Stmt::FunctionDef(_) | Stmt::ClassDef(_)) {
                return;
            }
            if let Stmt::Assign(assign) = stmt {
                let mut names = Vec::new();
                for target in &assign.targets {
                    assigned_names(target, &mut names);
                }
                for name in names {
                    if name.id.as_str().starts_with('_') {
                        continue;
                    }
                    self.first
                        .entry(name.id.as_str().to_string())
                        .or_insert_with(|| name.range().start());
                }
            }
            walk_stmt(self, stmt);
        }
        fn visit_expr(&mut self, expr: &'a Expr) {
            if matches!(expr, Expr::Lambda(_)) {
                return;
            }
            walk_expr(self, expr);
        }
    }
    let mut collector = Collector {
        first: HashMap::new(),
    };
    for stmt in body {
        collector.visit_stmt(stmt);
    }
    collector.first
}

struct FunctionCollector<'a, 'src, 'idx> {
    path: &'a str,
    source: &'a SourceCode<'src, 'idx>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'src, 'idx> Visitor<'a> for FunctionCollector<'a, 'src, 'idx> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if let Stmt::FunctionDef(func) = stmt {
            let mut usage = UsageCollector {
                used: HashSet::new(),
                declared_global: HashSet::new(),
            };
            walk_body(&mut usage, &func.body);
            let first_assignment = collect_first_assignments(&func.body);

            for (name, offset) in &first_assignment {
                if usage.used.contains(name) || usage.declared_global.contains(name) {
                    continue;
                }
                let (line, col) = loc(self.source, *offset);
                self.diagnostics.push(Diagnostic {
                    path: self.path.to_string(),
                    line,
                    col,
                    rule_id: UnusedVariables::RULE_ID,
                    message: format!("Local variable `{name}` is assigned but never used"),
                });
            }
        }
        walk_stmt(self, stmt);
    }
}

impl Rule for UnusedVariables {
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
        let mut collector = FunctionCollector {
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
        UnusedVariables.check(&tree, "example.py", &source_code, &Config::default())
    }

    #[test]
    fn flags_unused_local_variable() {
        let diagnostics =
            check("def compute():\n    total = 0\n    unused = 42\n    return total\n");
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("unused"));
        assert_eq!(diagnostics[0].rule_id, "SA006");
    }

    #[test]
    fn ignores_used_variable() {
        let diagnostics = check(
            "def compute():\n    total = 0\n    for item in range(10):\n        total += item\n    return total\n",
        );
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn ignores_underscore_prefixed() {
        let diagnostics = check("def compute():\n    _ignored = expensive_call()\n    return 1\n");
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn ignores_global_declarations() {
        let diagnostics = check(
            "counter = 0\n\ndef increment():\n    global counter\n    counter = counter + 1\n",
        );
        assert_eq!(diagnostics, vec![]);
    }
}
