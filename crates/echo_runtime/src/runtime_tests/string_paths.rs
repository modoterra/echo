use super::*;

#[test]
fn basename_preserves_php_unix_path_string_behavior() {
    let path = Box::into_raw(Box::new(EchoString {
        bytes: "/etc/sudoers.d".as_bytes().to_vec(),
    }));
    let suffix = Box::into_raw(Box::new(EchoString {
        bytes: ".d".as_bytes().to_vec(),
    }));
    let trailing = Box::into_raw(Box::new(EchoString {
        bytes: "/etc/".as_bytes().to_vec(),
    }));
    let root = Box::into_raw(Box::new(EchoString {
        bytes: "/".as_bytes().to_vec(),
    }));
    let dot = Box::into_raw(Box::new(EchoString {
        bytes: ".".as_bytes().to_vec(),
    }));
    let empty_suffix = Box::into_raw(Box::new(EchoString { bytes: Vec::new() }));
    let missing_suffix = Box::into_raw(Box::new(EchoString {
        bytes: ".txt".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_basename(EchoValue::string(path), EchoValue::string(suffix)).string_bytes(),
        Some("sudoers".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_basename(EchoValue::string(trailing), EchoValue::string(empty_suffix))
            .string_bytes(),
        Some("etc".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_basename(EchoValue::string(root), EchoValue::string(missing_suffix))
            .string_bytes(),
        Some(Vec::new())
    );
    assert_eq!(
        echo_php_basename(EchoValue::string(dot), EchoValue::string(empty_suffix)).string_bytes(),
        Some(".".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(path));
        drop(Box::from_raw(suffix));
        drop(Box::from_raw(trailing));
        drop(Box::from_raw(root));
        drop(Box::from_raw(dot));
        drop(Box::from_raw(empty_suffix));
        drop(Box::from_raw(missing_suffix));
    }
}

#[test]
fn dirname_preserves_php_unix_path_string_behavior() {
    let file = Box::into_raw(Box::new(EchoString {
        bytes: "/etc/passwd".as_bytes().to_vec(),
    }));
    let trailing = Box::into_raw(Box::new(EchoString {
        bytes: "/etc/".as_bytes().to_vec(),
    }));
    let root = Box::into_raw(Box::new(EchoString {
        bytes: "/".as_bytes().to_vec(),
    }));
    let dot = Box::into_raw(Box::new(EchoString {
        bytes: ".".as_bytes().to_vec(),
    }));
    let relative = Box::into_raw(Box::new(EchoString {
        bytes: "foo/bar/baz".as_bytes().to_vec(),
    }));
    let repeated = Box::into_raw(Box::new(EchoString {
        bytes: "foo//bar".as_bytes().to_vec(),
    }));
    let nested = Box::into_raw(Box::new(EchoString {
        bytes: "/usr/local/lib".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_dirname(EchoValue::string(file), EchoValue::int(1)).string_bytes(),
        Some("/etc".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(trailing), EchoValue::int(1)).string_bytes(),
        Some("/".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(root), EchoValue::int(1)).string_bytes(),
        Some("/".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(dot), EchoValue::int(1)).string_bytes(),
        Some(".".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(relative), EchoValue::int(1)).string_bytes(),
        Some("foo/bar".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(repeated), EchoValue::int(1)).string_bytes(),
        Some("foo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_dirname(EchoValue::string(nested), EchoValue::int(2)).string_bytes(),
        Some("/usr".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(file));
        drop(Box::from_raw(trailing));
        drop(Box::from_raw(root));
        drop(Box::from_raw(dot));
        drop(Box::from_raw(relative));
        drop(Box::from_raw(repeated));
        drop(Box::from_raw(nested));
    }
}
