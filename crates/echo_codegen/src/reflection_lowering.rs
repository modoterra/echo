use echo_ast::TypedParam;

use crate::{CoreRuntimeSymbol, IrModule};

const REFLECTION_SOURCE_PHP_BUILTIN: i32 = 1;
const REFLECTION_SOURCE_STD: i32 = 2;
const REFLECTION_SOURCE_USERLAND: i32 = 3;

impl IrModule {
    pub(super) fn render_reflection_registrations(&mut self, body: &mut String) {
        let php_builtins = echo_reflection::php_builtins()
            .iter()
            .map(|function| {
                (
                    function.qualified_name.clone(),
                    function.params_signature(),
                    function.return_type_signature().to_string(),
                    REFLECTION_SOURCE_PHP_BUILTIN,
                )
            })
            .collect::<Vec<_>>();
        for (name, params, return_type, source_kind) in php_builtins {
            self.register_function_reflection(body, &name, &params, &return_type, source_kind);
        }

        let std_functions = echo_reflection::functions()
            .iter()
            .filter(|function| function.source == echo_reflection::FunctionSource::Std)
            .map(|function| {
                (
                    function.qualified_name.clone(),
                    function.params_signature(),
                    function.return_type_signature().to_string(),
                    REFLECTION_SOURCE_STD,
                )
            })
            .collect::<Vec<_>>();
        for (name, params, return_type, source_kind) in std_functions {
            self.register_function_reflection(body, &name, &params, &return_type, source_kind);
        }

        let mut functions = self.functions.values().cloned().collect::<Vec<_>>();
        functions.sort_by(|left, right| left.name.cmp(&right.name));

        for function in &functions {
            let name = function.name.clone();
            let params = function_params_signature(&function.params);
            let return_type = function.return_type.clone().unwrap_or_default();

            self.register_function_reflection(
                body,
                &name,
                &params,
                &return_type,
                REFLECTION_SOURCE_USERLAND,
            );
        }
    }

    fn register_function_reflection(
        &mut self,
        body: &mut String,
        name: &str,
        params: &str,
        return_type: &str,
        source_kind: i32,
    ) {
        let name_global = self.string_global(name);
        let params_global = self.string_global(params);
        let return_type_global = self.string_global(return_type);

        body.push_str(&format!(
            "  call void @{}(ptr @{name_global}, i64 {}, ptr @{params_global}, i64 {}, ptr @{return_type_global}, i64 {}, i32 {source_kind})\n",
            CoreRuntimeSymbol::RegisterFunction.symbol(),
            name.len(),
            params.len(),
            return_type.len()
        ));
    }
}

fn function_params_signature(params: &[TypedParam]) -> String {
    params
        .iter()
        .map(|param| match &param.ty {
            Some(ty) => format!("{ty} ${}", param.name),
            None => format!("${}", param.name),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
