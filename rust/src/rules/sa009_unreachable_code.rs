use rustpython_ruff_python_ast::visitor::{
    walk_body, walk_elif_else_clause, walk_except_handler, walk_stmt, Visitor,
};
use rustpython_ruff_python_ast::{ElifElseClause, ExceptHandler, ModModule, Stmt};
use rustpython_ruff_source_file::SourceCode;
use rustpython_ruff_text_size::Ranged;

use super::Rule;
use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::visitor::loc;

pub struct UnreachableCode;

impl UnreachableCode {
    pub const RULE_ID: &'static str = "SA009";
}

fn keyword_for(stmt: &Stmt) -> Option<&'static str> {
    match stmt {
        Stmt::Return(_) => Some("return"),
        Stmt::Raise(_) => Some("raise"),
        Stmt::Break(_) => Some("break"),
        Stmt::Continue(_) => Some("continue"),
        _ => None,
    }
}

fn check_block(stmts: &[Stmt], path: &str, source: &SourceCode<'_, '_>) -> Option<Diagnostic> {
    let last = stmts.len().saturating_sub(1);
    for (i, stmt) in stmts.iter().take(last).enumerate() {
        if let Some(keyword) = keyword_for(stmt) {
            let unreachable = &stmts[i + 1];
            let (line, col) = loc(source, unreachable.range().start());
            return Some(Diagnostic {
                path: path.to_string(),
                line,
                col,
                rule_id: UnreachableCode::RULE_ID,
                message: format!("Unreachable code after `{keyword}`"),
            });
        }
    }
    None
}

struct Collector<'a, 'src, 'idx> {
    path: &'a str,
    source: &'a SourceCode<'src, 'idx>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'src, 'idx> Collector<'a, 'src, 'idx> {
    fn check_and_record(&mut self, stmts: &'a [Stmt]) {
        if let Some(diagnostic) = check_block(stmts, self.path, self.source) {
            self.diagnostics.push(diagnostic);
        }
    }
}

impl<'a, 'src, 'idx> Visitor<'a> for Collector<'a, 'src, 'idx> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::FunctionDef(s) => self.check_and_record(&s.body),
            Stmt::ClassDef(s) => self.check_and_record(&s.body),
            Stmt::For(s) => {
                self.check_and_record(&s.body);
                self.check_and_record(&s.orelse);
            }
            Stmt::While(s) => {
                self.check_and_record(&s.body);
                self.check_and_record(&s.orelse);
            }
            Stmt::If(s) => self.check_and_record(&s.body),
            Stmt::With(s) => self.check_and_record(&s.body),
            Stmt::Try(s) => {
                self.check_and_record(&s.body);
                self.check_and_record(&s.orelse);
                self.check_and_record(&s.finalbody);
            }
            _ => {}
        }
        walk_stmt(self, stmt);
    }

    fn visit_elif_else_clause(&mut self, clause: &'a ElifElseClause) {
        self.check_and_record(&clause.body);
        walk_elif_else_clause(self, clause);
    }

    fn visit_except_handler(&mut self, except_handler: &'a ExceptHandler) {
        let ExceptHandler::ExceptHandler(handler) = except_handler;
        self.check_and_record(&handler.body);
        walk_except_handler(self, except_handler);
    }
}

impl Rule for UnreachableCode {
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
        collector.check_and_record(&tree.body);
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
        UnreachableCode.check(&tree, "example.py", &source_code, &Config::default())
    }

    #[test]
    fn flags_code_after_return() {
        let diagnostics = check("def f():\n    return 1\n    print(\"never runs\")\n");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "SA009");
        assert_eq!(diagnostics[0].line, 3);
    }

    #[test]
    fn flags_code_after_break_in_loop() {
        let diagnostics =
            check("def f():\n    for i in range(10):\n        break\n        print(i)\n");
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_reachable_code() {
        let diagnostics = check("def f(x):\n    if x:\n        return 1\n    return 2\n");
        assert_eq!(diagnostics, vec![]);
    }
}
