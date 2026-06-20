use rustpython_ruff_python_ast::visitor::{walk_body, walk_except_handler, Visitor};
use rustpython_ruff_python_ast::{ExceptHandler, Expr, ModModule};
use rustpython_ruff_source_file::SourceCode;

use super::Rule;
use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::visitor::loc;

pub struct BroadExceptions;

impl BroadExceptions {
    pub const RULE_ID: &'static str = "SA003";
}

struct Collector<'a, 'src, 'idx> {
    path: &'a str,
    source: &'a SourceCode<'src, 'idx>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'src, 'idx> Visitor<'a> for Collector<'a, 'src, 'idx> {
    fn visit_except_handler(&mut self, except_handler: &'a ExceptHandler) {
        let ExceptHandler::ExceptHandler(handler) = except_handler;
        let (line, col) = loc(self.source, handler.range.start());
        let message = match handler.type_.as_deref() {
            None => Some("Broad exception handler `except:`".to_string()),
            Some(Expr::Name(name)) if matches!(name.id.as_str(), "Exception" | "BaseException") => {
                Some(format!(
                    "Broad exception handler `except {}`",
                    name.id.as_str()
                ))
            }
            _ => None,
        };
        if let Some(message) = message {
            self.diagnostics.push(Diagnostic {
                path: self.path.to_string(),
                line,
                col,
                rule_id: BroadExceptions::RULE_ID,
                message,
            });
        }
        walk_except_handler(self, except_handler);
    }
}

impl Rule for BroadExceptions {
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
        BroadExceptions.check(&tree, "example.py", &source_code, &Config::default())
    }

    #[test]
    fn flags_bare_except() {
        let diagnostics = check("try:\n    risky()\nexcept:\n    pass\n");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "SA003");
    }

    #[test]
    fn flags_except_exception() {
        let diagnostics = check("try:\n    risky()\nexcept Exception:\n    pass\n");
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_specific_exception() {
        let diagnostics = check("try:\n    risky()\nexcept ValueError:\n    pass\n");
        assert_eq!(diagnostics, vec![]);
    }
}
