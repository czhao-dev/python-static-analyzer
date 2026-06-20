use crate::config::Config;
use crate::diagnostics::Diagnostic;
use rustpython_ruff_python_ast::ModModule;
use rustpython_ruff_source_file::SourceCode;

pub trait Rule {
    fn id(&self) -> &'static str;
    fn check(
        &self,
        tree: &ModModule,
        path: &str,
        source: &SourceCode<'_, '_>,
        config: &Config,
    ) -> Vec<Diagnostic>;
}

mod sa001_mutable_defaults;
mod sa002_unused_imports;
mod sa003_broad_exceptions;
mod sa004_shadowed_builtins;
mod sa005_complexity;
mod sa006_unused_variables;
mod sa007_nesting;
mod sa008_missing_return;
mod sa009_unreachable_code;

pub use sa001_mutable_defaults::MutableDefaults;
pub use sa002_unused_imports::UnusedImports;
pub use sa003_broad_exceptions::BroadExceptions;
pub use sa004_shadowed_builtins::ShadowedBuiltins;
pub use sa005_complexity::Complexity;
pub use sa006_unused_variables::UnusedVariables;
pub use sa007_nesting::Nesting;
pub use sa008_missing_return::MissingReturn;
pub use sa009_unreachable_code::UnreachableCode;

pub const ALL_RULES: &[&dyn Rule] = &[
    &MutableDefaults,
    &UnusedImports,
    &BroadExceptions,
    &ShadowedBuiltins,
    &Complexity,
    &UnusedVariables,
    &Nesting,
    &MissingReturn,
    &UnreachableCode,
];
