/** Product surface derived from std module export lines.
 *  Only public exports are listed; suite-only helpers are omitted.
 *
 *  Each export is a first-class reference entry: call form + description at
 *  minimum; Core modules users hit first also carry params + examples.
 */

/** Modules that require full per-export docs (params + example). */
export const stdCoreFullPaths = [
  "std/io",
  "std/str",
  "std/bytes",
  "std/list",
  "std/time",
  "std/math",
] as const;

export type StdCoreFullPath = (typeof stdCoreFullPaths)[number];

export type StdExport = {
  name: string;
  /** Short index line */
  role: string;
  /** How you invoke it after import, e.g. `io.print(value)` or `reflect.KIND_INT` */
  call: string;
  /** Reference description */
  description: string;
  /** Parameter meanings (required for Core full modules) */
  params?: string;
  /** Concrete Echo usage with import (required for Core full modules) */
  example?: string;
};

export type StdModule = {
  /** Import path without leading slash, e.g. std/io */
  path: string;
  /** Final path segment used as the bind name */
  name: string;
  title: string;
  summary: string;
  group: string;
  /** Docs page path under /docs/std/... */
  docsPath: string;
  exports: StdExport[];
};

export const stdModules: StdModule[] = [
  {
    path: "std/io",
    name: "io",
    title: "I/O",
    summary: "Print strings to stdout, log, and stderr.",
    group: "Core",
    docsPath: "/docs/std/io",
    exports: [
      {
        name: "print",
        role: "Write a string to standard output with a newline",
        call: "io.print(value)",
        description:
          "Writes value as text to standard output, followed by a newline. Primary way to print from Echo programs.",
        params: "value — string (or value rendered as text) to write.",
        example: '/ std/io\n\nio.print("hello")',
      },
      {
        name: "log",
        role: "Write a string to the log stream with a newline",
        call: "io.log(value)",
        description:
          "Writes value to the log stream with a newline. Use for diagnostic lines distinct from primary program output.",
        params: "value — text to write to the log stream.",
        example: '/ std/io\n\nio.log("ready")',
      },
      {
        name: "eprint",
        role: "Write a string to standard error with a newline",
        call: "io.eprint(value)",
        description:
          "Writes value to standard error with a newline. Prefer this for error messages so they stay off stdout.",
        params: "value — text to write to stderr.",
        example: '/ std/io\n\nio.eprint("failed")',
      },
    ],
  },
  {
    path: "std/str",
    name: "str",
    title: "Strings",
    summary: "String conversion, text helpers, and number parse.",
    group: "Core",
    docsPath: "/docs/std/str",
    exports: [
      {
        name: "from_int",
        role: "Integer to decimal string",
        call: "str.from_int(n)",
        description: "Converts an integer to its decimal string form.",
        params: "n — integer to convert.",
        example: "/ std/str\n\n$ s = str.from_int(42)",
      },
      {
        name: "from_float",
        role: "Float to string",
        call: "str.from_float(n)",
        description: "Converts a float to a string.",
        params: "n — float to convert.",
        example: "/ std/str\n\n$ s = str.from_float(<f64>3.14)",
      },
      {
        name: "from_bytes",
        role: "UTF-8 decode bytes to string (lossy)",
        call: "str.from_bytes(b)",
        description:
          "Decodes a byte sequence as UTF-8 text. Invalid sequences are handled lossily.",
        params: "b — bytes value to decode.",
        example: "/ std/str\n\n$ s = str.from_bytes(b'hi')",
      },
      {
        name: "from_duration",
        role: "Duration to human unit string",
        call: "str.from_duration(d)",
        description: "Formats a duration value as a human-readable unit string.",
        params: "d — duration value.",
        example: "/ std/str\n\n$ s = str.from_duration(1s)",
      },
      {
        name: "from_locator",
        role: "Locator path or URI to string",
        call: "str.from_locator(loc)",
        description: "Converts a locator (path or URI) to its string form.",
        params: "loc — locator value.",
        example: "/ std/str\n\n$ s = str.from_locator(p'/tmp/x')",
      },
      {
        name: "from_debug",
        role: "Shallow debug text for any value",
        call: "str.from_debug(v)",
        description:
          "Returns shallow debug text for any value (REPL bare-expr display and diagnostics).",
        params: "v — any value.",
        example: "/ std/str\n\n$ s = str.from_debug([1, 2])",
      },
      {
        name: "len",
        role: "Length in UTF-8 bytes",
        call: "str.len(s)",
        description:
          "Returns the length of a string in UTF-8 bytes (same index space as get and slice).",
        params: "s — string value.",
        example: '/ std/str\n\n$ n = str.len("hi")',
      },
      {
        name: "is_empty",
        role: "True when the string is empty",
        call: "str.is_empty(s)",
        description: "True when the string has length zero.",
        params: "s — string value.",
        example: '/ std/str\n\n$ ok = str.is_empty("")',
      },
      {
        name: "cat",
        role: "Concatenate two strings",
        call: "str.cat(a, b)",
        description:
          "Concatenates two strings (or compatible byte sequences via the string helpers).",
        params: "a — left string; b — right string.",
        example: '/ std/str\n\n$ s = str.cat("hel", "lo")',
      },
      {
        name: "contains",
        role: "True when hay contains needle",
        call: "str.contains(hay, needle)",
        description: "True when hay contains needle as a substring.",
        params: "hay — string to search; needle — substring to find.",
        example: '/ std/str\n\n$ ok = str.contains("hello", "ell")',
      },
      {
        name: "starts_with",
        role: "True when string starts with prefix",
        call: "str.starts_with(s, prefix)",
        description: "True when s begins with prefix.",
        params: "s — string; prefix — expected prefix.",
        example: '/ std/str\n\n$ ok = str.starts_with("hello", "he")',
      },
      {
        name: "ends_with",
        role: "True when string ends with suffix",
        call: "str.ends_with(s, suffix)",
        description: "True when s ends with suffix.",
        params: "s — string; suffix — expected suffix.",
        example: '/ std/str\n\n$ ok = str.ends_with("hello", "lo")',
      },
      {
        name: "get",
        role: "UTF-8 byte at index (result-shaped)",
        call: "str.get(s, i)",
        description:
          'Result-shaped access: ok is the UTF-8 byte at index i as ui8; error is "out of bounds". Indices match str.len (byte length).',
        params: "s — string; i — zero-based UTF-8 byte index.",
        example: '/ std/str\n\n| str.get("AB", 1) {\n    $ b { }\n    ! e { }\n}',
      },
      {
        name: "slice",
        role: "UTF-8 byte slice [start, end) (result-shaped)",
        call: "str.slice(s, start, end)",
        description:
          'Result-shaped half-open UTF-8 byte range [start, end). Error is "out of bounds". Prefer char-safe bounds for non-ASCII.',
        params: "s — string; start — inclusive byte index; end — exclusive byte index.",
        example: '/ std/str\n\n| str.slice("hello", 1, 4) {\n    $ part { }\n    ! e { }\n}',
      },
      {
        name: "trim",
        role: "Strip leading and trailing ASCII whitespace",
        call: "str.trim(s)",
        description: "Removes leading and trailing ASCII whitespace.",
        params: "s — string to trim.",
        example: '/ std/str\n\n$ s = str.trim("  hi  ")',
      },
      {
        name: "to_lower",
        role: "ASCII lowercase",
        call: "str.to_lower(s)",
        description: "Lowercases ASCII letters in s.",
        params: "s — string.",
        example: '/ std/str\n\n$ s = str.to_lower("Hi")',
      },
      {
        name: "to_upper",
        role: "ASCII uppercase",
        call: "str.to_upper(s)",
        description: "Uppercases ASCII letters in s.",
        params: "s — string.",
        example: '/ std/str\n\n$ s = str.to_upper("Hi")',
      },
      {
        name: "split",
        role: "Split on separator into a list of strings",
        call: "str.split(s, sep)",
        description: "Splits s on separator sep into a list of strings.",
        params: "s — string; sep — separator substring.",
        example: '/ std/str\n\n$ parts = str.split("a,b,c", ",")',
      },
      {
        name: "replace",
        role: "Replace all occurrences of a substring",
        call: "str.replace(s, from, to)",
        description: "Replaces every occurrence of from in s with to.",
        params: "s — string; from — substring to replace; to — replacement.",
        example: '/ std/str\n\n$ s = str.replace("a-b-c", "-", ":")',
      },
      {
        name: "join",
        role: "Join list of strings with a separator",
        call: "str.join(parts, sep)",
        description: "Joins a list of strings with separator sep.",
        params: "parts — list of strings; sep — separator between parts.",
        example: '/ std/str\n\n$ s = str.join(["a", "b"], ",")',
      },
      {
        name: "repeat",
        role: "Repeat a string n times",
        call: "str.repeat(s, n)",
        description: "Returns s repeated n times.",
        params: "s — string; n — non-negative repeat count.",
        example: '/ std/str\n\n$ s = str.repeat("ab", 3)',
      },
      {
        name: "parse_int",
        role: "Parse an integer; error on failure",
        call: "str.parse_int(s)",
        description: "Parses an integer from text. Failure is result-shaped (error).",
        params: "s — decimal integer text.",
        example: '/ std/str\n\n| str.parse_int("42") {\n    $ n { }\n    ! e { }\n}',
      },
      {
        name: "parse_float",
        role: "Parse a float; error on failure",
        call: "str.parse_float(s)",
        description: "Parses a float from text. Failure is result-shaped (error).",
        params: "s — float text.",
        example: '/ std/str\n\n| str.parse_float("3.5") {\n    $ n { }\n    ! e { }\n}',
      },
    ],
  },
  {
    path: "std/bytes",
    name: "bytes",
    title: "Bytes",
    summary: "Byte sequences: length, slice, concat, and conversions.",
    group: "Core",
    docsPath: "/docs/std/bytes",
    exports: [
      {
        name: "len",
        role: "Length in bytes",
        call: "bytes.len(b)",
        description: "Returns the number of bytes in b.",
        params: "b — bytes value.",
        example: "/ std/bytes\n\n$ n = bytes.len(b'hi')",
      },
      {
        name: "is_empty",
        role: "True when empty",
        call: "bytes.is_empty(b)",
        description: "True when b has length zero.",
        params: "b — bytes value.",
        example: "/ std/bytes\n\n$ ok = bytes.is_empty(b'')",
      },
      {
        name: "get",
        role: "Bounds-checked byte access (result-shaped)",
        call: "bytes.get(b, i)",
        description: 'Result-shaped: ok is the byte at index i as ui8; error is "out of bounds".',
        params: "b — bytes; i — zero-based index.",
        example: "/ std/bytes\n\n| bytes.get(b'AB', 1) {\n    $ x { }\n    ! e { }\n}",
      },
      {
        name: "slice",
        role: "Half-open range extract (result-shaped)",
        call: "bytes.slice(b, start, end)",
        description: 'Result-shaped half-open byte range [start, end). Error is "out of bounds".',
        params: "b — bytes; start — inclusive index; end — exclusive index.",
        example: "/ std/bytes\n\n| bytes.slice(b'hello', 1, 4) {\n    $ part { }\n    ! e { }\n}",
      },
      {
        name: "cat",
        role: "Concatenate two byte sequences",
        call: "bytes.cat(a, b)",
        description: "Concatenates two byte sequences.",
        params: "a — left bytes; b — right bytes.",
        example: "/ std/bytes\n\n$ out = bytes.cat(b'hel', b'lo')",
      },
      {
        name: "from_int",
        role: "Pack integer as 8 little-endian bytes",
        call: "bytes.from_int(n)",
        description: "Packs an integer as 8 little-endian bytes (for hashing and framing).",
        params: "n — integer to pack.",
        example: "/ std/bytes\n\n$ b = bytes.from_int(1)",
      },
      {
        name: "from_str",
        role: "UTF-8 encode a string to bytes",
        call: "bytes.from_str(s)",
        description: "Returns the UTF-8 payload of string s as bytes.",
        params: "s — string to encode.",
        example: '/ std/bytes\n\n$ b = bytes.from_str("hi")',
      },
    ],
  },
  {
    path: "std/list",
    name: "list",
    title: "Lists",
    summary: "List helpers: length, access, sum, and sort.",
    group: "Core",
    docsPath: "/docs/std/list",
    exports: [
      {
        name: "len",
        role: "Element count",
        call: "list.len(xs)",
        description: "Returns the number of elements in xs.",
        params: "xs — list value.",
        example: "/ std/list\n\n$ n = list.len([1, 2, 3])",
      },
      {
        name: "is_empty",
        role: "True when the list is empty",
        call: "list.is_empty(xs)",
        description: "True when xs has no elements.",
        params: "xs — list value.",
        example: "/ std/list\n\n$ ok = list.is_empty([])",
      },
      {
        name: "get",
        role: "Bounds-checked element access (result-shaped)",
        call: "list.get(xs, i)",
        description: 'Result-shaped: ok is the element at index i; error is "out of bounds".',
        params: "xs — list; i — zero-based index.",
        example: "/ std/list\n\n| list.get([10, 20], 0) {\n    $ v { }\n    ! e { }\n}",
      },
      {
        name: "contains",
        role: "True when the list contains the value",
        call: "list.contains(xs, x)",
        description: "True when some element of xs deep-equals x (same as language ==).",
        params: "xs — list; x — value to find.",
        example: "/ std/list\n\n$ ok = list.contains([1, 2, 3], 2)",
      },
      {
        name: "sum_ints",
        role: "Sum of integer elements",
        call: "list.sum_ints(xs)",
        description: "Sums integer elements of xs.",
        params: "xs — list of integers.",
        example: "/ std/list\n\n$ n = list.sum_ints([1, 2, 3])",
      },
      {
        name: "sort_ints",
        role: "Sort integers ascending",
        call: "list.sort_ints(xs)",
        description: "Returns a new list of the integers in xs sorted ascending (insertion sort).",
        params: "xs — list of integers.",
        example: "/ std/list\n\n$ out = list.sort_ints([3, 1, 2])",
      },
    ],
  },
  {
    path: "std/time",
    name: "time",
    title: "Time",
    summary: "Wall clock, sleep, monotonic clock, and format/parse.",
    group: "Core",
    docsPath: "/docs/std/time",
    exports: [
      {
        name: "now_ms",
        role: "Unix epoch milliseconds (UTC wall clock)",
        call: "time.now_ms()",
        description: "Milliseconds since the Unix epoch (UTC wall clock), as an integer.",
        params: "No parameters.",
        example: "/ std/time\n\n$ t = time.now_ms()",
      },
      {
        name: "sleep_ms",
        role: "Sleep at least ms milliseconds",
        call: "time.sleep_ms(ms)",
        description:
          "Sleeps at least ms milliseconds. No-op when ms is less than or equal to zero.",
        params: "ms — milliseconds to sleep.",
        example: "/ std/time\n\ntime.sleep_ms(10)",
      },
      {
        name: "mono_ms",
        role: "Monotonic milliseconds since process start",
        call: "time.mono_ms()",
        description:
          "Monotonic milliseconds since process start (not wall clock). Useful for measuring intervals.",
        params: "No parameters.",
        example: "/ std/time\n\n$ a = time.mono_ms()\n$ b = time.mono_ms()",
      },
      {
        name: "format",
        role: "Format wall-ms with a strftime-like pattern",
        call: "time.format(ms, fmt)",
        description:
          'Formats wall-clock milliseconds with a strftime-like pattern (e.g. "%Y-%m-%d").',
        params: "ms — wall milliseconds since epoch; fmt — format pattern string.",
        example: '/ std/time\n\n$ s = time.format(1700000000000, "%Y-%m-%d")',
      },
      {
        name: "parse",
        role: "Parse text with a pattern to wall-ms",
        call: "time.parse(s, fmt)",
        description:
          'Parses text with a pattern to wall milliseconds since epoch. Failure yields ! "parse failed".',
        params: "s — text to parse; fmt — format pattern string.",
        example:
          '/ std/time\n\n| time.parse("2023-11-14", "%Y-%m-%d") {\n    $ ms { }\n    ! e { }\n}',
      },
    ],
  },
  {
    path: "std/math",
    name: "math",
    title: "Math",
    summary: "Integer and floating-point math helpers.",
    group: "Core",
    docsPath: "/docs/std/math",
    exports: [
      {
        name: "abs_i",
        role: "Absolute value of an integer",
        call: "math.abs_i(n)",
        description: "Integer absolute value.",
        params: "n — integer.",
        example: "/ std/math\n\n$ n = math.abs_i(-3)",
      },
      {
        name: "min",
        role: "Minimum of two integers",
        call: "math.min(a, b)",
        description: "Returns the smaller of two integers.",
        params: "a, b — integers.",
        example: "/ std/math\n\n$ n = math.min(2, 5)",
      },
      {
        name: "max",
        role: "Maximum of two integers",
        call: "math.max(a, b)",
        description: "Returns the larger of two integers.",
        params: "a, b — integers.",
        example: "/ std/math\n\n$ n = math.max(2, 5)",
      },
      {
        name: "sqrt",
        role: "Square root",
        call: "math.sqrt(x)",
        description: "Floating-point square root.",
        params: "x — float.",
        example: "/ std/math\n\n$ y = math.sqrt(<f64>4.0)",
      },
      {
        name: "sin",
        role: "Sine",
        call: "math.sin(x)",
        description: "Sine of x (radians).",
        params: "x — float radians.",
        example: "/ std/math\n\n$ y = math.sin(<f64>0.0)",
      },
      {
        name: "cos",
        role: "Cosine",
        call: "math.cos(x)",
        description: "Cosine of x (radians).",
        params: "x — float radians.",
        example: "/ std/math\n\n$ y = math.cos(<f64>0.0)",
      },
      {
        name: "tan",
        role: "Tangent",
        call: "math.tan(x)",
        description: "Tangent of x (radians).",
        params: "x — float radians.",
        example: "/ std/math\n\n$ y = math.tan(<f64>0.0)",
      },
      {
        name: "floor",
        role: "Floor",
        call: "math.floor(x)",
        description: "Greatest integer value not greater than x, as float.",
        params: "x — float.",
        example: "/ std/math\n\n$ y = math.floor(<f64>3.7)",
      },
      {
        name: "ceil",
        role: "Ceiling",
        call: "math.ceil(x)",
        description: "Least integer value not less than x, as float.",
        params: "x — float.",
        example: "/ std/math\n\n$ y = math.ceil(<f64>3.2)",
      },
      {
        name: "abs_f",
        role: "Absolute value of a float",
        call: "math.abs_f(x)",
        description: "Floating-point absolute value.",
        params: "x — float.",
        example: "/ std/math\n\n$ y = math.abs_f(<f64>-2.5)",
      },
      {
        name: "pow",
        role: "Power",
        call: "math.pow(a, b)",
        description: "Raises a to the power b (floating-point).",
        params: "a — base; b — exponent.",
        example: "/ std/math\n\n$ y = math.pow(<f64>2.0, <f64>3.0)",
      },
    ],
  },
  {
    path: "std/bufio",
    name: "bufio",
    title: "Buffered lines",
    summary: "Split text or files into lines.",
    group: "Core",
    docsPath: "/docs/std/bufio",
    exports: [
      {
        name: "lines",
        role: "Split text into lines on newline",
        call: "bufio.lines(s)",
        description: "Splits text s into a list of lines on newline.",
      },
      {
        name: "read_lines",
        role: "Read a file and split into lines",
        call: "bufio.read_lines(path)",
        description: "Reads the file at path and splits it into lines.",
      },
    ],
  },
  {
    path: "std/test",
    name: "test",
    title: "Testing",
    summary: "Test registration and assertions for xo test.",
    group: "Core",
    docsPath: "/docs/std/test",
    exports: [
      {
        name: "it",
        role: "Register a test case",
        call: "test.it(name, body)",
        description: "Registers a named test case. body is a zero-arg function run by xo test.",
      },
      {
        name: "eq",
        role: "Assert deep equality",
        call: "test.eq(left, right)",
        description: "Asserts left deep-equals right; fails the current test otherwise.",
      },
      {
        name: "ne",
        role: "Assert inequality",
        call: "test.ne(left, right)",
        description: "Asserts left is not equal to right.",
      },
      {
        name: "true",
        role: "Assert truthy",
        call: "test.true(cond)",
        description: "Asserts cond is true.",
      },
      {
        name: "false",
        role: "Assert falsey",
        call: "test.false(cond)",
        description: "Asserts cond is false.",
      },
      {
        name: "fail",
        role: "Fail the current test with a message",
        call: "test.fail(msg)",
        description: "Fails the current test with message msg.",
      },
    ],
  },
  {
    path: "std/reflect",
    name: "reflect",
    title: "Reflection",
    summary: "Runtime kind tags for values and collection keys.",
    group: "Core",
    docsPath: "/docs/std/reflect",
    exports: [
      {
        name: "kind",
        role: "Runtime kind tag for a value",
        call: "reflect.kind(v)",
        description: "Returns the runtime kind tag integer for value v.",
      },
      {
        name: "kind_name",
        role: "Human-readable kind name",
        call: "reflect.kind_name(v)",
        description: "Returns a human-readable kind name for value v.",
      },
      {
        name: "key_bytes",
        role: "Stable key bytes for map and set hashing",
        call: "reflect.key_bytes(v)",
        description: "Stable key bytes used for map and set hashing.",
      },
      {
        name: "is_int",
        role: "True when the value kind is int",
        call: "reflect.is_int(v)",
        description: "True when the value kind is int",
      },
      {
        name: "is_string",
        role: "True when the value kind is string",
        call: "reflect.is_string(v)",
        description: "True when the value kind is string",
      },
      {
        name: "is_bytes",
        role: "True when the value kind is bytes",
        call: "reflect.is_bytes(v)",
        description: "True when the value kind is bytes",
      },
      {
        name: "is_list",
        role: "True when the value kind is list",
        call: "reflect.is_list(v)",
        description: "True when the value kind is list",
      },
      {
        name: "is_float",
        role: "True when the value kind is float",
        call: "reflect.is_float(v)",
        description: "True when the value kind is float",
      },
      {
        name: "is_struct",
        role: "True when the value kind is struct",
        call: "reflect.is_struct(v)",
        description: "True when the value kind is struct",
      },
      {
        name: "KIND_INT",
        role: "Kind tag constant for int values",
        call: "reflect.KIND_INT",
        description: "Kind tag constant for int values.",
      },
      {
        name: "KIND_LIST",
        role: "Kind tag constant for list values",
        call: "reflect.KIND_LIST",
        description: "Kind tag constant for list values.",
      },
      {
        name: "KIND_STRING",
        role: "Kind tag constant for string values",
        call: "reflect.KIND_STRING",
        description: "Kind tag constant for string values.",
      },
      {
        name: "KIND_STRUCT",
        role: "Kind tag constant for struct values",
        call: "reflect.KIND_STRUCT",
        description: "Kind tag constant for struct values.",
      },
      {
        name: "KIND_FLOAT",
        role: "Kind tag constant for float values",
        call: "reflect.KIND_FLOAT",
        description: "Kind tag constant for float values.",
      },
      {
        name: "KIND_BYTES",
        role: "Kind tag constant for bytes values",
        call: "reflect.KIND_BYTES",
        description: "Kind tag constant for bytes values.",
      },
    ],
  },
  {
    path: "std/path",
    name: "path",
    title: "Path",
    summary: "Path join, clean, relative paths, and shallow walk.",
    group: "Files and process",
    docsPath: "/docs/std/path",
    exports: [
      {
        name: "join",
        role: "Join path segments with platform rules",
        call: "path.join(base, rel)",
        description: "Join path segments with platform rules",
      },
      {
        name: "is_abs",
        role: "True for absolute paths",
        call: "path.is_abs(p)",
        description: "True for absolute paths",
      },
      {
        name: "file_name",
        role: "Final path component",
        call: "path.file_name(p)",
        description: "Final path component",
      },
      {
        name: "parent",
        role: "Parent directory path",
        call: "path.parent(p)",
        description: "Parent directory path",
      },
      {
        name: "extension",
        role: "File extension without the dot",
        call: "path.extension(p)",
        description: "File extension without the dot",
      },
      {
        name: "clean",
        role: "Clean . and .. path segments",
        call: "path.clean(p)",
        description: "Clean . and .. path segments",
      },
      {
        name: "rel",
        role: "Relative path from base to target",
        call: "path.rel(base, target)",
        description: "Relative path from base to target",
      },
      {
        name: "walk",
        role: "List direct children under a directory",
        call: "path.walk(root)",
        description: "List direct children under a directory",
      },
    ],
  },
  {
    path: "std/fs",
    name: "fs",
    title: "Filesystem",
    summary: "Files, directories, metadata, streaming, and chmod.",
    group: "Files and process",
    docsPath: "/docs/std/fs",
    exports: [
      {
        name: "exists",
        role: "True if the path exists",
        call: "fs.exists(path)",
        description: "True if the path exists",
      },
      {
        name: "is_file",
        role: "True if the path is a regular file",
        call: "fs.is_file(path)",
        description: "True if the path is a regular file",
      },
      {
        name: "is_dir",
        role: "True if the path is a directory",
        call: "fs.is_dir(path)",
        description: "True if the path is a directory",
      },
      {
        name: "join",
        role: "Join path segments with platform rules",
        call: "fs.join(base, rel)",
        description: "Join path segments with platform rules",
      },
      {
        name: "read",
        role: "Read a whole file as bytes",
        call: "fs.read(path)",
        description: "Read a whole file as bytes",
      },
      {
        name: "write",
        role: "Write a whole file",
        call: "fs.write(path, data)",
        description: "Write a whole file",
      },
      {
        name: "remove",
        role: "Remove a file",
        call: "fs.remove(path)",
        description: "Remove a file",
      },
      {
        name: "copy",
        role: "Copy a file",
        call: "fs.copy(from, to)",
        description: "Copy a file",
      },
      {
        name: "rename",
        role: "Rename or move a path",
        call: "fs.rename(from, to)",
        description: "Rename or move a path",
      },
      {
        name: "create_dir",
        role: "Create a directory",
        call: "fs.create_dir(path)",
        description: "Create a directory",
      },
      {
        name: "create_dir_all",
        role: "Create a directory tree",
        call: "fs.create_dir_all(path)",
        description: "Create a directory tree",
      },
      {
        name: "read_dir",
        role: "List directory entry names",
        call: "fs.read_dir(path)",
        description: "List directory entry names",
      },
      {
        name: "remove_dir",
        role: "Remove an empty directory",
        call: "fs.remove_dir(path)",
        description: "Remove an empty directory",
      },
      {
        name: "metadata",
        role: "File metadata product",
        call: "fs.metadata(path)",
        description: "File metadata product",
      },
      {
        name: "open",
        role: "Open a file for reading",
        call: "fs.open(path)",
        description: "Open a file for reading",
      },
      {
        name: "create",
        role: "Create or truncate a file for writing",
        call: "fs.create(path)",
        description: "Create or truncate a file for writing",
      },
      {
        name: "append",
        role: "Open a file for append",
        call: "fs.append(path)",
        description: "Open a file for append",
      },
      {
        name: "meta",
        role: "Metadata product shape",
        call: "fs.meta",
        description: "Product shape describing file metadata fields.",
      },
      {
        name: "file",
        role: "Streaming file handle shape",
        call: "fs.file",
        description: "Product shape for a streaming file handle.",
      },
      {
        name: "temp_dir",
        role: "System temporary directory path",
        call: "fs.temp_dir()",
        description: "System temporary directory path",
      },
      {
        name: "create_temp",
        role: "Create a temporary path",
        call: "fs.create_temp(prefix)",
        description: "Create a temporary path",
      },
      {
        name: "symlink",
        role: "Create a symbolic link",
        call: "fs.symlink(original, link)",
        description: "Create a symbolic link",
      },
      {
        name: "chmod",
        role: "Set Unix file mode bits",
        call: "fs.chmod(path, mode)",
        description: "Set Unix file mode bits",
      },
    ],
  },
  {
    path: "std/process",
    name: "process",
    title: "Process",
    summary: "Args, environment, spawn, capture, and process pipes.",
    group: "Files and process",
    docsPath: "/docs/std/process",
    exports: [
      {
        name: "args",
        role: "Process argv as a list of strings",
        call: "process.args()",
        description: "Process argv as a list of strings",
      },
      {
        name: "env",
        role: "Look up an environment variable (option)",
        call: "process.env(name)",
        description: "Look up an environment variable (option)",
      },
      {
        name: "env_set",
        role: "Set an environment variable",
        call: "process.env_set(name, value)",
        description: "Set an environment variable",
      },
      {
        name: "env_unset",
        role: "Unset an environment variable",
        call: "process.env_unset(name)",
        description: "Unset an environment variable",
      },
      {
        name: "exit",
        role: "Terminate with an exit code",
        call: "process.exit(code)",
        description: "Terminate with an exit code",
      },
      {
        name: "run",
        role: "Spawn and wait for an exit code",
        call: "process.run(program, args)",
        description: "Spawn and wait for an exit code",
      },
      {
        name: "run_capture",
        role: "Spawn and capture stdout and stderr",
        call: "process.run_capture(program, args)",
        description: "Spawn and capture stdout and stderr",
      },
      {
        name: "run_cwd",
        role: "Spawn with a working directory; capture output",
        call: "process.run_cwd(program, args, cwd)",
        description: "Spawn with a working directory; capture output",
      },
      {
        name: "spawn_pipes",
        role: "Spawn with piped stdin, stdout, and stderr handles",
        call: "process.spawn_pipes(program, args)",
        description: "Spawn with piped stdin, stdout, and stderr handles",
      },
      {
        name: "pipe_write",
        role: "Write to a process pipe (stdin)",
        call: "process.pipe_write(pipe, data)",
        description: "Write to a process pipe (stdin)",
      },
      {
        name: "pipe_read",
        role: "Read from a process pipe (stdout or stderr)",
        call: "process.pipe_read(pipe, limit)",
        description: "Read from a process pipe (stdout or stderr)",
      },
      {
        name: "pipe_close",
        role: "Close one end of a process pipe",
        call: "process.pipe_close(pipe)",
        description: "Close one end of a process pipe",
      },
      {
        name: "wait",
        role: "Wait for a child process; returns exit code",
        call: "process.wait(child)",
        description: "Wait for a child process; returns exit code",
      },
    ],
  },
  {
    path: "std/os",
    name: "os",
    title: "OS",
    summary: "Process id, working directory, hostname, and platform.",
    group: "Files and process",
    docsPath: "/docs/std/os",
    exports: [
      {
        name: "pid",
        role: "Current process id",
        call: "os.pid()",
        description: "Current process id",
      },
      {
        name: "cwd",
        role: "Current working directory",
        call: "os.cwd()",
        description: "Current working directory",
      },
      {
        name: "chdir",
        role: "Change working directory",
        call: "os.chdir(path)",
        description: "Change working directory",
      },
      {
        name: "hostname",
        role: "Host name string",
        call: "os.hostname()",
        description: "Host name string",
      },
      {
        name: "platform",
        role: "OS platform string",
        call: "os.platform()",
        description: "OS platform string",
      },
    ],
  },
  {
    path: "std/json",
    name: "json",
    title: "JSON",
    summary: "Parse and stringify JSON product values.",
    group: "Data",
    docsPath: "/docs/std/json",
    exports: [
      {
        name: "parse",
        role: "Parse JSON text to a product value",
        call: "json.parse(s)",
        description: "Parse JSON text to a product value",
      },
      {
        name: "stringify",
        role: "Serialize a product value to JSON text",
        call: "json.stringify(v)",
        description: "Serialize a product value to JSON text",
      },
    ],
  },
  {
    path: "std/encoding/hex",
    name: "hex",
    title: "Hex",
    summary: "Hex encode and decode.",
    group: "Data",
    docsPath: "/docs/std/encoding-hex",
    exports: [
      {
        name: "encode",
        role: "Encode bytes or text to a hex string",
        call: "hex.encode(b)",
        description: "Encode bytes or text to a hex string",
      },
      {
        name: "decode",
        role: "Decode a hex string to bytes",
        call: "hex.decode(s)",
        description: "Decode a hex string to bytes",
      },
    ],
  },
  {
    path: "std/encoding/base64",
    name: "base64",
    title: "Base64",
    summary: "Base64 encode and decode.",
    group: "Data",
    docsPath: "/docs/std/encoding-base64",
    exports: [
      {
        name: "encode",
        role: "Encode bytes or text to a base64 string",
        call: "base64.encode(b)",
        description: "Encode bytes or text to a base64 string",
      },
      {
        name: "decode",
        role: "Decode a base64 string to bytes",
        call: "base64.decode(s)",
        description: "Decode a base64 string to bytes",
      },
    ],
  },
  {
    path: "std/encoding/csv",
    name: "csv",
    title: "CSV",
    summary: "Thin CSV line and multi-line helpers.",
    group: "Data",
    docsPath: "/docs/std/encoding-csv",
    exports: [
      {
        name: "parse_line",
        role: "Split one CSV line into fields",
        call: "csv.parse_line(s)",
        description: "Split one CSV line into fields",
      },
      {
        name: "parse",
        role: "Parse multi-line CSV into rows",
        call: "csv.parse(s)",
        description: "Parse multi-line CSV into rows",
      },
      {
        name: "format_line",
        role: "Join fields into a CSV line",
        call: "csv.format_line(fields)",
        description: "Join fields into a CSV line",
      },
    ],
  },
  {
    path: "std/compress/gzip",
    name: "gzip",
    title: "gzip",
    summary: "gzip compress and decompress.",
    group: "Data",
    docsPath: "/docs/std/compress-gzip",
    exports: [
      {
        name: "compress",
        role: "Compress input to gzip bytes",
        call: "gzip.compress(data)",
        description: "Compress input to gzip bytes",
      },
      {
        name: "decompress",
        role: "Decompress gzip bytes",
        call: "gzip.decompress(data)",
        description: "Decompress gzip bytes",
      },
    ],
  },
  {
    path: "std/compress/zip",
    name: "zip",
    title: "zip",
    summary: "Single-entry zip pack and unpack.",
    group: "Data",
    docsPath: "/docs/std/compress-zip",
    exports: [
      {
        name: "pack",
        role: "Pack a single named entry into a zip",
        call: "zip.pack(name, data)",
        description: "Pack a single named entry into a zip",
      },
      {
        name: "unpack_first",
        role: "Unpack the first zip entry",
        call: "zip.unpack_first(data)",
        description: "Unpack the first zip entry",
      },
    ],
  },
  {
    path: "std/collections/map",
    name: "map",
    title: "Map",
    summary: "Hash map with mixed keys.",
    group: "Collections",
    docsPath: "/docs/std/collections-map",
    exports: [
      {
        name: "map",
        role: "Map shape with put and get methods",
        call: "map.map",
        description: "Map product shape exposing put/get-style methods on instances.",
      },
      {
        name: "make",
        role: "Construct an empty map",
        call: "map.make()",
        description: "Construct an empty map",
      },
      {
        name: "from_indexed",
        role: "Build a map from indexed pairs",
        call: "map.from_indexed(ls)",
        description: "Build a map from indexed pairs",
      },
    ],
  },
  {
    path: "std/collections/set",
    name: "set",
    title: "Set",
    summary: "Hash set with mixed members.",
    group: "Collections",
    docsPath: "/docs/std/collections-set",
    exports: [
      {
        name: "set",
        role: "Set shape with add and has methods",
        call: "set.set",
        description: "Set product shape exposing add/has-style methods on instances.",
      },
      {
        name: "make",
        role: "Construct an empty set",
        call: "set.make()",
        description: "Construct an empty set",
      },
      {
        name: "from_list",
        role: "Build a set from a list",
        call: "set.from_list(ls)",
        description: "Build a set from a list",
      },
    ],
  },
  {
    path: "std/collections/queue",
    name: "queue",
    title: "Queue",
    summary: "FIFO queue over a list.",
    group: "Collections",
    docsPath: "/docs/std/collections-queue",
    exports: [
      {
        name: "queue",
        role: "FIFO queue shape",
        call: "queue.queue",
        description: "FIFO queue product shape.",
      },
      {
        name: "make",
        role: "Construct an empty queue",
        call: "queue.make()",
        description: "Construct an empty queue",
      },
    ],
  },
  {
    path: "std/collections/hash_table",
    name: "hash_table",
    title: "Hash table",
    summary: "Lower-level hash table.",
    group: "Collections",
    docsPath: "/docs/std/collections-hash_table",
    exports: [
      {
        name: "hash_table",
        role: "Hash table shape",
        call: "hash_table.hash_table",
        description: "Lower-level hash table product shape.",
      },
      {
        name: "make",
        role: "Construct an empty hash table",
        call: "hash_table.make()",
        description: "Construct an empty hash table",
      },
    ],
  },
  {
    path: "std/crypto/hash",
    name: "hash",
    title: "Hash",
    summary: "SipHash, SHA-256, and SHA-512.",
    group: "Crypto and random",
    docsPath: "/docs/std/crypto-hash",
    exports: [
      {
        name: "sha256",
        role: "SHA-256 digest (32 bytes)",
        call: "hash.sha256(data)",
        description: "SHA-256 digest (32 bytes)",
      },
      {
        name: "sha512",
        role: "SHA-512 digest (64 bytes)",
        call: "hash.sha512(data)",
        description: "SHA-512 digest (64 bytes)",
      },
      {
        name: "sip",
        role: "SipHash-2-4 of a message with k0 and k1 keys",
        call: "hash.sip(k0, k1, msg)",
        description: "SipHash-2-4 of a message with k0 and k1 keys",
      },
    ],
  },
  {
    path: "std/crypto/hmac",
    name: "hmac",
    title: "HMAC",
    summary: "HMAC-SHA256.",
    group: "Crypto and random",
    docsPath: "/docs/std/crypto-hmac",
    exports: [
      {
        name: "sha256",
        role: "HMAC-SHA256 of key and data (32 bytes)",
        call: "hmac.sha256(key, data)",
        description: "HMAC-SHA256 of key and data (32 bytes)",
      },
    ],
  },
  {
    path: "std/crypto/aes_gcm",
    name: "aes_gcm",
    title: "AES-GCM",
    summary: "AES-256-GCM encrypt and decrypt.",
    group: "Crypto and random",
    docsPath: "/docs/std/crypto-aes_gcm",
    exports: [
      {
        name: "encrypt",
        role: "AES-256-GCM encrypt (32-byte key, 12-byte nonce)",
        call: "aes_gcm.encrypt(key, nonce, plaintext)",
        description: "AES-256-GCM encrypt (32-byte key, 12-byte nonce)",
      },
      {
        name: "decrypt",
        role: "AES-256-GCM decrypt",
        call: "aes_gcm.decrypt(key, nonce, ciphertext)",
        description: "AES-256-GCM decrypt",
      },
    ],
  },
  {
    path: "std/crypto/random",
    name: "random",
    title: "CSPRNG",
    summary: "Cryptographic random bytes and u64.",
    group: "Crypto and random",
    docsPath: "/docs/std/crypto-random",
    exports: [
      {
        name: "fill",
        role: "Fill n bytes from the CSPRNG",
        call: "random.fill(n)",
        description: "Fill n bytes from the CSPRNG",
      },
      {
        name: "u64",
        role: "Cryptographic random u64",
        call: "random.u64()",
        description: "Cryptographic random u64",
      },
    ],
  },
  {
    path: "std/random",
    name: "random",
    title: "PRNG",
    summary: "Seeded non-cryptographic PRNG.",
    group: "Crypto and random",
    docsPath: "/docs/std/random",
    exports: [
      {
        name: "seed",
        role: "Seed the non-crypto PRNG",
        call: "random.seed(s)",
        description: "Seed the non-crypto PRNG",
      },
      {
        name: "u64",
        role: "Next PRNG u64",
        call: "random.u64()",
        description: "Next PRNG u64",
      },
      {
        name: "float",
        role: "Next PRNG float in [0, 1)",
        call: "random.float()",
        description: "Next PRNG float in [0, 1)",
      },
    ],
  },
  {
    path: "std/log",
    name: "log",
    title: "Logging",
    summary: "Leveled messages and key=value helpers.",
    group: "Crypto and random",
    docsPath: "/docs/std/log",
    exports: [
      {
        name: "emit",
        role: "Emit a message when msg_level meets min_level",
        call: "log.emit(min_level, msg_level, prefix, msg)",
        description: "Emit a message when msg_level meets min_level",
      },
      {
        name: "debug",
        role: "Emit a debug-level message",
        call: "log.debug(min_level, msg)",
        description: "Emit a debug-level message",
      },
      {
        name: "info",
        role: "Emit an info-level message",
        call: "log.info(min_level, msg)",
        description: "Emit an info-level message",
      },
      {
        name: "warn",
        role: "Emit a warning-level message",
        call: "log.warn(min_level, msg)",
        description: "Emit a warning-level message",
      },
      {
        name: "error",
        role: "Emit an error-level message",
        call: "log.error(min_level, msg)",
        description: "Emit an error-level message",
      },
      {
        name: "kv",
        role: "Join key=value pairs into a string",
        call: "log.kv(pairs)",
        description: "Join key=value pairs into a string",
      },
      {
        name: "info_kv",
        role: "Info line with key=value pairs",
        call: "log.info_kv(min_level, msg, pairs)",
        description: "Info line with key=value pairs",
      },
    ],
  },
  {
    path: "std/net/tcp",
    name: "tcp",
    title: "TCP",
    summary: "TCP listen, connect, read, and write.",
    group: "Network",
    docsPath: "/docs/std/net-tcp",
    exports: [
      {
        name: "conn",
        role: "TCP connection shape",
        call: "tcp.conn",
        description: "Product shape for a TCP connection handle.",
      },
      {
        name: "listener",
        role: "TCP listener shape",
        call: "tcp.listener",
        description: "Product shape for a TCP listener.",
      },
      {
        name: "listen",
        role: "Start listening; failure yields handle 0",
        call: "tcp.listen(addr)",
        description: "Start listening; failure yields handle 0",
      },
      {
        name: "connect",
        role: "Connect; failure yields handle 0 or a result error",
        call: "tcp.connect(addr)",
        description: "Connect; failure yields handle 0 or a result error",
      },
      {
        name: "accept",
        role: "Accept a connection; failure yields handle 0",
        call: "tcp.accept(lis)",
        description: "Accept a connection; failure yields handle 0",
      },
      {
        name: "read",
        role: "Read up to limit bytes",
        call: "tcp.read(c, limit)",
        description: "Read up to limit bytes",
      },
      {
        name: "write",
        role: "Write bytes or string data",
        call: "tcp.write(c, data)",
        description: "Write bytes or string data",
      },
      {
        name: "close",
        role: "Close the resource",
        call: "tcp.close(x)",
        description: "Close the resource",
      },
    ],
  },
  {
    path: "std/net/udp",
    name: "udp",
    title: "UDP",
    summary: "UDP bind, send, and receive.",
    group: "Network",
    docsPath: "/docs/std/net-udp",
    exports: [
      {
        name: "bind",
        role: "Bind a UDP socket",
        call: "udp.bind(addr)",
        description: "Bind a UDP socket",
      },
      {
        name: "send_to",
        role: "Send a datagram to an address",
        call: "udp.send_to(sock, data, addr)",
        description: "Send a datagram to an address",
      },
      {
        name: "recv_from",
        role: "Receive a datagram with sender address",
        call: "udp.recv_from(sock, limit)",
        description: "Receive a datagram with sender address",
      },
      {
        name: "close",
        role: "Close the resource",
        call: "udp.close(sock)",
        description: "Close the resource",
      },
      {
        name: "socket",
        role: "UDP socket shape",
        call: "udp.socket",
        description: "Product shape for a UDP socket.",
      },
    ],
  },
  {
    path: "std/net/unix",
    name: "unix",
    title: "Unix sockets",
    summary: "Unix domain sockets.",
    group: "Network",
    docsPath: "/docs/std/net-unix",
    exports: [
      {
        name: "listener",
        role: "Unix domain listener shape",
        call: "unix.listener",
        description: "Unix domain listener shape.",
      },
      {
        name: "conn",
        role: "Unix stream connection shape",
        call: "unix.conn",
        description: "Unix stream connection shape.",
      },
      {
        name: "listen",
        role: "Start listening; failure yields handle 0",
        call: "unix.listen(path)",
        description: "Start listening; failure yields handle 0",
      },
      {
        name: "accept",
        role: "Accept a connection; failure yields handle 0",
        call: "unix.accept(lis)",
        description: "Accept a connection; failure yields handle 0",
      },
      {
        name: "connect",
        role: "Connect; failure yields handle 0 or a result error",
        call: "unix.connect(path)",
        description: "Connect; failure yields handle 0 or a result error",
      },
      {
        name: "read",
        role: "Read up to limit bytes",
        call: "unix.read(c, limit)",
        description: "Read up to limit bytes",
      },
      {
        name: "write",
        role: "Write bytes or string data",
        call: "unix.write(c, data)",
        description: "Write bytes or string data",
      },
      {
        name: "close",
        role: "Close the resource",
        call: "unix.close(c)",
        description: "Close the resource",
      },
    ],
  },
  {
    path: "std/net/tls",
    name: "tls",
    title: "TLS",
    summary: "TLS client and server over rustls.",
    group: "Network",
    docsPath: "/docs/std/net-tls",
    exports: [
      {
        name: "listener",
        role: "TLS listener shape",
        call: "tls.listener",
        description: "TLS listener product shape.",
      },
      {
        name: "conn",
        role: "TLS connection shape",
        call: "tls.conn",
        description: "TLS connection product shape.",
      },
      {
        name: "listen",
        role: "Start listening; failure yields handle 0",
        call: "tls.listen(addr)",
        description: "Start listening; failure yields handle 0",
      },
      {
        name: "accept",
        role: "Accept a connection; failure yields handle 0",
        call: "tls.accept(lis, cert_pem, key_pem)",
        description: "Accept a connection; failure yields handle 0",
      },
      {
        name: "connect",
        role: "Connect; failure yields handle 0 or a result error",
        call: "tls.connect(host, port, server_name, ca_pem)",
        description: "Connect; failure yields handle 0 or a result error",
      },
      {
        name: "read",
        role: "Read up to limit bytes",
        call: "tls.read(c, limit)",
        description: "Read up to limit bytes",
      },
      {
        name: "write",
        role: "Write bytes or string data",
        call: "tls.write(c, data)",
        description: "Write bytes or string data",
      },
      {
        name: "close",
        role: "Close the resource",
        call: "tls.close(c)",
        description: "Close the resource",
      },
      {
        name: "close_listener",
        role: "Close a TLS listener",
        call: "tls.close_listener(lis)",
        description: "Close a TLS listener",
      },
      {
        name: "load_pem",
        role: "Load a PEM file as a string",
        call: "tls.load_pem(path)",
        description: "Load a PEM file as a string",
      },
    ],
  },
  {
    path: "std/net/dns",
    name: "dns",
    title: "DNS",
    summary: "Host name lookup.",
    group: "Network",
    docsPath: "/docs/std/net-dns",
    exports: [
      {
        name: "lookup",
        role: "Resolve a host name to address strings",
        call: "dns.lookup(host)",
        description: "Resolve a host name to address strings",
      },
    ],
  },
  {
    path: "std/net/url",
    name: "url",
    title: "URL",
    summary: "Parse and format http(s) URLs.",
    group: "Network",
    docsPath: "/docs/std/net-url",
    exports: [
      {
        name: "parse",
        role: "Parse an http(s) URL into product fields",
        call: "url.parse(s)",
        description: "Parse an http(s) URL into product fields",
      },
      {
        name: "format",
        role: "Build a URL string from scheme, host, port, and path",
        call: "url.format(scheme, host, port, path)",
        description: "Build a URL string from scheme, host, port, and path",
      },
    ],
  },
  {
    path: "std/net/http",
    name: "http",
    title: "HTTP server",
    summary: "HTTP serve loop and connection handling.",
    group: "Network",
    docsPath: "/docs/std/net-http",
    exports: [
      {
        name: "request",
        role: "HTTP request shape (re-export)",
        call: "http.request",
        description: "HTTP request product shape (re-export).",
      },
      {
        name: "response",
        role: "HTTP response shape (re-export)",
        call: "http.response",
        description: "HTTP response product shape (re-export).",
      },
      {
        name: "server",
        role: "HTTP server shape (re-export)",
        call: "http.server",
        description: "HTTP server product shape (re-export).",
      },
      {
        name: "text_response",
        role: "Build a text/plain response",
        call: "http.text_response(status, body)",
        description: "Build a text/plain response",
      },
      {
        name: "html_response",
        role: "Build a text/html response",
        call: "http.html_response(status, body)",
        description: "Build a text/html response",
      },
      {
        name: "json_response",
        role: "Build an application/json response",
        call: "http.json_response(status, body)",
        description: "Build an application/json response",
      },
      {
        name: "parse_request",
        role: "Parse raw HTTP request bytes",
        call: "http.parse_request(raw)",
        description: "Parse raw HTTP request bytes",
      },
      {
        name: "format_response",
        role: "Format a response as HTTP bytes",
        call: "http.format_response(res)",
        description: "Format a response as HTTP bytes",
      },
      {
        name: "dispatch",
        role: "Dispatch a request to matching routes",
        call: "http.dispatch(routes, req)",
        description: "Dispatch a request to matching routes",
      },
      {
        name: "handle_connection",
        role: "Handle one HTTP connection",
        call: "http.handle_connection(c, routes)",
        description: "Handle one HTTP connection",
      },
      {
        name: "serve",
        role: "Accept loop serving routes",
        call: "http.serve(addr, routes)",
        description: "Accept loop serving routes",
      },
    ],
  },
  {
    path: "std/net/http_client",
    name: "http_client",
    title: "HTTP client",
    summary: "HTTP and HTTPS client requests.",
    group: "Network",
    docsPath: "/docs/std/net-http_client",
    exports: [
      {
        name: "get",
        role: "HTTP GET over cleartext TCP",
        call: "http_client.get(host, port, path)",
        description: "HTTP GET over cleartext TCP",
      },
      {
        name: "request",
        role: "HTTP request with method, headers, and body",
        call: "http_client.request(method, host, port, path, headers, body)",
        description: "HTTP request with method, headers, and body",
      },
      {
        name: "get_tls",
        role: "HTTPS GET over TLS",
        call: "http_client.get_tls(host, port, path, server_name, ca_pem)",
        description: "HTTPS GET over TLS",
      },
      {
        name: "request_tls",
        role: "HTTPS request over TLS",
        call: "http_client.request_tls(method, host, port, path, headers, body, server_name, ca_pem)",
        description: "HTTPS request over TLS",
      },
    ],
  },
  {
    path: "std/net/request",
    name: "request",
    title: "HTTP request",
    summary: "HTTP request product shape.",
    group: "Network",
    docsPath: "/docs/std/net-request",
    exports: [
      {
        name: "request",
        role: "HTTP request shape and helpers",
        call: "request.request",
        description: "HTTP request product shape and helpers.",
      },
    ],
  },
  {
    path: "std/net/response",
    name: "response",
    title: "HTTP response",
    summary: "HTTP response product shape.",
    group: "Network",
    docsPath: "/docs/std/net-response",
    exports: [
      {
        name: "response",
        role: "HTTP response shape and helpers",
        call: "response.response",
        description: "HTTP response product shape and helpers.",
      },
    ],
  },
  {
    path: "std/net/server",
    name: "server",
    title: "HTTP server type",
    summary: "HTTP server product shape.",
    group: "Network",
    docsPath: "/docs/std/net-server",
    exports: [
      {
        name: "server",
        role: "HTTP server shape",
        call: "server.server",
        description: "HTTP server product shape.",
      },
    ],
  },
  {
    path: "std/cli",
    name: "cli",
    title: "CLI flags",
    summary: "Parse flags and positionals from argv.",
    group: "CLI",
    docsPath: "/docs/std/cli",
    exports: [
      {
        name: "parse",
        role: "Parse argv into flags and positionals",
        call: "cli.parse(argv)",
        description: "Parse argv into flags and positionals",
      },
      {
        name: "has",
        role: "True if a flag is present",
        call: "cli.has(tokens, name)",
        description: "True if a flag is present",
      },
      {
        name: "get",
        role: "Get a flag value",
        call: "cli.get(tokens, name)",
        description: "Get a flag value",
      },
      {
        name: "positionals",
        role: "Positional arguments list",
        call: "cli.positionals(tokens)",
        description: "Positional arguments list",
      },
    ],
  },
];

export const stdModuleByPath: Record<string, StdModule> = Object.fromEntries(
  stdModules.map((m) => [m.path, m]),
);

export function stdImportLine(m: StdModule): string {
  return `/ ${m.path}`;
}

/** Stable heading title for an export section (used for fragment ids). */
export function stdExportHeading(e: StdExport): string {
  return e.name;
}

/** True when this export has Core-full documentation. */
export function stdExportIsFull(e: StdExport): boolean {
  return Boolean(e.params && e.example);
}

/** True when path is a Core module requiring full entries. */
export function stdModuleRequiresFull(path: string): boolean {
  return (stdCoreFullPaths as readonly string[]).includes(path);
}

/**
 * Validates product std reference completeness.
 * Throws with a detailed message if any export is under-documented.
 */
export function assertStdReferenceComplete(modules: readonly StdModule[] = stdModules): void {
  const problems: string[] = [];

  for (const m of modules) {
    for (const e of m.exports) {
      if (!e.name?.trim()) {
        problems.push(`${m.path}: export missing name`);
        continue;
      }
      if (!e.call?.trim()) {
        problems.push(`${m.path}.${e.name}: missing call form`);
      }
      if (!e.description?.trim()) {
        problems.push(`${m.path}.${e.name}: missing description`);
      }
      if (!e.role?.trim()) {
        problems.push(`${m.path}.${e.name}: missing role`);
      }
      if (stdModuleRequiresFull(m.path)) {
        if (!e.params?.trim()) {
          problems.push(`${m.path}.${e.name}: Core full entry missing params`);
        }
        if (!e.example?.trim()) {
          problems.push(`${m.path}.${e.name}: Core full entry missing example`);
        } else {
          if (!e.example.includes(`/ ${m.path}`)) {
            problems.push(`${m.path}.${e.name}: example must import \`/ ${m.path}\``);
          }
          if (!e.example.includes(e.name) && !e.example.includes(e.call.split("(")[0]!)) {
            problems.push(`${m.path}.${e.name}: example should mention the export`);
          }
        }
      }
    }
  }

  if (problems.length > 0) {
    throw new Error(
      `std reference incomplete (${problems.length}):\n` + problems.slice(0, 40).join("\n"),
    );
  }
}

/** Compact listing still used by the API index overview blocks. */
export function stdExportsListing(m: StdModule): string {
  return m.exports.map((e) => `${e.name}  —  ${e.call}  —  ${e.role}`).join("\n");
}

/** Groups in nav order. */
export const stdGroups: { title: string; modules: StdModule[] }[] = [
  { title: "Core", modules: stdModules.filter((m) => m.group === "Core") },
  {
    title: "Files and process",
    modules: stdModules.filter((m) => m.group === "Files and process"),
  },
  { title: "Data", modules: stdModules.filter((m) => m.group === "Data") },
  { title: "Collections", modules: stdModules.filter((m) => m.group === "Collections") },
  {
    title: "Crypto and random",
    modules: stdModules.filter((m) => m.group === "Crypto and random"),
  },
  { title: "Network", modules: stdModules.filter((m) => m.group === "Network") },
  { title: "CLI", modules: stdModules.filter((m) => m.group === "CLI") },
];

/** Total public export count across all product modules. */
export const stdExportCount = stdModules.reduce((n, m) => n + m.exports.length, 0);
