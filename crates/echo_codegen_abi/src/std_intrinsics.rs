use super::RuntimeSignature;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StdIntrinsic {
    pub echo_name: &'static str,
    pub symbol: &'static str,
    pub signature: RuntimeSignature,
    pub arity: usize,
}

impl StdIntrinsic {
    pub fn llvm_decl(self) -> String {
        self.signature.llvm_decl(self.symbol)
    }
}

pub const STD_INTRINSICS: &[StdIntrinsic] = &[
    StdIntrinsic {
        echo_name: "assert.ok",
        symbol: "echo_std_assert_ok",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "assert.equals",
        symbol: "echo_std_assert_equals",
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        arity: 2,
    },
    StdIntrinsic {
        echo_name: "http.responseText",
        symbol: "echo_std_http_response_text",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "http.readRequest",
        symbol: "echo_std_http_read_request",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "net.listen",
        symbol: "echo_std_net_listen",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "net.connect",
        symbol: "echo_std_net_connect",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "net.accept",
        symbol: "echo_std_net_accept",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "net.read",
        symbol: "echo_std_net_read",
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        arity: 2,
    },
    StdIntrinsic {
        echo_name: "net.write",
        symbol: "echo_std_net_write",
        signature: RuntimeSignature::EchoValueEchoValueEchoValue,
        arity: 2,
    },
    StdIntrinsic {
        echo_name: "net.close",
        symbol: "echo_std_net_close",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "reflect.exists",
        symbol: "echo_std_reflect_exists",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "reflect.params",
        symbol: "echo_std_reflect_params",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "reflect.returnType",
        symbol: "echo_std_reflect_return_type",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
    StdIntrinsic {
        echo_name: "reflect.typeOf",
        symbol: "echo_std_reflect_type_of",
        signature: RuntimeSignature::EchoValueEchoValue,
        arity: 1,
    },
];

pub fn std_intrinsic(name: &str) -> Option<StdIntrinsic> {
    STD_INTRINSICS
        .iter()
        .copied()
        .find(|intrinsic| intrinsic.echo_name == name)
}
