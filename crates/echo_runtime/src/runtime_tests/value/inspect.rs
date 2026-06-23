use super::*;

#[test]
fn repl_inspector_displays_array_values() {
    let key = test_string_value(b"name");
    let array = echo_value_array_set(echo_value_array_new(), key, test_string_value(b"Echo"));
    let array = echo_value_array_append(array, EchoValue::int(4));

    assert_eq!(
        array.inspect_bytes(),
        Some(br#"Array ["name" => "Echo", 0 => 4]"#.to_vec())
    );
}

#[test]
fn repl_inspector_truncates_large_arrays() {
    let mut array = echo_value_array_new();
    for value in 0..10 {
        array = echo_value_array_append(array, EchoValue::int(value));
    }

    assert_eq!(
        array.inspect_bytes(),
        Some(
            b"Array [0 => 0, 1 => 1, 2 => 2, 3 => 3, 4 => 4, 5 => 5, 6 => 6, 7 => 7, ... 2 more]"
                .to_vec()
        )
    );
}

#[test]
fn runtime_capture_stdout_enables_repl_inspection_without_process_env() {
    let array = echo_value_array_append(echo_value_array_new(), EchoValue::int(2));

    let ((), stdout) = capture_stdout(true, || unsafe {
        echo_write_value(array);
    });

    assert_eq!(stdout, b"Array [0 => 2]");
}
