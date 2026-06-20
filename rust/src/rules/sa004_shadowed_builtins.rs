use rustpython_ruff_python_ast::visitor::{walk_body, Visitor};
use rustpython_ruff_python_ast::{Expr, ModModule, Stmt};
use rustpython_ruff_source_file::SourceCode;
use rustpython_ruff_text_size::Ranged;

use super::Rule;
use crate::config::Config;
use crate::diagnostics::Diagnostic;
use crate::visitor::loc;

pub struct ShadowedBuiltins;

impl ShadowedBuiltins {
    pub const RULE_ID: &'static str = "SA004";
}

/// Captured via `python3 -c "import builtins; print(sorted(dir(builtins)))"`
/// against CPython 3.14.5. Drifts as CPython adds builtins; revisit if a
/// newer minimum Python version needs to be targeted.
const BUILTIN_NAMES: &[&str] = &[
    "ArithmeticError",
    "AssertionError",
    "AttributeError",
    "BaseException",
    "BaseExceptionGroup",
    "BlockingIOError",
    "BrokenPipeError",
    "BufferError",
    "BytesWarning",
    "ChildProcessError",
    "ConnectionAbortedError",
    "ConnectionError",
    "ConnectionRefusedError",
    "ConnectionResetError",
    "DeprecationWarning",
    "EOFError",
    "Ellipsis",
    "EncodingWarning",
    "EnvironmentError",
    "Exception",
    "ExceptionGroup",
    "False",
    "FileExistsError",
    "FileNotFoundError",
    "FloatingPointError",
    "FutureWarning",
    "GeneratorExit",
    "IOError",
    "ImportError",
    "ImportWarning",
    "IndentationError",
    "IndexError",
    "InterruptedError",
    "IsADirectoryError",
    "KeyError",
    "KeyboardInterrupt",
    "LookupError",
    "MemoryError",
    "ModuleNotFoundError",
    "NameError",
    "None",
    "NotADirectoryError",
    "NotImplemented",
    "NotImplementedError",
    "OSError",
    "OverflowError",
    "PendingDeprecationWarning",
    "PermissionError",
    "ProcessLookupError",
    "PythonFinalizationError",
    "RecursionError",
    "ReferenceError",
    "ResourceWarning",
    "RuntimeError",
    "RuntimeWarning",
    "StopAsyncIteration",
    "StopIteration",
    "SyntaxError",
    "SyntaxWarning",
    "SystemError",
    "SystemExit",
    "TabError",
    "TimeoutError",
    "True",
    "TypeError",
    "UnboundLocalError",
    "UnicodeDecodeError",
    "UnicodeEncodeError",
    "UnicodeError",
    "UnicodeTranslateError",
    "UnicodeWarning",
    "UserWarning",
    "ValueError",
    "Warning",
    "ZeroDivisionError",
    "_IncompleteInputError",
    "__build_class__",
    "__debug__",
    "__doc__",
    "__import__",
    "__loader__",
    "__name__",
    "__package__",
    "__spec__",
    "abs",
    "aiter",
    "all",
    "anext",
    "any",
    "ascii",
    "bin",
    "bool",
    "breakpoint",
    "bytearray",
    "bytes",
    "callable",
    "chr",
    "classmethod",
    "compile",
    "complex",
    "copyright",
    "credits",
    "delattr",
    "dict",
    "dir",
    "divmod",
    "enumerate",
    "eval",
    "exec",
    "exit",
    "filter",
    "float",
    "format",
    "frozenset",
    "getattr",
    "globals",
    "hasattr",
    "hash",
    "help",
    "hex",
    "id",
    "input",
    "int",
    "isinstance",
    "issubclass",
    "iter",
    "len",
    "license",
    "list",
    "locals",
    "map",
    "max",
    "memoryview",
    "min",
    "next",
    "object",
    "oct",
    "open",
    "ord",
    "pow",
    "print",
    "property",
    "quit",
    "range",
    "repr",
    "reversed",
    "round",
    "set",
    "setattr",
    "slice",
    "sorted",
    "staticmethod",
    "str",
    "sum",
    "super",
    "tuple",
    "type",
    "vars",
    "zip",
];

fn is_builtin(name: &str) -> bool {
    BUILTIN_NAMES.contains(&name)
}

struct Collector<'a, 'src, 'idx> {
    path: &'a str,
    source: &'a SourceCode<'src, 'idx>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'src, 'idx> Collector<'a, 'src, 'idx> {
    fn report(&mut self, name: &str, offset: rustpython_ruff_text_size::TextSize, kind: &str) {
        if !is_builtin(name) {
            return;
        }
        let (line, col) = loc(self.source, offset);
        self.diagnostics.push(Diagnostic {
            path: self.path.to_string(),
            line,
            col,
            rule_id: ShadowedBuiltins::RULE_ID,
            message: format!("{kind} `{name}` shadows a built-in name"),
        });
    }
}

impl<'a, 'src, 'idx> Visitor<'a> for Collector<'a, 'src, 'idx> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::FunctionDef(func) => {
                self.report(func.name.id.as_str(), func.range().start(), "Function");
                let params = &func.parameters;
                for param in params
                    .posonlyargs
                    .iter()
                    .chain(&params.args)
                    .chain(&params.kwonlyargs)
                {
                    self.report(
                        param.parameter.name.id.as_str(),
                        param.parameter.range().start(),
                        "Parameter",
                    );
                }
                if let Some(vararg) = &params.vararg {
                    self.report(vararg.name.id.as_str(), vararg.range().start(), "Parameter");
                }
                if let Some(kwarg) = &params.kwarg {
                    self.report(kwarg.name.id.as_str(), kwarg.range().start(), "Parameter");
                }
            }
            Stmt::ClassDef(class) => {
                self.report(class.name.id.as_str(), class.range().start(), "Class");
            }
            Stmt::Assign(assign) => {
                for target in &assign.targets {
                    if let Expr::Name(name) = target {
                        self.report(name.id.as_str(), name.range().start(), "Variable");
                    }
                }
            }
            _ => {}
        }
        rustpython_ruff_python_ast::visitor::walk_stmt(self, stmt);
    }
}

impl Rule for ShadowedBuiltins {
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
        ShadowedBuiltins.check(&tree, "example.py", &source_code, &Config::default())
    }

    #[test]
    fn flags_shadowed_function_name() {
        let diagnostics = check("def list():\n    return []\n");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "SA004");
    }

    #[test]
    fn flags_shadowed_parameter() {
        let diagnostics = check("def process(id, dict):\n    return id, dict\n");
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn flags_shadowed_variable_assignment() {
        let diagnostics = check("list = [1, 2, 3]\n");
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn ignores_non_builtin_names() {
        let diagnostics = check("def process(item, value):\n    return item, value\n");
        assert_eq!(diagnostics, vec![]);
    }
}
