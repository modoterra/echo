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
];

pub const INTRINSICS: &[IntrinsicBinding] = &[
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
    fn exposes_net_intrinsic_bindings() {
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
}
