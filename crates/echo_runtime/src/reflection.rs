use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::{
    EchoValue, echo_runtime_string, echo_value_array_append, echo_value_array_new,
    echo_value_array_set,
};

pub(crate) const REFLECTION_SOURCE_PHP_BUILTIN: i32 = 1;
const REFLECTION_SOURCE_USERLAND: i32 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeFunctionReflection {
    pub(crate) name: String,
    pub(crate) params_signature: String,
    pub(crate) return_type: String,
    pub(crate) source_kind: i32,
}

pub(crate) fn function_reflection_for_value(value: EchoValue) -> Option<RuntimeFunctionReflection> {
    let bytes = value.string_bytes()?;
    let name = std::str::from_utf8(&bytes).ok()?;
    function_registry()
        .lock()
        .expect("function reflection registry should not be poisoned")
        .by_name(name)
}

pub(crate) fn function_reflection_by_name_and_source(
    name: &str,
    source_kind: i32,
) -> Option<RuntimeFunctionReflection> {
    function_registry()
        .lock()
        .expect("function reflection registry should not be poisoned")
        .by_name_and_source(name, source_kind)
}

pub(crate) fn function_reflection_by_name(name: &str) -> Option<RuntimeFunctionReflection> {
    function_registry()
        .lock()
        .expect("function reflection registry should not be poisoned")
        .by_name(name)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_function_exists(value: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::error();
    };
    let Ok(name) = std::str::from_utf8(&bytes) else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(
        function_reflection_by_name_and_source(name, REFLECTION_SOURCE_PHP_BUILTIN).is_some(),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_defined_functions(_exclude_disabled: EchoValue) -> EchoValue {
    let registry = function_registry()
        .lock()
        .expect("function reflection registry should not be poisoned");

    let mut internal = echo_value_array_new();
    for function in registry.names_by_source(REFLECTION_SOURCE_PHP_BUILTIN) {
        internal = echo_value_array_append(internal, echo_runtime_string(function.into_bytes()));
    }

    let mut user = echo_value_array_new();
    for function in registry.names_by_source(REFLECTION_SOURCE_USERLAND) {
        user = echo_value_array_append(user, echo_runtime_string(function.into_bytes()));
    }

    let mut result = echo_value_array_new();
    result = echo_value_array_set(result, echo_runtime_string(b"internal".to_vec()), internal);
    echo_value_array_set(result, echo_runtime_string(b"user".to_vec()), user)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_debug_backtrace() -> EchoValue {
    echo_value_array_new()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_declared_classes() -> EchoValue {
    echo_value_array_new()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_declared_interfaces() -> EchoValue {
    echo_value_array_new()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_get_declared_traits() -> EchoValue {
    echo_value_array_new()
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_class_exists(_class: EchoValue, _autoload: EchoValue) -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_interface_exists(
    _interface: EchoValue,
    _autoload: EchoValue,
) -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_trait_exists(_trait: EchoValue, _autoload: EchoValue) -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_enum_exists(_enum: EchoValue, _autoload: EchoValue) -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_method_exists(
    _object_or_class: EchoValue,
    _method: EchoValue,
) -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_property_exists(
    _object_or_class: EchoValue,
    _property: EchoValue,
) -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_a(
    _object_or_class: EchoValue,
    _class: EchoValue,
    _allow_string: EchoValue,
) -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_subclass_of(
    _object_or_class: EchoValue,
    _class: EchoValue,
    _allow_string: EchoValue,
) -> EchoValue {
    EchoValue::bool(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_callable(value: EchoValue) -> EchoValue {
    let Some(bytes) = value.string_bytes() else {
        return EchoValue::bool(false);
    };
    let Ok(name) = std::str::from_utf8(&bytes) else {
        return EchoValue::bool(false);
    };

    EchoValue::bool(function_reflection_by_name(name).is_some())
}

pub(crate) unsafe fn register_function_raw(
    name_ptr: *const u8,
    name_len: usize,
    params_ptr: *const u8,
    params_len: usize,
    return_type_ptr: *const u8,
    return_type_len: usize,
    source_kind: i32,
) {
    let Some(name) = runtime_utf8_arg(name_ptr, name_len) else {
        return;
    };
    let Some(params_signature) = runtime_utf8_arg(params_ptr, params_len) else {
        return;
    };
    let Some(return_type) = runtime_utf8_arg(return_type_ptr, return_type_len) else {
        return;
    };

    register_function(name, params_signature, return_type, source_kind);
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_reflect_exists(name: EchoValue) -> EchoValue {
    match function_reflection_for_value(name) {
        Some(_) => EchoValue::bool(true),
        None => EchoValue::bool(false),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_reflect_params(name: EchoValue) -> EchoValue {
    let params = function_reflection_for_value(name)
        .map(|function| function.params_signature)
        .unwrap_or_default();

    echo_runtime_string(params.into_bytes())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_reflect_return_type(name: EchoValue) -> EchoValue {
    let return_type = function_reflection_for_value(name)
        .map(|function| function.return_type)
        .unwrap_or_default();

    echo_runtime_string(return_type.into_bytes())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_reflection_register_function(
    name_ptr: *const u8,
    name_len: usize,
    params_ptr: *const u8,
    params_len: usize,
    return_type_ptr: *const u8,
    return_type_len: usize,
    source_kind: i32,
) {
    unsafe {
        register_function_raw(
            name_ptr,
            name_len,
            params_ptr,
            params_len,
            return_type_ptr,
            return_type_len,
            source_kind,
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_std_reflect_type_of(value: EchoValue) -> EchoValue {
    echo_runtime_string(value.type_name_bytes().to_vec())
}

fn register_function(
    name: String,
    params_signature: String,
    return_type: String,
    source_kind: i32,
) {
    let mut registry = function_registry()
        .lock()
        .expect("function reflection registry should not be poisoned");
    registry.insert(RuntimeFunctionReflection {
        name,
        params_signature,
        return_type,
        source_kind,
    });
}

#[derive(Debug, Default)]
struct FunctionReflectionRegistry {
    by_name: HashMap<String, RuntimeFunctionReflection>,
    by_name_and_source: HashMap<(String, i32), RuntimeFunctionReflection>,
}

impl FunctionReflectionRegistry {
    fn insert(&mut self, function: RuntimeFunctionReflection) {
        let normalized_name = normalize_function_name(&function.name);
        let key = (normalized_name.clone(), function.source_kind);

        match self.by_name.get_mut(&normalized_name) {
            Some(existing) if existing.source_kind == function.source_kind => {
                *existing = function.clone();
            }
            Some(_) => {}
            None => {
                self.by_name.insert(normalized_name, function.clone());
            }
        }
        self.by_name_and_source.insert(key, function);
    }

    fn by_name(&self, name: &str) -> Option<RuntimeFunctionReflection> {
        self.by_name.get(&normalize_function_name(name)).cloned()
    }

    fn by_name_and_source(
        &self,
        name: &str,
        source_kind: i32,
    ) -> Option<RuntimeFunctionReflection> {
        self.by_name_and_source
            .get(&(normalize_function_name(name), source_kind))
            .cloned()
    }

    fn names_by_source(&self, source_kind: i32) -> Vec<String> {
        let mut names = self
            .by_name_and_source
            .values()
            .filter(|function| function.source_kind == source_kind)
            .map(|function| function.name.clone())
            .collect::<Vec<_>>();
        names.sort_by_key(|name| normalize_function_name(name));
        names
    }
}

fn normalize_function_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn function_registry() -> &'static Mutex<FunctionReflectionRegistry> {
    static FUNCTION_REFLECTIONS: OnceLock<Mutex<FunctionReflectionRegistry>> = OnceLock::new();
    FUNCTION_REFLECTIONS.get_or_init(|| Mutex::new(FunctionReflectionRegistry::default()))
}

fn runtime_utf8_arg(ptr: *const u8, len: usize) -> Option<String> {
    if ptr.is_null() && len != 0 {
        return None;
    }

    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(bytes)
        .ok()
        .map(std::string::ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reflection(
        name: &str,
        params: &str,
        return_type: &str,
        source_kind: i32,
    ) -> RuntimeFunctionReflection {
        RuntimeFunctionReflection {
            name: name.to_string(),
            params_signature: params.to_string(),
            return_type: return_type.to_string(),
            source_kind,
        }
    }

    #[test]
    fn registry_lookup_is_case_insensitive_and_keyed_by_source() {
        let mut registry = FunctionReflectionRegistry::default();

        registry.insert(reflection(
            "strlen",
            "string $string",
            "int",
            REFLECTION_SOURCE_PHP_BUILTIN,
        ));

        assert_eq!(
            registry.by_name("STRLEN").expect("by name").return_type,
            "int"
        );
        assert_eq!(
            registry
                .by_name_and_source("STRLEN", REFLECTION_SOURCE_PHP_BUILTIN)
                .expect("by source")
                .params_signature,
            "string $string"
        );
        assert!(registry.by_name_and_source("strlen", 0).is_none());
    }

    #[test]
    fn registry_updates_existing_source_without_reordering_any_source_lookup() {
        let mut registry = FunctionReflectionRegistry::default();

        registry.insert(reflection(
            "fixture",
            "first",
            "int",
            REFLECTION_SOURCE_PHP_BUILTIN,
        ));
        registry.insert(reflection("fixture", "userland", "string", 0));
        registry.insert(reflection(
            "FIXTURE",
            "updated",
            "bool",
            REFLECTION_SOURCE_PHP_BUILTIN,
        ));

        assert_eq!(
            registry.by_name("fixture").expect("by name").return_type,
            "bool"
        );
        assert_eq!(
            registry
                .by_name_and_source("fixture", 0)
                .expect("userland")
                .return_type,
            "string"
        );
    }

    #[test]
    fn type_of_reports_runtime_value_category() {
        assert_eq!(
            echo_std_reflect_type_of(EchoValue::int(42)).string_bytes(),
            Some(b"int".to_vec())
        );
    }
}
