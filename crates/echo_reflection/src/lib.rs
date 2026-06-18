use std::sync::OnceLock;

use echo_ast::{NamespaceSource, Stmt, TypedParam};

pub const PHP_BUILTINS_SOURCE: &str = include_str!("../../../std/php_builtins.echo");
const STD_MODULE_SOURCES: &[(&str, &str)] = &[
    ("http", include_str!("../../../std/http.echo")),
    ("net", include_str!("../../../std/net.echo")),
    ("reflect", include_str!("../../../std/reflect.echo")),
    ("time", include_str!("../../../std/time.echo")),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionReflection {
    pub name: String,
    pub qualified_name: String,
    pub source: FunctionSource,
    pub params: Vec<ParamReflection>,
    pub return_type: Option<String>,
    pub is_intrinsic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionSource {
    PhpBuiltin,
    Std,
    Userland,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamReflection {
    pub name: String,
    pub ty: Option<String>,
}

impl FunctionReflection {
    pub fn params_signature(&self) -> String {
        self.params
            .iter()
            .map(ParamReflection::signature)
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn return_type_signature(&self) -> &str {
        self.return_type.as_deref().unwrap_or("")
    }
}

impl ParamReflection {
    pub fn signature(&self) -> String {
        match &self.ty {
            Some(ty) => format!("{ty} ${}", self.name),
            None => format!("${}", self.name),
        }
    }
}

pub fn function(name: &str) -> Option<&'static FunctionReflection> {
    functions()
        .iter()
        .find(|function| function.matches_name(name))
}

pub fn php_builtin(name: &str) -> Option<&'static FunctionReflection> {
    functions().iter().find(|function| {
        function.source == FunctionSource::PhpBuiltin && function.matches_name(name)
    })
}

pub fn std_function(name: &str) -> Option<&'static FunctionReflection> {
    functions()
        .iter()
        .find(|function| function.source == FunctionSource::Std && function.matches_name(name))
}

pub fn functions() -> &'static [FunctionReflection] {
    static FUNCTIONS: OnceLock<Vec<FunctionReflection>> = OnceLock::new();
    FUNCTIONS.get_or_init(|| {
        let mut functions = reflect_php_builtins(PHP_BUILTINS_SOURCE);
        for (_, source) in STD_MODULE_SOURCES {
            functions.extend(reflect_std_functions(source));
        }
        functions
    })
}

pub fn php_builtins() -> &'static [FunctionReflection] {
    static PHP_BUILTINS: OnceLock<Vec<FunctionReflection>> = OnceLock::new();
    PHP_BUILTINS.get_or_init(|| reflect_php_builtins(PHP_BUILTINS_SOURCE))
}

pub fn reflect_php_builtins(source: &str) -> Vec<FunctionReflection> {
    reflect_functions(source, FunctionSource::PhpBuiltin)
}

pub fn reflect_std_functions(source: &str) -> Vec<FunctionReflection> {
    reflect_functions(source, FunctionSource::Std)
}

pub fn reflect_functions(source: &str, function_source: FunctionSource) -> Vec<FunctionReflection> {
    let program =
        echo_parser::parse_trusted_std(source).expect("trusted std reflection source should parse");

    let namespace = program
        .statements
        .iter()
        .find_map(|statement| match statement {
            Stmt::Namespace(namespace) if namespace.source == NamespaceSource::Std => {
                Some(namespace.name.as_string())
            }
            _ => None,
        });

    program
        .statements
        .into_iter()
        .filter_map(|statement| match statement {
            Stmt::FunctionDecl(function) => {
                let qualified_name = match (function_source, &namespace) {
                    (FunctionSource::PhpBuiltin, _) => function.name.clone(),
                    (FunctionSource::Std, Some(namespace)) => {
                        format!("{namespace}.{}", function.name)
                    }
                    _ => function.name.clone(),
                };

                Some(FunctionReflection {
                    name: function.name,
                    qualified_name,
                    source: function_source,
                    params: function
                        .params
                        .into_iter()
                        .map(ParamReflection::from)
                        .collect(),
                    return_type: function.return_type,
                    is_intrinsic: function.is_intrinsic,
                })
            }
            _ => None,
        })
        .collect()
}

impl FunctionReflection {
    fn matches_name(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name) || self.qualified_name.eq_ignore_ascii_case(name)
    }
}

impl From<TypedParam> for ParamReflection {
    fn from(param: TypedParam) -> Self {
        Self {
            name: param.name,
            ty: param.ty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflects_php_builtin_declarations_from_echo_source() {
        let strlen = php_builtin("STRLEN").expect("strlen reflected");

        assert_eq!(strlen.name, "strlen");
        assert_eq!(strlen.qualified_name, "strlen");
        assert_eq!(strlen.source, FunctionSource::PhpBuiltin);
        assert_eq!(strlen.params_signature(), "string $string");
        assert_eq!(strlen.return_type_signature(), "int");
        assert!(strlen.is_intrinsic);
    }

    #[test]
    fn reflects_std_functions_by_qualified_name() {
        let params = std_function("reflect.params").expect("reflect.params reflected");

        assert_eq!(params.name, "params");
        assert_eq!(params.qualified_name, "reflect.params");
        assert_eq!(params.source, FunctionSource::Std);
        assert_eq!(params.params_signature(), "string $name");
        assert_eq!(params.return_type_signature(), "string");
        assert!(params.is_intrinsic);
    }

    #[test]
    fn does_not_reflect_std_reflection_api_as_php_builtins() {
        assert!(php_builtin("params").is_none());
        assert!(php_builtin("returnType").is_none());
        assert!(function("reflect.params").is_some());
    }
}
