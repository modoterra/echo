//! Echo standard library boundary.
//!
//! `echo_std` is the home for Echo-facing standard library APIs. It should be
//! built on top of `echo_runtime` primitives and kept separate from PHP builtin
//! compatibility exports (`echo_php_*`) and future extension exports
//! (`echo_ext_*`).
//!
//! The first HTTP server should be expressed through this standard library, not
//! as an `xo serve` command.

pub mod net;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StdModule {
    pub name: &'static str,
    pub path: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicReceiver {
    Static,
    Instance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntrinsicBinding {
    pub owner: &'static str,
    pub method: &'static str,
    pub receiver: IntrinsicReceiver,
    pub intrinsic: &'static str,
    pub abi_symbol: &'static str,
}

pub const MODULES: &[StdModule] = &[
    StdModule {
        name: "std.net",
        path: "std/net.echo",
        source: include_str!("../../../std/net.echo"),
    },
    StdModule {
        name: "std.time",
        path: "std/time.echo",
        source: include_str!("../../../std/time.echo"),
    },
    StdModule {
        name: "std.http",
        path: "std/http.echo",
        source: include_str!("../../../std/http.echo"),
    },
    StdModule {
        name: "std.reflect",
        path: "std/reflect.echo",
        source: include_str!("../../../std/reflect.echo"),
    },
    StdModule {
        name: "std.php_builtins",
        path: "std/php_builtins.echo",
        source: include_str!("../../../std/php_builtins.echo"),
    },
];

pub const INTRINSICS: &[IntrinsicBinding] = &[
    IntrinsicBinding {
        owner: "std.http",
        method: "responseText",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.http.response_text",
        abi_symbol: "echo_std_http_response_text",
    },
    IntrinsicBinding {
        owner: "std.http",
        method: "readRequest",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.http.read_request",
        abi_symbol: "echo_std_http_read_request",
    },
    IntrinsicBinding {
        owner: "std.net",
        method: "listen",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.net.listen",
        abi_symbol: "echo_std_net_listen",
    },
    IntrinsicBinding {
        owner: "std.net",
        method: "connect",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.net.connect",
        abi_symbol: "echo_std_net_connect",
    },
    IntrinsicBinding {
        owner: "std.net",
        method: "accept",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.net.accept",
        abi_symbol: "echo_std_net_accept",
    },
    IntrinsicBinding {
        owner: "std.net",
        method: "read",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.net.read",
        abi_symbol: "echo_std_net_read",
    },
    IntrinsicBinding {
        owner: "std.net",
        method: "write",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.net.write",
        abi_symbol: "echo_std_net_write",
    },
    IntrinsicBinding {
        owner: "std.net",
        method: "close",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.net.close",
        abi_symbol: "echo_std_net_close",
    },
    IntrinsicBinding {
        owner: "std.net.TcpServer",
        method: "listen",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.net.tcp_server.listen",
        abi_symbol: "echo_std_net_tcp_server_listen",
    },
    IntrinsicBinding {
        owner: "std.net.TcpServer",
        method: "accept",
        receiver: IntrinsicReceiver::Instance,
        intrinsic: "std.net.tcp_server.accept",
        abi_symbol: "echo_std_net_tcp_server_accept",
    },
    IntrinsicBinding {
        owner: "std.net.TcpConnection",
        method: "read",
        receiver: IntrinsicReceiver::Instance,
        intrinsic: "std.net.tcp_connection.read",
        abi_symbol: "echo_std_net_tcp_connection_read",
    },
    IntrinsicBinding {
        owner: "std.net.TcpConnection",
        method: "write",
        receiver: IntrinsicReceiver::Instance,
        intrinsic: "std.net.tcp_connection.write",
        abi_symbol: "echo_std_net_tcp_connection_write",
    },
    IntrinsicBinding {
        owner: "std.net.TcpConnection",
        method: "close",
        receiver: IntrinsicReceiver::Instance,
        intrinsic: "std.net.tcp_connection.close",
        abi_symbol: "echo_std_net_tcp_connection_close",
    },
    IntrinsicBinding {
        owner: "std.time",
        method: "sleep",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.time.sleep",
        abi_symbol: "echo_time_sleep",
    },
    IntrinsicBinding {
        owner: "std.reflect",
        method: "exists",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.reflect.exists",
        abi_symbol: "echo_std_reflect_exists",
    },
    IntrinsicBinding {
        owner: "std.reflect",
        method: "params",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.reflect.params",
        abi_symbol: "echo_std_reflect_params",
    },
    IntrinsicBinding {
        owner: "std.reflect",
        method: "returnType",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.reflect.return_type",
        abi_symbol: "echo_std_reflect_return_type",
    },
    IntrinsicBinding {
        owner: "std.reflect",
        method: "typeOf",
        receiver: IntrinsicReceiver::Static,
        intrinsic: "std.reflect.type_of",
        abi_symbol: "echo_std_reflect_type_of",
    },
];

pub fn library_name() -> &'static str {
    "echo_std"
}

pub fn modules() -> &'static [StdModule] {
    MODULES
}

pub fn intrinsics() -> &'static [IntrinsicBinding] {
    INTRINSICS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packages_net_module_source() {
        let module = modules()
            .iter()
            .find(|module| module.name == "std.net")
            .expect("std.net module is packaged");

        assert_eq!(module.path, "std/net.echo");
        assert!(module.source.contains("namespace std net"));
        assert!(module.source.contains("intrinsic function listen"));
        assert!(module.source.contains("class TcpServer"));
        assert!(module.source.contains("intrinsic static function listen"));
    }

    #[test]
    fn packages_time_module_source() {
        let module = modules()
            .iter()
            .find(|module| module.name == "std.time")
            .expect("std.time module is packaged");

        assert_eq!(module.path, "std/time.echo");
        assert!(module.source.contains("namespace std time"));
        assert!(module.source.contains("intrinsic function sleep"));
    }

    #[test]
    fn packages_http_module_source() {
        let module = modules()
            .iter()
            .find(|module| module.name == "std.http")
            .expect("std.http module is packaged");

        assert_eq!(module.path, "std/http.echo");
        assert!(module.source.contains("namespace std http"));
        assert!(module.source.contains("intrinsic function responseText"));
    }

    #[test]
    fn packages_reflect_module_source() {
        let module = modules()
            .iter()
            .find(|module| module.name == "std.reflect")
            .expect("std.reflect module is packaged");

        assert_eq!(module.path, "std/reflect.echo");
        assert!(module.source.contains("namespace std reflect"));
        assert!(module.source.contains("intrinsic function params"));
        assert!(module.source.contains("intrinsic function returnType"));
        assert!(module.source.contains("intrinsic function typeOf"));
    }

    #[test]
    fn packages_php_builtins_module_source() {
        let module = modules()
            .iter()
            .find(|module| module.name == "std.php_builtins")
            .expect("std.php_builtins module is packaged");

        assert_eq!(module.path, "std/php_builtins.echo");
        assert!(module.source.contains("namespace std php_builtins"));
        assert!(module.source.contains("intrinsic function strlen"));
        assert!(module.source.contains("intrinsic function function_exists"));
    }

    #[test]
    fn exposes_net_intrinsic_bindings() {
        assert!(intrinsics().contains(&IntrinsicBinding {
            owner: "std.net",
            method: "connect",
            receiver: IntrinsicReceiver::Static,
            intrinsic: "std.net.connect",
            abi_symbol: "echo_std_net_connect",
        }));
        assert!(intrinsics().contains(&IntrinsicBinding {
            owner: "std.net.TcpServer",
            method: "listen",
            receiver: IntrinsicReceiver::Static,
            intrinsic: "std.net.tcp_server.listen",
            abi_symbol: "echo_std_net_tcp_server_listen",
        }));

        assert_eq!(
            intrinsics()
                .iter()
                .filter(|binding| binding.owner.starts_with("std.net."))
                .count(),
            5
        );
    }

    #[test]
    fn exposes_time_intrinsic_binding() {
        assert!(intrinsics().contains(&IntrinsicBinding {
            owner: "std.time",
            method: "sleep",
            receiver: IntrinsicReceiver::Static,
            intrinsic: "std.time.sleep",
            abi_symbol: "echo_time_sleep",
        }));
    }

    #[test]
    fn exposes_http_intrinsic_binding() {
        assert!(intrinsics().contains(&IntrinsicBinding {
            owner: "std.http",
            method: "responseText",
            receiver: IntrinsicReceiver::Static,
            intrinsic: "std.http.response_text",
            abi_symbol: "echo_std_http_response_text",
        }));
    }

    #[test]
    fn exposes_reflect_intrinsic_bindings() {
        assert!(intrinsics().contains(&IntrinsicBinding {
            owner: "std.reflect",
            method: "params",
            receiver: IntrinsicReceiver::Static,
            intrinsic: "std.reflect.params",
            abi_symbol: "echo_std_reflect_params",
        }));
        assert!(intrinsics().contains(&IntrinsicBinding {
            owner: "std.reflect",
            method: "returnType",
            receiver: IntrinsicReceiver::Static,
            intrinsic: "std.reflect.return_type",
            abi_symbol: "echo_std_reflect_return_type",
        }));
        assert!(intrinsics().contains(&IntrinsicBinding {
            owner: "std.reflect",
            method: "typeOf",
            receiver: IntrinsicReceiver::Static,
            intrinsic: "std.reflect.type_of",
            abi_symbol: "echo_std_reflect_type_of",
        }));
    }
}
