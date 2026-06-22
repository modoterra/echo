use crate::EchoSymbol;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EchoError {
    InvalidCallable,
    UndefinedFunction(EchoSymbol),
}
