use super::*;

#[test]
fn trim_builtins_strip_default_php_ascii_whitespace() {
    let trim = Box::into_raw(Box::new(EchoString {
        bytes: "\t Echo \n".as_bytes().to_vec(),
    }));
    let ltrim = Box::into_raw(Box::new(EchoString {
        bytes: "\t Echo \n".as_bytes().to_vec(),
    }));
    let rtrim = Box::into_raw(Box::new(EchoString {
        bytes: "\t Echo \n".as_bytes().to_vec(),
    }));
    let non_ascii = Box::into_raw(Box::new(EchoString {
        bytes: " Ä ".as_bytes().to_vec(),
    }));

    assert_eq!(
        echo_php_trim(EchoValue::string(trim)).string_bytes(),
        Some("Echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_ltrim(EchoValue::string(ltrim)).string_bytes(),
        Some("Echo \n".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_rtrim(EchoValue::string(rtrim)).string_bytes(),
        Some("\t Echo".as_bytes().to_vec())
    );
    assert_eq!(
        echo_php_trim(EchoValue::string(non_ascii)).string_bytes(),
        Some("Ä".as_bytes().to_vec())
    );

    unsafe {
        drop(Box::from_raw(trim));
        drop(Box::from_raw(ltrim));
        drop(Box::from_raw(rtrim));
        drop(Box::from_raw(non_ascii));
    }
}

#[test]
fn string_rewrite_builtins_preserve_php_byte_behavior() {
    let chop = Box::into_raw(Box::new(EchoString {
        bytes: b"invoice:1001\n".to_vec(),
    }));
    let quoted = Box::into_raw(Box::new(EchoString {
        bytes: b"a=b\nnext".to_vec(),
    }));
    let quoted_decode = Box::into_raw(Box::new(EchoString {
        bytes: b"a=3Db=0Anext".to_vec(),
    }));
    let nl2br = Box::into_raw(Box::new(EchoString {
        bytes: b"line1\nline2".to_vec(),
    }));
    let search = Box::into_raw(Box::new(EchoString {
        bytes: b"{{name}}".to_vec(),
    }));
    let replace = Box::into_raw(Box::new(EchoString {
        bytes: b"Ada".to_vec(),
    }));
    let subject = Box::into_raw(Box::new(EchoString {
        bytes: b"Hello {{name}}".to_vec(),
    }));
    let isearch = Box::into_raw(Box::new(EchoString {
        bytes: b"TOKEN".to_vec(),
    }));
    let ireplace = Box::into_raw(Box::new(EchoString {
        bytes: b"redacted".to_vec(),
    }));
    let isubject = Box::into_raw(Box::new(EchoString {
        bytes: b"token TOKEN".to_vec(),
    }));
    let tr_value = Box::into_raw(Box::new(EchoString {
        bytes: b"abc-123".to_vec(),
    }));
    let tr_from = Box::into_raw(Box::new(EchoString {
        bytes: b"abc123".to_vec(),
    }));
    let tr_to = Box::into_raw(Box::new(EchoString {
        bytes: b"xyz789".to_vec(),
    }));

    assert_eq!(
        echo_php_rtrim(EchoValue::string(chop)).string_bytes(),
        Some(b"invoice:1001".to_vec())
    );
    assert_eq!(
        echo_php_quoted_printable_encode(EchoValue::string(quoted)).string_bytes(),
        Some(b"a=3Db=0Anext".to_vec())
    );
    assert_eq!(
        echo_php_quoted_printable_decode(EchoValue::string(quoted_decode)).string_bytes(),
        Some(b"a=b\nnext".to_vec())
    );
    assert_eq!(
        echo_php_nl2br(EchoValue::string(nl2br), EchoValue::bool(false)).string_bytes(),
        Some(b"line1<br>\nline2".to_vec())
    );
    assert_eq!(
        echo_php_htmlspecialchars(test_string_value(
            b"<a href=\"/?q=Tom & Jerry\">Tom's link</a>",
        ))
        .string_bytes(),
        Some(b"&lt;a href=&quot;/?q=Tom &amp; Jerry&quot;&gt;Tom&#039;s link&lt;/a&gt;".to_vec())
    );
    assert_eq!(
        echo_php_htmlspecialchars_decode(test_string_value(
            b"&lt;a href=&quot;/?q=Tom &amp; Jerry&quot;&gt;Tom&#039;s link&lt;/a&gt;",
        ))
        .string_bytes(),
        Some(b"<a href=\"/?q=Tom & Jerry\">Tom's link</a>".to_vec())
    );
    assert_eq!(
        echo_php_htmlspecialchars_decode(test_string_value(b"&copy; stays named")).string_bytes(),
        Some(b"&copy; stays named".to_vec())
    );
    assert_eq!(
        echo_php_strip_tags(test_string_value(b"<p>Hello <strong>Ada</strong></p>")).string_bytes(),
        Some(b"Hello Ada".to_vec())
    );
    assert_eq!(
        echo_php_strip_tags(test_string_value(b"Keep<!-- hidden -->Visible")).string_bytes(),
        Some(b"KeepVisible".to_vec())
    );
    assert_eq!(
        echo_php_strip_tags(test_string_value(b"A\0B")).string_bytes(),
        Some(b"AB".to_vec())
    );
    assert_eq!(
        echo_php_str_word_count(test_string_value(
            b"Invoice #A-100 shipped to O'Reilly-Smith on 2026-06-30",
        ))
        .int_value(),
        Some(6)
    );
    assert_eq!(
        echo_php_str_word_count(test_string_value(b"12345")).int_value(),
        Some(0)
    );
    assert_eq!(
        echo_php_str_replace(
            EchoValue::string(search),
            EchoValue::string(replace),
            EchoValue::string(subject),
        )
        .string_bytes(),
        Some(b"Hello Ada".to_vec())
    );
    assert_eq!(
        echo_php_str_ireplace(
            EchoValue::string(isearch),
            EchoValue::string(ireplace),
            EchoValue::string(isubject),
        )
        .string_bytes(),
        Some(b"redacted redacted".to_vec())
    );
    assert_eq!(
        echo_php_strtr(
            EchoValue::string(tr_value),
            EchoValue::string(tr_from),
            EchoValue::string(tr_to),
        )
        .string_bytes(),
        Some(b"xyz-789".to_vec())
    );

    unsafe {
        drop(Box::from_raw(chop));
        drop(Box::from_raw(quoted));
        drop(Box::from_raw(quoted_decode));
        drop(Box::from_raw(nl2br));
        drop(Box::from_raw(search));
        drop(Box::from_raw(replace));
        drop(Box::from_raw(subject));
        drop(Box::from_raw(isearch));
        drop(Box::from_raw(ireplace));
        drop(Box::from_raw(isubject));
        drop(Box::from_raw(tr_value));
        drop(Box::from_raw(tr_from));
        drop(Box::from_raw(tr_to));
    }
}
