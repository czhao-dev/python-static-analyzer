use rustpython_ruff_python_ast::visitor::{walk_body, Visitor};
use rustpython_ruff_python_ast::{Expr, ModModule, ParameterWithDefault, Stmt};
use rustpython_ruff_source_file::SourceCode;
use rustpython_ruff_text_size::Ranged;

use super::Rule;
use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::visitor::loc;

pub struct MutableDefaults;

impl MutableDefaults {
    pub const RULE_ID: &'static str = "SA001";
}

fn is_mutable(expr: &Expr) -> bool {
    match expr {
        Expr::List(_)
        | Expr::Dict(_)
        | Expr::Set(_)
        | Expr::ListComp(_)
        | Expr::DictComp(_)
        | Expr::SetComp(_) => true,
        Expr::Call(call) => {
            matches!(call.func.as_ref(), Expr::Name(name) if matches!(name.id.as_str(), "list" | "dict" | "set"))
        }
        _ => false,
    }
}

struct Collector<'a, 'src, 'idx> {
    path: &'a str,
    source: &'a SourceCode<'src, 'idx>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'src, 'idx> Collector<'a, 'src, 'idx> {
    fn check_params(&mut self, params: &'a [ParameterWithDefault]) {
        for param in params {
            let Some(default) = &param.default else {
                continue;
            };
            if !is_mutable(default) {
                continue;
            }
            let (line, col) = loc(self.source, default.range().start());
            let snippet = &self.source.text()[default.range()];
            self.diagnostics.push(Diagnostic {
                path: self.path.to_string(),
                line,
                col,
                rule_id: MutableDefaults::RULE_ID,
                message: format!(
                    "Mutable default argument `{}={}`",
                    param.parameter.name.as_str(),
                    snippet
                ),
            });
        }
    }
}

impl<'a, 'src, 'idx> Visitor<'a> for Collector<'a, 'src, 'idx> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if let Stmt::FunctionDef(func) = stmt {
            self.check_params(&func.parameters.posonlyargs);
            self.check_params(&func.parameters.args);
            self.check_params(&func.parameters.kwonlyargs);
        }
        rustpython_ruff_python_ast::visitor::walk_stmt(self, stmt);
    }
}

impl Rule for MutableDefaults {
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
        MutableDefaults.check(&tree, "example.py", &source_code, &Config::default())
    }

    #[test]
    fn flags_list_default() {
        let diagnostics =
            check("def add_item(item, items=[]):\n    items.append(item)\n    return items\n");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "SA001");
        assert!(diagnostics[0].message.contains("items=[]"));
    }

    #[test]
    fn flags_dict_and_set_defaults() {
        let diagnostics = check("def merge(data={}, tags=set()):\n    return data, tags\n");
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn ignores_immutable_defaults() {
        let diagnostics =
            check("def add_item(item, items=None, count=0, name=\"\"):\n    return item\n");
        assert_eq!(diagnostics, vec![]);
    }

    #[test]
    fn flags_kwonly_mutable_default() {
        let diagnostics = check("def configure(*, options=[]):\n    return options\n");
        assert_eq!(diagnostics.len(), 1);
    }
}
