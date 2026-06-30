use super::*;
use std::path::PathBuf;

fn temp_file(path_fragment: &str, contents: &[u8]) -> (PathBuf, EchoValue) {
    let temp_dir =
        std::env::temp_dir().join(format!("echo-runtime-runtime-hash-{}", std::process::id()));
    let path = temp_dir.join(path_fragment);
    std::fs::create_dir_all(&temp_dir).expect("create temp test directory");
    std::fs::write(&path, contents).expect("write temp test file");
    (
        path.clone(),
        EchoValue::string(Box::into_raw(Box::new(EchoString {
            bytes: path.to_string_lossy().as_bytes().to_vec(),
        }))),
    )
}

fn array_entries(array: EchoValue) -> Vec<Vec<u8>> {
    let Some(array) = (unsafe { (array.payload as *const EchoArray).as_ref() }) else {
        return Vec::new();
    };
    array
        .values
        .iter()
        .map(|value| value.string_bytes().unwrap_or_default())
        .collect()
}

#[test]
fn file_hash_builtins_match_digests_and_fail_on_missing_file() {
    let (_path, path_value) = temp_file("file-hashes.txt", b"Echo\nPHP");
    let md5_hex = echo_php_md5(test_string_value(b"Echo\nPHP"), EchoValue::bool(false));
    let md5_raw = echo_php_md5(test_string_value(b"Echo\nPHP"), EchoValue::bool(true));
    let sha1_hex = echo_php_sha1(test_string_value(b"Echo\nPHP"), EchoValue::bool(false));
    let sha1_raw = echo_php_sha1(test_string_value(b"Echo\nPHP"), EchoValue::bool(true));

    let md5_file = echo_php_md5_file(path_value, EchoValue::bool(false), EchoValue::null());
    let md5_file_raw = echo_php_md5_file(path_value, EchoValue::bool(true), EchoValue::null());
    let sha1_file = echo_php_sha1_file(path_value, EchoValue::bool(false), EchoValue::null());
    let sha1_file_raw = echo_php_sha1_file(path_value, EchoValue::bool(true), EchoValue::null());

    assert_eq!(md5_file.string_bytes(), md5_hex.string_bytes());
    assert_eq!(md5_file_raw.string_bytes(), md5_raw.string_bytes());
    assert_eq!(sha1_file.string_bytes(), sha1_hex.string_bytes());
    assert_eq!(sha1_file_raw.string_bytes(), sha1_raw.string_bytes());

    std::fs::remove_dir_all(_path.parent().unwrap()).ok();

    let missing = test_string_value(b"/tmp/echo-runtime-does-not-exist");
    assert_eq!(
        echo_php_md5_file(missing, EchoValue::bool(false), EchoValue::null()),
        EchoValue::bool(false)
    );
}

#[test]
fn hash_dispatches_algorithms_and_rejects_unknown() {
    let data = test_string_value(b"message");
    let known = [
        ("md5", hash_output_len_bytes("md5")),
        ("sha1", hash_output_len_bytes("sha1")),
        ("sha256", hash_output_len_bytes("sha256")),
        ("sha3-256", hash_output_len_bytes("sha3-256")),
    ];

    for (algorithm, expected) in known {
        let value = echo_php_hash(
            test_string_value(algorithm.as_bytes()),
            data,
            EchoValue::bool(false),
        );
        assert_eq!(value.string_bytes().unwrap().len(), expected);
    }

    assert_eq!(
        echo_php_hash(
            test_string_value(b"does-not-exist"),
            test_string_value(b"x"),
            EchoValue::bool(false)
        ),
        EchoValue::bool(false)
    );
}

#[test]
fn hash_file_and_incremental_context_roundtrip() {
    let (path, path_value) = temp_file("hash-file.txt", b"hello world");
    let file_hash = echo_php_hash_file(
        test_string_value(b"sha256"),
        path_value,
        EchoValue::bool(false),
    );

    let context = echo_php_hash_init(
        test_string_value(b"sha256"),
        EchoValue::int(0),
        EchoValue::null(),
    );
    assert_eq!(
        echo_php_hash_update(context, test_string_value(b"hello ")),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_hash_update(context, test_string_value(b"world")),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_hash_final(context, EchoValue::bool(false)).string_bytes(),
        file_hash.string_bytes()
    );

    assert_eq!(
        echo_php_hash_final(context, EchoValue::bool(false)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_hash_update(context, test_string_value(b"extra")),
        EchoValue::bool(false)
    );

    let update_result = echo_php_hash_update_file(
        echo_php_hash_init(
            test_string_value(b"sha256"),
            EchoValue::int(0),
            EchoValue::null(),
        ),
        path_value,
        EchoValue::null(),
    );
    assert_eq!(update_result, EchoValue::bool(true));

    std::fs::remove_dir_all(path.parent().unwrap()).ok();
}

#[test]
fn hash_hmac_and_copy_contexts() {
    let key = test_string_value(b"secret");
    let data = test_string_value(b"message");

    let hmac = echo_php_hash_hmac(
        test_string_value(b"sha256"),
        data,
        key,
        EchoValue::bool(false),
    );
    assert_eq!(
        hmac.string_bytes().unwrap().len(),
        hash_output_len_bytes("sha256")
    );

    assert_eq!(
        echo_php_hash_hmac(
            test_string_value(b"crc32"),
            data,
            key,
            EchoValue::bool(false)
        ),
        EchoValue::bool(false)
    );

    let hmac_file = echo_php_hash_hmac_file(
        test_string_value(b"sha256"),
        {
            let (_file_path, file_value) = temp_file("hash-hmac.txt", b"message");
            file_value
        },
        key,
        EchoValue::bool(false),
    );
    assert_eq!(hmac_file.string_bytes(), hmac.string_bytes());

    let mut hmac_algos = array_entries(echo_php_hash_hmac_algos());
    hmac_algos.sort();
    assert!(hmac_algos.contains(&b"sha256".to_vec()));
    assert!(!hmac_algos.contains(&b"crc32".to_vec()));

    let copy_source = echo_php_hash_init(
        test_string_value(b"sha256"),
        EchoValue::int(0),
        EchoValue::null(),
    );
    assert!(copy_source.is_object());
    assert_eq!(
        echo_php_hash_update(copy_source, test_string_value(b"left-")),
        EchoValue::bool(true)
    );

    let copy = echo_php_hash_copy(copy_source);
    assert!(copy.is_object());

    assert_eq!(
        echo_php_hash_update(copy_source, test_string_value(b"right")),
        EchoValue::bool(true)
    );
    let left = echo_php_hash_final(copy_source, EchoValue::bool(false));

    assert_eq!(
        echo_php_hash_update(copy, test_string_value(b"right")),
        EchoValue::bool(true)
    );
    let right = echo_php_hash_final(copy, EchoValue::bool(false));
    assert_eq!(left.string_bytes(), right.string_bytes());

    assert_eq!(
        echo_php_hash_copy(test_string_value(b"not-an-object")),
        EchoValue::bool(false)
    );
}

#[test]
fn hash_pbkdf2_and_hkdf_have_reasonable_shape() {
    let password = test_string_value(b"password");
    let salt = test_string_value(b"salt");

    let pbkdf2_default = echo_php_hash_pbkdf2(
        test_string_value(b"sha256"),
        password,
        salt,
        EchoValue::int(2),
        EchoValue::int(0),
        EchoValue::bool(false),
    );
    assert_eq!(pbkdf2_default.string_bytes().unwrap().len(), 64);

    let pbkdf2_repeat = echo_php_hash_pbkdf2(
        test_string_value(b"sha256"),
        password,
        salt,
        EchoValue::int(2),
        EchoValue::int(0),
        EchoValue::bool(false),
    );
    assert_eq!(pbkdf2_default.string_bytes(), pbkdf2_repeat.string_bytes());

    let pbkdf2_raw = echo_php_hash_pbkdf2(
        test_string_value(b"sha256"),
        password,
        salt,
        EchoValue::int(2),
        EchoValue::int(16),
        EchoValue::bool(true),
    );
    assert_eq!(pbkdf2_raw.string_bytes().unwrap().len(), 16);

    assert_eq!(
        echo_php_hash_pbkdf2(
            test_string_value(b"sha256"),
            password,
            salt,
            EchoValue::int(0),
            EchoValue::int(16),
            EchoValue::bool(false)
        ),
        EchoValue::bool(false)
    );

    let hkdf_default = echo_php_hash_hkdf(
        test_string_value(b"sha256"),
        test_string_value(b"input-key-material"),
        EchoValue::int(0),
        test_string_value(b"context-info"),
        test_string_value(b""),
        EchoValue::bool(false),
    );
    assert_eq!(hkdf_default.string_bytes().unwrap().len(), 64);

    let hkdf_raw = echo_php_hash_hkdf(
        test_string_value(b"sha256"),
        test_string_value(b"input-key-material"),
        EchoValue::int(16),
        test_string_value(b"context-info"),
        test_string_value(b""),
        EchoValue::bool(true),
    );
    assert_eq!(hkdf_raw.string_bytes().unwrap().len(), 16);

    assert_eq!(
        echo_php_hash_hkdf(
            test_string_value(b"sha256"),
            test_string_value(b"input-key-material"),
            EchoValue::int(-1),
            test_string_value(b""),
            test_string_value(b""),
            EchoValue::bool(false)
        ),
        EchoValue::bool(false)
    );
}

#[test]
fn hash_equals_and_update_stream_read_from_resources() {
    let bytes = b"abc\x00def";
    assert_eq!(
        echo_php_hash_equals(test_string_value(bytes), test_string_value(bytes)),
        EchoValue::bool(true)
    );
    assert_eq!(
        echo_php_hash_equals(test_string_value(bytes), test_string_value(b"abc\x00de")),
        EchoValue::bool(false)
    );

    let (file_path, file_value) = temp_file("hash-update-stream.txt", b"hash stream payload");
    let stream = echo_php_fopen(
        file_value,
        test_string_value(b"r"),
        EchoValue::bool(false),
        EchoValue::null(),
    );
    let context = echo_php_hash_init(
        test_string_value(b"sha256"),
        EchoValue::int(0),
        EchoValue::null(),
    );
    assert_eq!(
        echo_php_hash_update_stream(context, stream, EchoValue::null()),
        EchoValue::bool(true)
    );
    assert_eq!(echo_php_fclose(stream), EchoValue::bool(true));

    let expected = echo_php_hash(
        test_string_value(b"sha256"),
        test_string_value(b"hash stream payload"),
        EchoValue::bool(false),
    );
    assert_eq!(
        echo_php_hash_final(context, EchoValue::bool(false)).string_bytes(),
        expected.string_bytes()
    );

    let stream = echo_php_fopen(
        file_value,
        test_string_value(b"r"),
        EchoValue::bool(false),
        EchoValue::null(),
    );
    let context = echo_php_hash_init(
        test_string_value(b"sha256"),
        EchoValue::int(0),
        EchoValue::null(),
    );
    assert_eq!(
        echo_php_hash_update_stream(context, stream, EchoValue::int(4)),
        EchoValue::bool(true)
    );
    let expected = echo_php_hash(
        test_string_value(b"sha256"),
        test_string_value(b"hash"),
        EchoValue::bool(false),
    );
    assert_eq!(
        echo_php_hash_final(context, EchoValue::bool(false)).string_bytes(),
        expected.string_bytes()
    );
    assert_eq!(echo_php_fclose(stream), EchoValue::bool(true));
    std::fs::remove_dir_all(file_path.parent().unwrap()).ok();

    let context = echo_php_hash_init(
        test_string_value(b"sha256"),
        EchoValue::int(0),
        EchoValue::null(),
    );
    assert_eq!(
        echo_php_hash_update_stream(context, EchoValue::null(), EchoValue::int(0)),
        EchoValue::bool(false)
    );
    assert_eq!(
        echo_php_hash_update_stream(
            context,
            test_string_value(b"not-a-stream"),
            EchoValue::int(0)
        ),
        EchoValue::bool(false)
    );
}

#[test]
fn crypt_supports_md5_and_sha_prefixes() {
    let password = test_string_value(b"hunter2");

    let md5_salt = test_string_value(b"$1$abc$");
    let sha256_salt = test_string_value(b"$5$hello$");
    let sha512_salt = test_string_value(b"$6$hello$");

    let md5_hash = echo_php_crypt(password, md5_salt);
    let sha256_hash = echo_php_crypt(password, sha256_salt);
    let sha512_hash = echo_php_crypt(password, sha512_salt);

    assert!(md5_hash.string_bytes().is_some());
    assert!(sha256_hash.string_bytes().is_some());
    assert!(sha512_hash.string_bytes().is_some());

    assert_eq!(
        echo_php_crypt(password, md5_hash).string_bytes(),
        md5_hash.string_bytes(),
        "md5 crypt should be stable for the same salt"
    );
    assert_eq!(
        echo_php_crypt(password, sha256_hash).string_bytes(),
        sha256_hash.string_bytes(),
        "sha256 crypt should be stable for the same salt"
    );
    assert_eq!(
        echo_php_crypt(password, sha512_hash).string_bytes(),
        sha512_hash.string_bytes(),
        "sha512 crypt should be stable for the same salt"
    );

    let md5_hash = md5_hash.string_bytes().unwrap_or_default();
    let sha256_hash = sha256_hash.string_bytes().unwrap_or_default();
    let sha512_hash = sha512_hash.string_bytes().unwrap_or_default();

    assert!(md5_hash.starts_with(b"$1$"));
    assert!(sha256_hash.starts_with(b"$5$"));
    assert!(sha512_hash.starts_with(b"$6$"));

    let des_hash = echo_php_crypt(password, test_string_value(b"AB"));
    let des_hash_bytes = des_hash.string_bytes().unwrap_or_default();
    assert_eq!(des_hash_bytes.len(), 13);
    assert_eq!(
        echo_php_crypt(password, des_hash).string_bytes(),
        Some(des_hash_bytes),
        "traditional DES crypt should be stable for the same salt"
    );
}

fn hash_output_len_bytes(algorithm: &str) -> usize {
    match algorithm {
        "md5" => 32,
        "sha1" => 40,
        "sha224" => 56,
        "sha256" => 64,
        "sha384" => 96,
        "sha512" => 128,
        "sha512/224" => 56,
        "sha512/256" => 64,
        "sha3-224" => 56,
        "sha3-256" => 64,
        "sha3-384" => 96,
        "sha3-512" => 128,
        _ => 0,
    }
}
