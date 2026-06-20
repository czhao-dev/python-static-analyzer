//! Shared traversal helpers reused across multiple rules.

use rustpython_ruff_python_ast::visitor::{walk_expr, walk_stmt, Visitor};
use rustpython_ruff_python_ast::{Expr, Stmt};
use rustpython_ruff_source_file::SourceCode;
use rustpython_ruff_text_size::TextSize;

pub use rustpython_ruff_text_size::Ranged;

/// 1-indexed line, 0-indexed column — matches CPython ast's `(lineno, col_offset)`.
pub fn loc(source: &SourceCode<'_, '_>, offset: TextSize) -> (usize, usize) {
    let lc = source.line_column(offset);
    (lc.line.get(), lc.column.get() - 1)
}

/// Statements reachable from `stmts` without crossing into a nested
/// function/lambda scope. Mirrors most rules' `_NESTED_SCOPES = (FunctionDef,
/// AsyncFunctionDef, Lambda)` — notably this does NOT stop at `ClassDef`,
/// matching `complexity.py`/`nesting.py`/`missing_return.py`. `SA006`
/// (`unused_variables.py`) is the one rule whose `_NESTED_SCOPES` also
/// includes `ClassDef`; it implements its own boundary rather than reusing this.
pub fn own_scope_stmts(stmts: &[Stmt]) -> Vec<&Stmt> {
    struct Collector<'a> {
        out: Vec<&'a Stmt>,
    }
    impl<'a> Visitor<'a> for Collector<'a> {
        fn visit_stmt(&mut self, stmt: &'a Stmt) {
            if matches!(stmt, Stmt::FunctionDef(_)) {
                return;
            }
            self.out.push(stmt);
            walk_stmt(self, stmt);
        }
        fn visit_expr(&mut self, expr: &'a Expr) {
            if matches!(expr, Expr::Lambda(_)) {
                return;
            }
            walk_expr(self, expr);
        }
    }
    let mut collector = Collector { out: Vec::new() };
    for stmt in stmts {
        collector.visit_stmt(stmt);
    }
    collector.out
}

/// Expressions reachable from `stmts` without crossing into a nested
/// function/class/lambda scope.
pub fn own_scope_exprs(stmts: &[Stmt]) -> Vec<&Expr> {
    struct Collector<'a> {
        out: Vec<&'a Expr>,
    }
    impl<'a> Visitor<'a> for Collector<'a> {
        fn visit_stmt(&mut self, stmt: &'a Stmt) {
            if matches!(stmt, Stmt::FunctionDef(_)) {
                return;
            }
            walk_stmt(self, stmt);
        }
        fn visit_expr(&mut self, expr: &'a Expr) {
            if matches!(expr, Expr::Lambda(_)) {
                return;
            }
            self.out.push(expr);
            walk_expr(self, expr);
        }
    }
    let mut collector = Collector { out: Vec::new() };
    for stmt in stmts {
        collector.visit_stmt(stmt);
    }
    collector.out
}
