use super::*;

#[cfg(unix)]
#[test]
fn filesystem_link_builtins_create_and_read_links() {
    let temp_dir = std::env::temp_dir().join(format!("echo-runtime-link-{}", std::process::id()));
    let target_path = temp_dir.join("target.txt");
    let symlink_path = temp_dir.join("target-link.txt");
    let hard_link_path = temp_dir.join("target-hard.txt");
    let missing_path = temp_dir.join("missing-link.txt");
    std::fs::remove_dir_all(&temp_dir).ok();
    std::fs::create_dir_all(&temp_dir).expect("create temp test directory");
    std::fs::write(&target_path, b"target").expect("write link target");

    fn path_value(path: &Path) -> EchoValue {
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: path.to_string_lossy().as_bytes().to_vec(),
        })))
    }

    let target = path_value(&target_path);
    let symlink = path_value(&symlink_path);
    let hard_link = path_value(&hard_link_path);
    let missing = path_value(&missing_path);

    assert_eq!(echo_php_symlink(target, symlink), EchoValue::bool(true));
    assert_eq!(echo_php_is_link(symlink), EchoValue::bool(true));
    assert_eq!(
        echo_php_readlink(symlink).string_bytes(),
        Some(target_path.to_string_lossy().as_bytes().to_vec())
    );
    assert_eq!(echo_php_link(target, hard_link), EchoValue::bool(true));
    assert_eq!(echo_php_is_link(hard_link), EchoValue::bool(false));
    assert_eq!(echo_php_file_exists(hard_link), EchoValue::bool(true));
    assert_eq!(echo_php_readlink(missing), EchoValue::bool(false));

    std::fs::remove_dir_all(&temp_dir).ok();
}
