//! Standard-library and runtime-primitive package facts.
//!
//! Authority: `docs/stdlib.md`. The `/ runtime` package is virtual (no `.echo`
//! tree); exports map to `echo_runtime_*` via [`echo_codegen_abi`].

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use echo_codegen_abi::{
    RT_BYTES_CAT, RT_BYTES_FROM_I64, RT_BYTES_FROM_STR, RT_BYTES_GET, RT_BYTES_LEN, RT_BYTES_SLICE,
    RT_FLOAT_FROM_F64, RT_FLOAT_TO_F64, RT_HTTP_HEADERS_COMPLETE, RT_HTTP_PARSE_REQUEST,
    RT_HTTP_REQUEST_COMPLETE, RT_LIST_GET, RT_LIST_LEN, RT_NOW_MS, RT_PRINT_I64,
    RT_PROCESS_ARGS, RT_PROCESS_ENV_GET, RT_PROCESS_ENV_HAS, RT_PROCESS_ENV_SET,
    RT_PROCESS_ENV_UNSET, RT_PROCESS_EXIT, RT_PROCESS_RUN, RT_REFLECT_KEY_BYTES, RT_REFLECT_KIND,
    RT_REFLECT_KIND_NAME, RT_SLEEP_MS, RT_STR_CAT, RT_STR_CONTAINS, RT_STR_ENDS_WITH,
    RT_STR_FROM_BYTES, RT_STR_FROM_DEBUG, RT_STR_FROM_DURATION, RT_STR_FROM_FLOAT, RT_STR_FROM_INT,
    RT_STR_FROM_LOCATOR, RT_STR_GET, RT_STR_LEN, RT_STR_SLICE, RT_STR_STARTS_WITH, RT_TCP_ACCEPT,
    RT_TCP_CLOSE, RT_TCP_CONNECT, RT_TCP_LISTEN, RT_TCP_READ, RT_TCP_WRITE, RT_TEST_FAIL,
    RT_TEST_FINISH, RT_TEST_REGISTER, RT_UDP_BIND, RT_UDP_CLOSE, RT_UDP_RECV_FROM, RT_UDP_SEND_TO,
};

/// Stable crate identity for workspace linkage checks.
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// Sentinel path for the privileged runtime-primitive package (not a real file).
pub const RUNTIME_MODULE_PATH: &str = "<echo:runtime>";

/// One export on the virtual `runtime` module.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeExport {
    pub name: &'static str,
    /// Native symbol (`echo_runtime_*`).
    pub native: &'static str,
}

/// Runtime package exports (extend as the ABI grows).
pub const RUNTIME_EXPORTS: &[RuntimeExport] = &[
    RuntimeExport {
        name: "print",
        native: RT_PRINT_I64,
    },
    RuntimeExport {
        name: "str_from_int",
        native: RT_STR_FROM_INT,
    },
    RuntimeExport {
        name: "str_from_float",
        native: RT_STR_FROM_FLOAT,
    },
    RuntimeExport {
        name: "str_from_bytes",
        native: RT_STR_FROM_BYTES,
    },
    RuntimeExport {
        name: "str_from_duration",
        native: RT_STR_FROM_DURATION,
    },
    RuntimeExport {
        name: "str_from_locator",
        native: RT_STR_FROM_LOCATOR,
    },
    RuntimeExport {
        name: "str_from_debug",
        native: RT_STR_FROM_DEBUG,
    },
    RuntimeExport {
        name: "str_len",
        native: RT_STR_LEN,
    },
    RuntimeExport {
        name: "bytes_len",
        native: RT_BYTES_LEN,
    },
    RuntimeExport {
        name: "list_len",
        native: RT_LIST_LEN,
    },
    RuntimeExport {
        name: "list_get",
        native: RT_LIST_GET,
    },
    RuntimeExport {
        name: "bytes_get",
        native: RT_BYTES_GET,
    },
    RuntimeExport {
        name: "bytes_from_i64",
        native: RT_BYTES_FROM_I64,
    },
    RuntimeExport {
        name: "bytes_slice",
        native: RT_BYTES_SLICE,
    },
    RuntimeExport {
        name: "bytes_cat",
        native: RT_BYTES_CAT,
    },
    RuntimeExport {
        name: "bytes_from_str",
        native: RT_BYTES_FROM_STR,
    },
    RuntimeExport {
        name: "str_get",
        native: RT_STR_GET,
    },
    RuntimeExport {
        name: "reflect_kind",
        native: RT_REFLECT_KIND,
    },
    RuntimeExport {
        name: "reflect_kind_name",
        native: RT_REFLECT_KIND_NAME,
    },
    RuntimeExport {
        name: "reflect_key_bytes",
        native: RT_REFLECT_KEY_BYTES,
    },
    RuntimeExport {
        name: "str_cat",
        native: RT_STR_CAT,
    },
    RuntimeExport {
        name: "str_slice",
        native: RT_STR_SLICE,
    },
    RuntimeExport {
        name: "str_contains",
        native: RT_STR_CONTAINS,
    },
    RuntimeExport {
        name: "str_starts_with",
        native: RT_STR_STARTS_WITH,
    },
    RuntimeExport {
        name: "str_ends_with",
        native: RT_STR_ENDS_WITH,
    },
    RuntimeExport {
        name: "float_from_f64",
        native: RT_FLOAT_FROM_F64,
    },
    RuntimeExport {
        name: "float_to_f64",
        native: RT_FLOAT_TO_F64,
    },
    RuntimeExport {
        name: "http_parse_request",
        native: RT_HTTP_PARSE_REQUEST,
    },
    RuntimeExport {
        name: "http_headers_complete",
        native: RT_HTTP_HEADERS_COMPLETE,
    },
    RuntimeExport {
        name: "http_request_complete",
        native: RT_HTTP_REQUEST_COMPLETE,
    },
    RuntimeExport {
        name: "tcp_listen",
        native: RT_TCP_LISTEN,
    },
    RuntimeExport {
        name: "tcp_accept",
        native: RT_TCP_ACCEPT,
    },
    RuntimeExport {
        name: "tcp_connect",
        native: RT_TCP_CONNECT,
    },
    RuntimeExport {
        name: "tcp_read",
        native: RT_TCP_READ,
    },
    RuntimeExport {
        name: "tcp_write",
        native: RT_TCP_WRITE,
    },
    RuntimeExport {
        name: "tcp_close",
        native: RT_TCP_CLOSE,
    },
    RuntimeExport {
        name: "udp_bind",
        native: RT_UDP_BIND,
    },
    RuntimeExport {
        name: "udp_send_to",
        native: RT_UDP_SEND_TO,
    },
    RuntimeExport {
        name: "udp_recv_from",
        native: RT_UDP_RECV_FROM,
    },
    RuntimeExport {
        name: "udp_close",
        native: RT_UDP_CLOSE,
    },
    RuntimeExport {
        name: "test_register",
        native: RT_TEST_REGISTER,
    },
    RuntimeExport {
        name: "test_fail",
        native: RT_TEST_FAIL,
    },
    RuntimeExport {
        name: "test_finish",
        native: RT_TEST_FINISH,
    },
    RuntimeExport {
        name: "now_ms",
        native: RT_NOW_MS,
    },
    RuntimeExport {
        name: "sleep_ms",
        native: RT_SLEEP_MS,
    },
    RuntimeExport {
        name: "process_args",
        native: RT_PROCESS_ARGS,
    },
    RuntimeExport {
        name: "process_env_has",
        native: RT_PROCESS_ENV_HAS,
    },
    RuntimeExport {
        name: "process_env_get",
        native: RT_PROCESS_ENV_GET,
    },
    RuntimeExport {
        name: "process_env_set",
        native: RT_PROCESS_ENV_SET,
    },
    RuntimeExport {
        name: "process_env_unset",
        native: RT_PROCESS_ENV_UNSET,
    },
    RuntimeExport {
        name: "process_exit",
        native: RT_PROCESS_EXIT,
    },
    RuntimeExport {
        name: "process_run",
        native: RT_PROCESS_RUN,
    },
];

#[must_use]
pub fn is_runtime_module_path(path: &Path) -> bool {
    path.to_str() == Some(RUNTIME_MODULE_PATH)
}

#[must_use]
pub fn runtime_module_path() -> PathBuf {
    PathBuf::from(RUNTIME_MODULE_PATH)
}

#[must_use]
pub fn runtime_native_symbol(export: &str) -> Option<&'static str> {
    RUNTIME_EXPORTS
        .iter()
        .find(|e| e.name == export)
        .map(|e| e.native)
}

/// True if `file` is under a privileged `std/` directory for the given package roots.
///
/// Package roots are parents of `std/` (same as resolver `SearchPaths`).
#[must_use]
pub fn is_under_privileged_std(file: &Path, package_roots: &[PathBuf]) -> bool {
    let Ok(file) = file.canonicalize() else {
        return false;
    };
    for root in package_roots {
        let std_dir = root.join("std");
        let Ok(std_canon) = std_dir.canonicalize() else {
            continue;
        };
        if file.starts_with(&std_canon) {
            return true;
        }
    }
    false
}

/// Import path is exactly `/ runtime` (single name segment `runtime`).
#[must_use]
pub fn is_runtime_import_segments(segments: &[impl AsRef<str>]) -> bool {
    matches!(segments, [s] if s.as_ref() == "runtime")
}

#[cfg(test)]
mod tests {
    use super::*;
    use echo_codegen_abi::{
        RT_LIST_LEN, RT_PRINT_I64, RT_STR_FROM_BYTES, RT_STR_FROM_DURATION, RT_STR_FROM_FLOAT,
        RT_STR_FROM_INT, RT_STR_FROM_LOCATOR, RT_STR_LEN,
    };

    #[test]
    fn runtime_print_and_str_maps() {
        assert_eq!(runtime_native_symbol("print"), Some(RT_PRINT_I64));
        assert_eq!(runtime_native_symbol("str_from_int"), Some(RT_STR_FROM_INT));
        assert_eq!(
            runtime_native_symbol("str_from_float"),
            Some(RT_STR_FROM_FLOAT)
        );
        assert_eq!(
            runtime_native_symbol("str_from_bytes"),
            Some(RT_STR_FROM_BYTES)
        );
        assert_eq!(
            runtime_native_symbol("str_from_duration"),
            Some(RT_STR_FROM_DURATION)
        );
        assert_eq!(
            runtime_native_symbol("str_from_locator"),
            Some(RT_STR_FROM_LOCATOR)
        );
        assert_eq!(runtime_native_symbol("str_len"), Some(RT_STR_LEN));
        assert_eq!(runtime_native_symbol("list_len"), Some(RT_LIST_LEN));
        assert_eq!(runtime_native_symbol("nope"), None);
    }

    #[test]
    fn runtime_path_sentinel() {
        assert!(is_runtime_module_path(&runtime_module_path()));
        assert!(!is_runtime_module_path(Path::new("/tmp/runtime.echo")));
    }
}
