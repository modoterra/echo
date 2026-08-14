//! Line-table DWARF for AOT/JIT IR.
//!
//! Policy: [`docs/llvm.md`](../../../docs/llvm.md) — compile unit + subprogram +
//! one location (file, line 1, column 1) per function. MIR ops do not carry
//! spans yet, so instructions in a function share that location. No types,
//! locals, or inlined frames.

use std::path::Path;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::debug_info::{
    AsDIScope, DICompileUnit, DIFlags, DIFlagsConstants, DISubroutineType, DWARFEmissionKind,
    DWARFSourceLanguage, DebugInfoBuilder,
};
use inkwell::module::{FlagBehavior, Module};
use inkwell::values::FunctionValue;

/// File name + directory for `DIFile` / compile unit.
#[must_use]
pub fn split_debug_path(path: &Path) -> (String, String) {
    let file = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown.echo".into());
    let dir = match path.parent() {
        Some(p) => {
            let s = p.to_string_lossy();
            if s.is_empty() {
                ".".into()
            } else {
                s.into_owned()
            }
        }
        None => ".".into(),
    };
    (file, dir)
}

/// Module flags + compile unit. Caller must [`DebugInfoBuilder::finalize`]
/// after all functions are emitted and before verify.
pub fn begin_module<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    entry_path: &Path,
) -> (DebugInfoBuilder<'ctx>, DICompileUnit<'ctx>, DISubroutineType<'ctx>) {
    let three = context.i32_type().const_int(3, false);
    module.add_basic_value_flag("Debug Info Version", FlagBehavior::Warning, three);
    let four = context.i32_type().const_int(4, false);
    module.add_basic_value_flag("Dwarf Version", FlagBehavior::Warning, four);

    let (file, dir) = split_debug_path(entry_path);
    module.set_source_file_name(&file);

    let (dibuilder, cu) = module.create_debug_info_builder(
        true,
        DWARFSourceLanguage::C,
        &file,
        &dir,
        "xo",
        false,
        "",
        0,
        "",
        DWARFEmissionKind::Full,
        0,
        false,
        false,
        "",
        "",
    );
    let sub_ty = dibuilder.create_subroutine_type(cu.get_file(), None, &[], DIFlags::PUBLIC);
    (dibuilder, cu, sub_ty)
}

/// Attach a `DISubprogram` at line 1 of `file_path`.
pub fn attach_subprogram<'ctx>(
    dibuilder: &DebugInfoBuilder<'ctx>,
    cu: &DICompileUnit<'ctx>,
    sub_ty: DISubroutineType<'ctx>,
    fv: FunctionValue<'ctx>,
    display_name: &str,
    file_path: &Path,
    is_local: bool,
) {
    let (file, dir) = split_debug_path(file_path);
    let difile = dibuilder.create_file(&file, &dir);
    let linkage = fv.get_name().to_str().unwrap_or(display_name);
    let sp = dibuilder.create_function(
        cu.as_debug_info_scope(),
        display_name,
        Some(linkage),
        difile,
        1,
        sub_ty,
        is_local,
        true,
        1,
        DIFlags::PUBLIC,
        false,
    );
    fv.set_subprogram(sp);
}

/// Point the builder at line 1 / column 1 of the function's subprogram.
pub fn set_function_line<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    dibuilder: &DebugInfoBuilder<'ctx>,
    fv: FunctionValue<'ctx>,
) {
    let Some(sp) = fv.get_subprogram() else {
        return;
    };
    let loc = dibuilder.create_debug_location(context, 1, 1, sp.as_debug_info_scope(), None);
    builder.set_current_debug_location(loc);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn split_file_and_dir() {
        let (f, d) = split_debug_path(Path::new("/home/u/app.echo"));
        assert_eq!(f, "app.echo");
        assert_eq!(d, "/home/u");
        let (f, d) = split_debug_path(&PathBuf::from("t.echo"));
        assert_eq!(f, "t.echo");
        assert_eq!(d, ".");
    }
}
