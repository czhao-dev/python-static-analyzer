use std::collections::HashSet;

use rustpython_ruff_python_ast::visitor::{walk_body, Visitor};
use rustpython_ruff_python_ast::{Expr, ModModule, Stmt};
use rustpython_ruff_source_file::SourceCode;
use rustpython_ruff_text_size::Ranged;

use super::Rule;
use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::visitor::loc;

pub struct UnusedImports;

impl UnusedImports {
    pub const RULE_ID: &'static str = "SA002";
}

struct UsedNameCollector {
    used: HashSet<String>,
}

impl<'a> Visitor<'a> for UsedNameCollector {
    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Name(name) => {
                self.used.insert(name.id.as_str().to_string());
            }
            Expr::Attribute(attr) => {
                if let Expr::Name(name) = attr.value.as_ref() {
                    self.used.insert(name.id.as_str().to_string());
                }
            }
            _ => {}
        }
        rustpython_ruff_python_ast::visitor::walk_expr(self, expr);
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if let Stmt::Assign(assign) = stmt {
            let targets_dunder_all = assign
                .targets
                .iter()
                .any(|target| matches!(target, Expr::Name(name) if name.id.as_str() == "__all__"));
            if targets_dunder_all {
                let elements = match assign.value.as_ref() {
                    Expr::List(list) => Some(&list.elts),
                    Expr::Tuple(tuple) => Some(&tuple.elts),
                    _ => None,
                };
                if let Some(elements) = elements {
                    for element in elements {
                        if let Expr::StringLiteral(literal) = element {
                            self.used.insert(literal.value.to_str().to_string());
                        }
                    }
                }
            }
        }
        rustpython_ruff_python_ast::visitor::walk_stmt(self, stmt);
    }
}

fn collect_used_names(tree: &ModModule) -> HashSet<String> {
    let mut collector = UsedNameCollector {
        used: HashSet::new(),
    };
    walk_body(&mut collector, &tree.body);
    collector.used
}

struct ImportCollector<'a, 'src, 'idx> {
    path: &'a str,
    source: &'a SourceCode<'src, 'idx>,
    used: &'a HashSet<String>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'src, 'idx> ImportCollector<'a, 'src, 'idx> {
    fn report(&mut self, stmt: &Stmt, import_name: &str) {
        let (line, col) = loc(self.source, stmt.range().start());
        self.diagnostics.push(Diagnostic {
            path: self.path.to_string(),
            line,
            col,
            rule_id: UnusedImports::RULE_ID,
            message: format!("Unused import `{import_name}`"),
        });
    }
}

impl<'a, 'src, 'idx> Visitor<'a> for ImportCollector<'a, 'src, 'idx> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Import(import) => {
                for alias in &import.names {
                    let bound = alias
                        .asname
                        .as_ref()
                        .map(|n| n.id.as_str().to_string())
                        .unwrap_or_else(|| {
                            alias
                                .name
                                .id
                                .as_str()
                                .split('.')
                                .next()
                                .unwrap_or_default()
                                .to_string()
                        });
                    if !self.used.contains(&bound) {
                        self.report(stmt, alias.name.id.as_str());
                    }
                }
            }
            Stmt::ImportFrom(import_from) => {
                let is_future = import_from
                    .module
                    .as_ref()
                    .is_some_and(|m| m.id.as_str() == "__future__");
                if !is_future {
                    for alias in &import_from.names {
                        if alias.name.id.as_str() == "*" {
                            continue;
                        }
                        let bound = alias
                            .asname
                            .as_ref()
                            .map(|n| n.id.as_str().to_string())
                            .unwrap_or_else(|| alias.name.id.as_str().to_string());
                        if !self.used.contains(&bound) {
                            self.report(stmt, alias.name.id.as_str());
                        }
                    }
                }
            }
            _ => {}
        }
        rustpython_ruff_python_ast::visitor::walk_stmt(self, stmt);
    }
}

impl Rule for UnusedImports {
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
        let used = collect_used_names(tree);
        let mut collector = ImportCollector {
            path,
            source,
            used: &used,
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
        UnusedImports.check(&tree, "example.py", &source_code, &Config::default())
    }

    #[test]
    fn flags_unused_import() {
        let diagnostics = check("import json\nimport os\n\nprint(os.getcwd())\n");
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("json"));
        assert_eq!(diagnostics[0].rule_id, "SA002");
    }

    #[test]
    fn ignores_used_import_from() {
        let diagnostics = check("from pathlib import Path\n\np = Path(\".\")\n");
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn respects_aliases() {
        let diagnostics = check("import numpy as np\n\nprint(np.array([1]))\n");
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn ignores_future_imports() {
        let diagnostics = check("from __future__ import annotations\n");
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn respects_dunder_all() {
        let diagnostics = check("from mymodule import helper\n\n__all__ = [\"helper\"]\n");
        assert_eq!(diagnostics, vec![]);
    }
}
