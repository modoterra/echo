use super::*;
use crate::filesystem::{path_buf_from_bytes, path_is_dir, path_is_file};

#[test]
fn temporary_name_builtins_create_files_and_identifiers() {
    let temp_dir = std::env::temp_dir().join(format!(
        "echo-runtime-temporary-names-{}",
        std::process::id()
    ));
    std::fs::remove_dir_all(&temp_dir).ok();
    std::fs::create_dir_all(&temp_dir).expect("create temp test directory");

    fn string_value(bytes: &[u8]) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: bytes.to_vec(),
        })))
    }

    let sys_temp = echo_php_sys_get_temp_dir();
    let sys_temp_bytes = sys_temp.string_bytes().expect("temp dir string");
    assert!(path_is_dir(&sys_temp_bytes));

    let temp_file = echo_php_tempnam(
        string_value(temp_dir.to_string_lossy().as_bytes()),
        string_value(b"exo"),
    );
    let temp_file_bytes = temp_file.string_bytes().expect("tempnam string");
    assert!(path_is_file(&temp_file_bytes));
    assert!(
        path_buf_from_bytes(&temp_file_bytes)
            .and_then(|path| path.file_name().map(|name| name.to_owned()))
            .and_then(|name| name.into_string().ok())
            .is_some_and(|name| name.starts_with("exo"))
    );

    let plain = echo_php_uniqid(EchoValue::null(), EchoValue::bool(false))
        .string_bytes()
        .expect("uniqid string");
    let prefixed = echo_php_uniqid(string_value(b"job_"), EchoValue::bool(true))
        .string_bytes()
        .expect("prefixed uniqid string");
    assert_eq!(plain.len(), 13);
    assert_eq!(prefixed.len(), 27);
    assert!(prefixed.starts_with(b"job_"));

    std::fs::remove_file(path_buf_from_bytes(&temp_file_bytes).expect("tempnam path")).ok();
    std::fs::remove_dir_all(&temp_dir).ok();
}
