use super::{
    ECHO_VALUE_ARRAY, ECHO_VALUE_BOOL, ECHO_VALUE_ERROR, ECHO_VALUE_FLOAT, ECHO_VALUE_INT,
    ECHO_VALUE_LIST, ECHO_VALUE_NULL, ECHO_VALUE_OBJECT, ECHO_VALUE_PROCESS, ECHO_VALUE_STRING,
    ECHO_VALUE_TASK, ECHO_VALUE_TASK_GROUP, ECHO_VALUE_THREAD, EchoString, EchoValue,
    format_php_float,
};
use crate::collections::{EchoArray, EchoArrayKey, EchoList};

const INSPECT_MAX_DEPTH: usize = 3;
const INSPECT_MAX_ITEMS: usize = 8;

impl EchoValue {
    pub(crate) fn inspect_bytes(self) -> Option<Vec<u8>> {
        Some(self.inspect_string(0).into_bytes())
    }

    fn inspect_string(self, depth: usize) -> String {
        if depth >= INSPECT_MAX_DEPTH {
            return match self.kind {
                ECHO_VALUE_ARRAY => "Array [...]".to_string(),
                ECHO_VALUE_LIST => "List [...]".to_string(),
                ECHO_VALUE_OBJECT => "Object {...}".to_string(),
                _ => self
                    .string_bytes()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or_default(),
            };
        }

        match self.kind {
            ECHO_VALUE_NULL | ECHO_VALUE_ERROR => String::new(),
            ECHO_VALUE_BOOL => {
                if self.payload == 0 {
                    String::new()
                } else {
                    "1".to_string()
                }
            }
            ECHO_VALUE_INT => (self.payload as i64).to_string(),
            ECHO_VALUE_FLOAT => format_php_float(f64::from_bits(self.payload)),
            ECHO_VALUE_STRING => unsafe {
                (self.payload as *const EchoString)
                    .as_ref()
                    .map(|value| {
                        if depth == 0 {
                            String::from_utf8_lossy(&value.bytes).into_owned()
                        } else {
                            inspect_string_literal(&value.bytes)
                        }
                    })
                    .unwrap_or_default()
            },
            ECHO_VALUE_ARRAY => unsafe {
                (self.payload as *const EchoArray)
                    .as_ref()
                    .map(|array| inspect_array(array, depth + 1))
                    .unwrap_or_else(|| "Array".to_string())
            },
            ECHO_VALUE_LIST => unsafe {
                (self.payload as *const EchoList)
                    .as_ref()
                    .map(|list| inspect_list(list, depth + 1))
                    .unwrap_or_else(|| "List".to_string())
            },
            ECHO_VALUE_TASK
            | ECHO_VALUE_TASK_GROUP
            | ECHO_VALUE_OBJECT
            | ECHO_VALUE_PROCESS
            | ECHO_VALUE_THREAD => "Object".to_string(),
            _ => String::new(),
        }
    }
}

fn inspect_array(array: &EchoArray, depth: usize) -> String {
    let mut parts = Vec::new();
    for (key, value) in array
        .keys
        .iter()
        .zip(array.values.iter())
        .take(INSPECT_MAX_ITEMS)
    {
        parts.push(format!(
            "{} => {}",
            inspect_array_key(key),
            value.inspect_string(depth)
        ));
    }

    if array.values.len() > INSPECT_MAX_ITEMS {
        parts.push(format!(
            "... {} more",
            array.values.len() - INSPECT_MAX_ITEMS
        ));
    }

    format!("Array [{}]", parts.join(", "))
}

fn inspect_list(list: &EchoList, depth: usize) -> String {
    let mut parts = Vec::new();
    for value in list.values.iter().take(INSPECT_MAX_ITEMS) {
        parts.push(value.inspect_string(depth));
    }

    if list.values.len() > INSPECT_MAX_ITEMS {
        parts.push(format!(
            "... {} more",
            list.values.len() - INSPECT_MAX_ITEMS
        ));
    }

    format!("List [{}]", parts.join(", "))
}

fn inspect_array_key(key: &EchoArrayKey) -> String {
    match key {
        EchoArrayKey::Int(value) => value.to_string(),
        EchoArrayKey::String(bytes) => inspect_string_literal(bytes),
    }
}

fn inspect_string_literal(bytes: &[u8]) -> String {
    let mut literal = String::from("\"");
    for byte in bytes {
        match byte {
            b'\\' => literal.push_str("\\\\"),
            b'"' => literal.push_str("\\\""),
            b'\n' => literal.push_str("\\n"),
            b'\r' => literal.push_str("\\r"),
            b'\t' => literal.push_str("\\t"),
            0x20..=0x7e => literal.push(*byte as char),
            _ => literal.push_str(&format!("\\x{byte:02x}")),
        }
    }
    literal.push('"');
    literal
}
