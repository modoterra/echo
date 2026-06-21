use std::sync::OnceLock;

use echo_ast::{NamespaceSource, Stmt, TypedParam};

const STD_MODULE_SOURCES: &[(&str, &str)] = &[
    ("http", include_str!("../../../std/http.echo")),
    ("assert", include_str!("../../../std/assert.echo")),
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
        let mut functions = php_builtin_reflections();
        for (_, source) in STD_MODULE_SOURCES {
            functions.extend(reflect_std_functions(source));
        }
        functions
    })
}

pub fn php_builtins() -> &'static [FunctionReflection] {
    static PHP_BUILTINS: OnceLock<Vec<FunctionReflection>> = OnceLock::new();
    PHP_BUILTINS.get_or_init(php_builtin_reflections)
}

pub fn reflect_std_functions(source: &str) -> Vec<FunctionReflection> {
    reflect_functions(source, FunctionSource::Std)
}

fn php_builtin_reflections() -> Vec<FunctionReflection> {
    [
        php_builtin_reflection("abs", &[("num", Some("int|float"))], Some("int|float")),
        php_builtin_reflection("flush", &[], Some("void")),
        php_builtin_reflection(
            "define",
            &[("constant_name", Some("string")), ("value", Some("mixed"))],
            Some("bool"),
        ),
        php_builtin_reflection(
            "microtime",
            &[("as_float", Some("bool"))],
            Some("string|float"),
        ),
        php_builtin_reflection(
            "ob_implicit_flush",
            &[("enable", Some("bool"))],
            Some("void"),
        ),
        php_builtin_reflection("ob_start", &[], Some("bool")),
        php_builtin_reflection("ob_flush", &[], Some("bool")),
        php_builtin_reflection("ob_clean", &[], Some("bool")),
        php_builtin_reflection("ob_end_flush", &[], Some("bool")),
        php_builtin_reflection("ob_end_clean", &[], Some("bool")),
        php_builtin_reflection("ob_get_clean", &[], Some("string|false")),
        php_builtin_reflection("ob_get_contents", &[], Some("string|false")),
        php_builtin_reflection("ob_get_flush", &[], Some("string|false")),
        php_builtin_reflection("ob_get_length", &[], Some("int|false")),
        php_builtin_reflection("ob_get_level", &[], Some("int")),
        php_builtin_reflection("strlen", &[("string", Some("string"))], Some("int")),
        php_builtin_reflection(
            "basename",
            &[("path", Some("string")), ("suffix", Some("string"))],
            Some("string"),
        ),
        php_builtin_reflection(
            "dirname",
            &[("path", Some("string")), ("levels", Some("int"))],
            Some("string"),
        ),
        php_builtin_reflection("count", &[("value", Some("Countable|array"))], Some("int")),
        php_builtin_reflection("sizeof", &[("value", Some("Countable|array"))], Some("int")),
        php_builtin_reflection(
            "function_exists",
            &[("function", Some("string"))],
            Some("bool"),
        ),
        php_builtin_reflection("gettype", &[("value", Some("mixed"))], Some("string")),
        php_builtin_reflection("array_is_list", &[("array", Some("array"))], Some("bool")),
        php_builtin_reflection("is_array", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_countable", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_iterable", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_numeric", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_null", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_bool", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_callable", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_int", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_integer", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_long", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_float", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_double", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_finite", &[("num", Some("float"))], Some("bool")),
        php_builtin_reflection("is_infinite", &[("num", Some("float"))], Some("bool")),
        php_builtin_reflection("is_nan", &[("num", Some("float"))], Some("bool")),
        php_builtin_reflection("is_object", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_resource", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_string", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("is_scalar", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("strval", &[("value", Some("mixed"))], Some("string")),
        php_builtin_reflection("boolval", &[("value", Some("mixed"))], Some("bool")),
        php_builtin_reflection("intval", &[("value", Some("mixed"))], Some("int")),
        php_builtin_reflection("strtoupper", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("strtolower", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("ucwords", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("strrev", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("ucfirst", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("lcfirst", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("ord", &[("character", Some("string"))], Some("int")),
        php_builtin_reflection("str_rot13", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("chr", &[("codepoint", Some("int"))], Some("string")),
        php_builtin_reflection("decbin", &[("num", Some("int"))], Some("string")),
        php_builtin_reflection("dechex", &[("num", Some("int"))], Some("string")),
        php_builtin_reflection("decoct", &[("num", Some("int"))], Some("string")),
        php_builtin_reflection(
            "bindec",
            &[("binary_string", Some("string"))],
            Some("int|float"),
        ),
        php_builtin_reflection(
            "hexdec",
            &[("hex_string", Some("string"))],
            Some("int|float"),
        ),
        php_builtin_reflection(
            "octdec",
            &[("octal_string", Some("string"))],
            Some("int|float"),
        ),
        php_builtin_reflection("deg2rad", &[("num", Some("float"))], Some("float")),
        php_builtin_reflection("rad2deg", &[("num", Some("float"))], Some("float")),
        php_builtin_reflection("bin2hex", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection(
            "base64_encode",
            &[("string", Some("string"))],
            Some("string"),
        ),
        php_builtin_reflection(
            "base64_decode",
            &[("string", Some("string"))],
            Some("string|false"),
        ),
        php_builtin_reflection(
            "rawurlencode",
            &[("string", Some("string"))],
            Some("string"),
        ),
        php_builtin_reflection(
            "rawurldecode",
            &[("string", Some("string"))],
            Some("string"),
        ),
        php_builtin_reflection("urlencode", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("urldecode", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection(
            "hex2bin",
            &[("string", Some("string"))],
            Some("string|false"),
        ),
        php_builtin_reflection("escapeshellarg", &[("arg", Some("string"))], Some("string")),
        php_builtin_reflection(
            "escapeshellcmd",
            &[("command", Some("string"))],
            Some("string"),
        ),
        php_builtin_reflection(
            "explode",
            &[
                ("separator", Some("string")),
                ("string", Some("string")),
                ("limit", Some("int")),
            ],
            Some("array"),
        ),
        php_builtin_reflection("file_exists", &[("filename", Some("string"))], Some("bool")),
        php_builtin_reflection("is_dir", &[("filename", Some("string"))], Some("bool")),
        php_builtin_reflection("is_file", &[("filename", Some("string"))], Some("bool")),
        php_builtin_reflection("is_link", &[("filename", Some("string"))], Some("bool")),
        php_builtin_reflection("is_readable", &[("filename", Some("string"))], Some("bool")),
        php_builtin_reflection("is_writable", &[("filename", Some("string"))], Some("bool")),
        php_builtin_reflection(
            "is_writeable",
            &[("filename", Some("string"))],
            Some("bool"),
        ),
        php_builtin_reflection(
            "is_executable",
            &[("filename", Some("string"))],
            Some("bool"),
        ),
        php_builtin_reflection(
            "filesize",
            &[("filename", Some("string"))],
            Some("int|false"),
        ),
        php_builtin_reflection(
            "realpath",
            &[("path", Some("string"))],
            Some("string|false"),
        ),
        php_builtin_reflection("trim", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("ltrim", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("rtrim", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection("addslashes", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection(
            "stripslashes",
            &[("string", Some("string"))],
            Some("string"),
        ),
        php_builtin_reflection("quotemeta", &[("string", Some("string"))], Some("string")),
        php_builtin_reflection(
            "str_contains",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("bool"),
        ),
        php_builtin_reflection(
            "str_starts_with",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("bool"),
        ),
        php_builtin_reflection(
            "str_ends_with",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("bool"),
        ),
        php_builtin_reflection(
            "str_repeat",
            &[("string", Some("string")), ("times", Some("int"))],
            Some("string"),
        ),
        php_builtin_reflection(
            "substr",
            &[("string", Some("string")), ("offset", Some("int"))],
            Some("string"),
        ),
        php_builtin_reflection(
            "strpos",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("int|false"),
        ),
        php_builtin_reflection(
            "stripos",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("int|false"),
        ),
        php_builtin_reflection(
            "strrpos",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("int|false"),
        ),
        php_builtin_reflection(
            "strripos",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("int|false"),
        ),
        php_builtin_reflection(
            "strstr",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("string|false"),
        ),
        php_builtin_reflection(
            "strchr",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("string|false"),
        ),
        php_builtin_reflection(
            "stristr",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("string|false"),
        ),
        php_builtin_reflection(
            "strrchr",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("string|false"),
        ),
        php_builtin_reflection(
            "strpbrk",
            &[("string", Some("string")), ("characters", Some("string"))],
            Some("string|false"),
        ),
        php_builtin_reflection(
            "strspn",
            &[("string", Some("string")), ("characters", Some("string"))],
            Some("int"),
        ),
        php_builtin_reflection(
            "strcspn",
            &[("string", Some("string")), ("characters", Some("string"))],
            Some("int"),
        ),
        php_builtin_reflection(
            "substr_count",
            &[("haystack", Some("string")), ("needle", Some("string"))],
            Some("int"),
        ),
        php_builtin_reflection(
            "substr_compare",
            &[
                ("haystack", Some("string")),
                ("needle", Some("string")),
                ("offset", Some("int")),
                ("length", Some("int|null")),
                ("case_insensitive", Some("bool")),
            ],
            Some("int"),
        ),
        php_builtin_reflection(
            "strcmp",
            &[("string1", Some("string")), ("string2", Some("string"))],
            Some("int"),
        ),
        php_builtin_reflection(
            "strcasecmp",
            &[("string1", Some("string")), ("string2", Some("string"))],
            Some("int"),
        ),
        php_builtin_reflection(
            "strncmp",
            &[
                ("string1", Some("string")),
                ("string2", Some("string")),
                ("length", Some("int")),
            ],
            Some("int"),
        ),
        php_builtin_reflection(
            "strncasecmp",
            &[
                ("string1", Some("string")),
                ("string2", Some("string")),
                ("length", Some("int")),
            ],
            Some("int"),
        ),
    ]
    .into()
}

fn php_builtin_reflection(
    name: &str,
    params: &[(&str, Option<&str>)],
    return_type: Option<&str>,
) -> FunctionReflection {
    FunctionReflection {
        name: name.to_string(),
        qualified_name: name.to_string(),
        source: FunctionSource::PhpBuiltin,
        params: params
            .iter()
            .map(|(name, ty)| ParamReflection {
                name: (*name).to_string(),
                ty: ty.map(str::to_string),
            })
            .collect(),
        return_type: return_type.map(str::to_string),
        is_intrinsic: true,
    }
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
    fn reflects_php_builtin_signatures_from_global_registry() {
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

        let type_of = function("reflect.typeOf").expect("reflect.typeOf reflected");
        assert_eq!(type_of.params_signature(), "mixed $value");
        assert_eq!(type_of.return_type_signature(), "string");
    }

    #[test]
    fn does_not_reflect_std_reflection_api_as_php_builtins() {
        assert!(php_builtin("params").is_none());
        assert!(php_builtin("returnType").is_none());
        assert!(function("reflect.params").is_some());
    }
}
