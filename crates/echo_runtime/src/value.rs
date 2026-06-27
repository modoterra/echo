use crate::collections::{EchoArray, EchoList, echo_arrays_equal, echo_lists_equal};
use crate::{echo_runtime_string, net, process, task, task_group, thread};

mod arithmetic;
mod coercion;
mod inspect;

pub use arithmetic::{
    echo_php_abs, echo_php_base_convert, echo_php_bindec, echo_php_hexdec, echo_php_octdec,
    echo_value_add, echo_value_div, echo_value_mod, echo_value_mul, echo_value_pow, echo_value_sub,
    echo_value_unary_minus, echo_value_unary_plus,
};
pub(crate) use arithmetic::{php_number_add, php_number_mul};
pub(crate) use coercion::{PhpNumber, format_php_float};
use coercion::{is_php_numeric_string, parse_php_decimal_int, php_float_cast};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EchoValue {
    pub kind: i32,
    pub payload: u64,
}

pub(crate) const ECHO_VALUE_NULL: i32 = 0;
pub(crate) const ECHO_VALUE_ERROR: i32 = -1;
pub(crate) const ECHO_VALUE_BOOL: i32 = 1;
pub(crate) const ECHO_VALUE_INT: i32 = 2;
pub(crate) const ECHO_VALUE_STRING: i32 = 3;
pub(crate) const ECHO_VALUE_ARRAY: i32 = 4;
pub(crate) const ECHO_VALUE_TASK: i32 = 5;
pub(crate) const ECHO_VALUE_PENDING: i32 = 6;
pub(crate) const ECHO_VALUE_TCP_LISTENER: i32 = 7;
pub(crate) const ECHO_VALUE_TCP_CONNECTION: i32 = 8;
pub(crate) const ECHO_VALUE_OBJECT: i32 = 9;
pub(crate) const ECHO_VALUE_LIST: i32 = 10;
pub(crate) const ECHO_VALUE_FLOAT: i32 = 11;
pub(crate) const ECHO_VALUE_PROCESS: i32 = 12;
pub(crate) const ECHO_VALUE_THREAD: i32 = 13;
pub(crate) const ECHO_VALUE_TASK_GROUP: i32 = 14;

impl EchoValue {
    pub const fn null() -> Self {
        Self {
            kind: ECHO_VALUE_NULL,
            payload: 0,
        }
    }

    pub const fn error() -> Self {
        Self {
            kind: ECHO_VALUE_ERROR,
            payload: 0,
        }
    }

    pub const fn bool(value: bool) -> Self {
        Self {
            kind: ECHO_VALUE_BOOL,
            payload: value as u64,
        }
    }

    pub const fn int(value: i64) -> Self {
        Self {
            kind: ECHO_VALUE_INT,
            payload: value as u64,
        }
    }

    pub const fn float(value: f64) -> Self {
        Self {
            kind: ECHO_VALUE_FLOAT,
            payload: value.to_bits(),
        }
    }

    pub const fn is_null(self) -> bool {
        self.kind == ECHO_VALUE_NULL
    }

    pub const fn is_false(self) -> bool {
        self.kind == ECHO_VALUE_BOOL && self.payload == 0
    }

    pub const fn is_bool(self) -> bool {
        self.kind == ECHO_VALUE_BOOL
    }

    pub const fn is_int(self) -> bool {
        self.kind == ECHO_VALUE_INT
    }

    pub const fn is_float(self) -> bool {
        self.kind == ECHO_VALUE_FLOAT
    }

    pub fn string(value: *mut EchoString) -> Self {
        Self {
            kind: ECHO_VALUE_STRING,
            payload: value as u64,
        }
    }

    pub fn task(value: *mut task::EchoTask) -> Self {
        Self {
            kind: ECHO_VALUE_TASK,
            payload: value as u64,
        }
    }

    pub fn task_group(value: *mut task_group::EchoTaskGroup) -> Self {
        Self {
            kind: ECHO_VALUE_TASK_GROUP,
            payload: value as u64,
        }
    }

    pub fn process(value: *mut process::EchoProcess) -> Self {
        Self {
            kind: ECHO_VALUE_PROCESS,
            payload: value as u64,
        }
    }

    pub fn thread(value: *mut thread::EchoThread) -> Self {
        Self {
            kind: ECHO_VALUE_THREAD,
            payload: value as u64,
        }
    }

    pub fn list(value: *mut EchoList) -> Self {
        Self {
            kind: ECHO_VALUE_LIST,
            payload: value as u64,
        }
    }

    pub fn array(value: *mut EchoArray) -> Self {
        Self {
            kind: ECHO_VALUE_ARRAY,
            payload: value as u64,
        }
    }

    pub fn object(value: *mut EchoObject) -> Self {
        Self {
            kind: ECHO_VALUE_OBJECT,
            payload: value as u64,
        }
    }

    pub const fn pending() -> Self {
        Self {
            kind: ECHO_VALUE_PENDING,
            payload: 0,
        }
    }

    pub fn tcp_listener(value: *mut net::EchoTcpListener) -> Self {
        Self {
            kind: ECHO_VALUE_TCP_LISTENER,
            payload: value as u64,
        }
    }

    pub fn tcp_connection(value: *mut net::EchoTcpConnection) -> Self {
        Self {
            kind: ECHO_VALUE_TCP_CONNECTION,
            payload: value as u64,
        }
    }

    pub(crate) fn string_bytes(self) -> Option<Vec<u8>> {
        match self.kind {
            ECHO_VALUE_NULL | ECHO_VALUE_ERROR => Some(Vec::new()),
            ECHO_VALUE_BOOL => {
                if self.payload == 0 {
                    Some(Vec::new())
                } else {
                    Some(b"1".to_vec())
                }
            }
            ECHO_VALUE_INT => Some((self.payload as i64).to_string().into_bytes()),
            ECHO_VALUE_FLOAT => Some(format_php_float(f64::from_bits(self.payload)).into_bytes()),
            ECHO_VALUE_STRING => unsafe {
                (self.payload as *const EchoString)
                    .as_ref()
                    .map(|value| value.bytes.clone())
            },
            ECHO_VALUE_ARRAY => Some(b"Array".to_vec()),
            ECHO_VALUE_LIST => Some(b"List".to_vec()),
            ECHO_VALUE_TASK
            | ECHO_VALUE_TASK_GROUP
            | ECHO_VALUE_OBJECT
            | ECHO_VALUE_PROCESS
            | ECHO_VALUE_THREAD => Some(b"Object".to_vec()),
            _ => None,
        }
    }

    pub(crate) fn int_value(self) -> Option<i64> {
        match self.kind {
            ECHO_VALUE_BOOL => Some(if self.payload == 0 { 0 } else { 1 }),
            ECHO_VALUE_INT => Some(self.payload as i64),
            ECHO_VALUE_FLOAT => Some(f64::from_bits(self.payload) as i64),
            ECHO_VALUE_STRING => unsafe {
                let bytes = &(self.payload as *const EchoString).as_ref()?.bytes;
                let text = std::str::from_utf8(bytes).ok()?.trim_ascii();
                text.parse::<i64>().ok()
            },
            _ => None,
        }
    }

    pub(crate) fn bool_value(self) -> Option<bool> {
        match self.kind {
            ECHO_VALUE_NULL | ECHO_VALUE_ERROR => Some(false),
            ECHO_VALUE_BOOL => Some(self.payload != 0),
            ECHO_VALUE_INT => Some(self.payload as i64 != 0),
            ECHO_VALUE_FLOAT => Some(f64::from_bits(self.payload) != 0.0),
            ECHO_VALUE_STRING => unsafe {
                let bytes = &(self.payload as *const EchoString).as_ref()?.bytes;
                Some(!bytes.is_empty() && bytes != b"0")
            },
            ECHO_VALUE_ARRAY | ECHO_VALUE_LIST => Some(true),
            ECHO_VALUE_TASK
            | ECHO_VALUE_TASK_GROUP
            | ECHO_VALUE_TCP_LISTENER
            | ECHO_VALUE_TCP_CONNECTION
            | ECHO_VALUE_PROCESS
            | ECHO_VALUE_THREAD => Some(true),
            ECHO_VALUE_PENDING => Some(false),
            _ => None,
        }
    }

    pub(crate) fn php_int_value(self) -> Option<i64> {
        match self.kind {
            ECHO_VALUE_NULL | ECHO_VALUE_ERROR => Some(0),
            ECHO_VALUE_BOOL => Some(if self.payload == 0 { 0 } else { 1 }),
            ECHO_VALUE_INT => Some(self.payload as i64),
            ECHO_VALUE_FLOAT => Some(f64::from_bits(self.payload) as i64),
            ECHO_VALUE_STRING => unsafe {
                let bytes = &(self.payload as *const EchoString).as_ref()?.bytes;
                Some(parse_php_decimal_int(bytes))
            },
            _ => None,
        }
    }

    pub const fn is_string(self) -> bool {
        self.kind == ECHO_VALUE_STRING
    }

    pub const fn is_array(self) -> bool {
        self.kind == ECHO_VALUE_ARRAY
    }

    pub const fn is_list(self) -> bool {
        self.kind == ECHO_VALUE_LIST
    }

    pub const fn is_object(self) -> bool {
        self.kind == ECHO_VALUE_OBJECT
    }

    pub const fn is_task(self) -> bool {
        self.kind == ECHO_VALUE_TASK
    }

    pub const fn is_pending(self) -> bool {
        self.kind == ECHO_VALUE_PENDING
    }

    pub(crate) const fn is_true_bool(self) -> bool {
        self.kind == ECHO_VALUE_BOOL && self.payload != 0
    }

    pub(crate) fn type_name_bytes(self) -> &'static [u8] {
        match self.kind {
            ECHO_VALUE_NULL => b"null".as_slice(),
            ECHO_VALUE_BOOL => b"bool".as_slice(),
            ECHO_VALUE_INT => b"int".as_slice(),
            ECHO_VALUE_FLOAT => b"float".as_slice(),
            ECHO_VALUE_STRING => b"string".as_slice(),
            ECHO_VALUE_ARRAY => b"array".as_slice(),
            ECHO_VALUE_LIST => b"list".as_slice(),
            ECHO_VALUE_TASK => b"task".as_slice(),
            ECHO_VALUE_TASK_GROUP => b"task_group".as_slice(),
            ECHO_VALUE_THREAD => b"thread".as_slice(),
            ECHO_VALUE_PROCESS => b"process".as_slice(),
            ECHO_VALUE_PENDING => b"pending".as_slice(),
            ECHO_VALUE_TCP_LISTENER => b"TcpServer".as_slice(),
            ECHO_VALUE_TCP_CONNECTION => b"TcpConnection".as_slice(),
            ECHO_VALUE_OBJECT => b"object".as_slice(),
            _ => b"unknown".as_slice(),
        }
    }

    pub(crate) fn as_task_mut(self) -> Option<&'static mut task::EchoTask> {
        if self.kind != ECHO_VALUE_TASK || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut task::EchoTask).as_mut() }
    }

    pub(crate) fn as_task_group_mut(self) -> Option<&'static mut task_group::EchoTaskGroup> {
        if self.kind != ECHO_VALUE_TASK_GROUP || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut task_group::EchoTaskGroup).as_mut() }
    }

    pub(crate) fn as_process_mut(self) -> Option<&'static mut process::EchoProcess> {
        if self.kind != ECHO_VALUE_PROCESS || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut process::EchoProcess).as_mut() }
    }

    pub(crate) fn as_thread_mut(self) -> Option<&'static mut thread::EchoThread> {
        if self.kind != ECHO_VALUE_THREAD || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut thread::EchoThread).as_mut() }
    }

    pub(crate) fn as_tcp_listener_ref(self) -> Option<&'static net::EchoTcpListener> {
        if self.kind != ECHO_VALUE_TCP_LISTENER || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *const net::EchoTcpListener).as_ref() }
    }

    pub(crate) fn as_tcp_connection_mut(self) -> Option<&'static mut net::EchoTcpConnection> {
        if self.kind != ECHO_VALUE_TCP_CONNECTION || self.payload == 0 {
            return None;
        }

        unsafe { (self.payload as *mut net::EchoTcpConnection).as_mut() }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_value_string_equals_ptr(
    value: EchoValue,
    expected_ptr: *const u8,
    expected_len: usize,
) -> bool {
    if expected_ptr.is_null() && expected_len != 0 {
        return false;
    }

    let Some(bytes) = value.string_bytes() else {
        return false;
    };
    let expected = unsafe { std::slice::from_raw_parts(expected_ptr, expected_len) };

    bytes == expected
}

pub(crate) fn echo_values_equal(left: EchoValue, right: EchoValue) -> bool {
    if left.kind != right.kind {
        return false;
    }

    match left.kind {
        ECHO_VALUE_NULL => true,
        ECHO_VALUE_BOOL | ECHO_VALUE_INT | ECHO_VALUE_FLOAT => left.payload == right.payload,
        ECHO_VALUE_STRING => left.string_bytes() == right.string_bytes(),
        ECHO_VALUE_ARRAY => echo_arrays_equal(left, right, echo_values_equal),
        ECHO_VALUE_LIST => echo_lists_equal(left, right, echo_values_equal),
        _ => left.payload == right.payload,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_identical(left: EchoValue, right: EchoValue) -> EchoValue {
    EchoValue::bool(echo_values_equal(left, right))
}

pub(crate) fn php_values_equal(left: EchoValue, right: EchoValue) -> bool {
    if let (Some(left), Some(right)) = (PhpNumber::coerce(left), PhpNumber::coerce(right)) {
        return left.as_float() == right.as_float();
    }

    match (left.string_bytes(), right.string_bytes()) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_gettype(value: EchoValue) -> EchoValue {
    let type_name = match value.kind {
        ECHO_VALUE_NULL => b"NULL".as_slice(),
        ECHO_VALUE_BOOL => b"boolean".as_slice(),
        ECHO_VALUE_INT => b"integer".as_slice(),
        ECHO_VALUE_FLOAT => b"double".as_slice(),
        ECHO_VALUE_STRING => b"string".as_slice(),
        ECHO_VALUE_ARRAY => b"array".as_slice(),
        ECHO_VALUE_LIST => b"list".as_slice(),
        ECHO_VALUE_TASK
        | ECHO_VALUE_TASK_GROUP
        | ECHO_VALUE_OBJECT
        | ECHO_VALUE_PROCESS
        | ECHO_VALUE_THREAD => b"object".as_slice(),
        ECHO_VALUE_TCP_LISTENER | ECHO_VALUE_TCP_CONNECTION => b"resource".as_slice(),
        _ => b"unknown type".as_slice(),
    };
    echo_runtime_string(type_name.to_vec())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_array(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_array())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_countable(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_array() || value.is_list())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_iterable(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_array() || value.is_list())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_null(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_null())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_bool(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_bool())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_int(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_int())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_float(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_float())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_object(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_object())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_resource(value: EchoValue) -> EchoValue {
    EchoValue::bool(matches!(
        value.kind,
        ECHO_VALUE_TCP_LISTENER
            | ECHO_VALUE_TCP_CONNECTION
            | ECHO_VALUE_PROCESS
            | ECHO_VALUE_TASK_GROUP
            | ECHO_VALUE_THREAD
    ))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_string(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_scalar(value: EchoValue) -> EchoValue {
    EchoValue::bool(value.is_bool() || value.is_int() || value.is_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_boolval(value: EchoValue) -> EchoValue {
    match value.bool_value() {
        Some(value) => EchoValue::bool(value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_intval(value: EchoValue) -> EchoValue {
    match value.php_int_value() {
        Some(value) => EchoValue::int(value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_floatval(value: EchoValue) -> EchoValue {
    match php_float_cast(value) {
        Some(value) => EchoValue::float(value),
        None => EchoValue::error(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_bool(value: EchoValue) -> bool {
    value.bool_value().unwrap_or(false)
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_not(value: EchoValue) -> EchoValue {
    EchoValue::bool(!value.bool_value().unwrap_or(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_less_than(left: EchoValue, right: EchoValue) -> EchoValue {
    let Some(left) = PhpNumber::coerce(left) else {
        return EchoValue::error();
    };
    let Some(right) = PhpNumber::coerce(right) else {
        return EchoValue::error();
    };
    EchoValue::bool(left.as_float() < right.as_float())
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_or(left: EchoValue, right: EchoValue) -> EchoValue {
    EchoValue::bool(left.bool_value().unwrap_or(false) || right.bool_value().unwrap_or(false))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_php_is_numeric(value: EchoValue) -> EchoValue {
    let is_numeric = match value.kind {
        ECHO_VALUE_INT => true,
        ECHO_VALUE_STRING => unsafe {
            (value.payload as *const EchoString)
                .as_ref()
                .is_some_and(|value| is_php_numeric_string(&value.bytes))
        },
        _ => false,
    };
    EchoValue::bool(is_numeric)
}

#[derive(Debug)]
pub struct EchoString {
    pub(crate) bytes: Vec<u8>,
}

impl EchoString {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

#[derive(Debug)]
pub struct EchoObject {
    pub(crate) fields: Vec<(String, EchoValue)>,
}

impl EchoObject {
    pub(crate) fn new() -> Self {
        Self { fields: Vec::new() }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_object_new() -> EchoValue {
    EchoValue::object(Box::into_raw(Box::new(EchoObject::new())))
}

#[unsafe(no_mangle)]
pub extern "C" fn echo_value_exit_status(value: EchoValue) -> i32 {
    match value.kind {
        ECHO_VALUE_INT => (value.payload as i64) as i32 & 0xff,
        ECHO_VALUE_BOOL if value.payload != 0 => 1,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_value_object_set(
    object: EchoValue,
    field_ptr: *const u8,
    field_len: usize,
    value: EchoValue,
) -> EchoValue {
    if object.kind != ECHO_VALUE_OBJECT || (field_ptr.is_null() && field_len != 0) {
        return EchoValue::error();
    }

    let Some(fields) = (unsafe { (object.payload as *mut EchoObject).as_mut() }) else {
        return EchoValue::error();
    };
    let field_bytes = unsafe { std::slice::from_raw_parts(field_ptr, field_len) };
    let Ok(field) = std::str::from_utf8(field_bytes) else {
        return EchoValue::error();
    };

    fields.fields.push((field.to_string(), value));
    object
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn echo_value_object_get(
    object: EchoValue,
    field_ptr: *const u8,
    field_len: usize,
) -> EchoValue {
    if object.kind != ECHO_VALUE_OBJECT || (field_ptr.is_null() && field_len != 0) {
        return EchoValue::error();
    }

    let Some(fields) = (unsafe { (object.payload as *const EchoObject).as_ref() }) else {
        return EchoValue::error();
    };
    let field_bytes = unsafe { std::slice::from_raw_parts(field_ptr, field_len) };
    let Ok(field) = std::str::from_utf8(field_bytes) else {
        return EchoValue::error();
    };

    fields
        .fields
        .iter()
        .rev()
        .find_map(|(name, value)| (name == field).then_some(*value))
        .unwrap_or_else(EchoValue::error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn php_float_formatting_matches_runtime_scalar_strings() {
        assert_eq!(format_php_float(12.5), "12.5");
        assert_eq!(format_php_float(12.0), "12");
        assert_eq!(format_php_float(f64::INFINITY), "INF");
        assert_eq!(format_php_float(f64::NEG_INFINITY), "-INF");
        assert_eq!(format_php_float(f64::NAN), "NAN");
    }
}
