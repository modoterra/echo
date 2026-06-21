export type DocsNavGroup = {
  title: string;
  links: DocsNavLink[];
};

export type DocsNavLink = {
  label: string;
  to: string;
  disabled?: boolean;
  children?: DocsNavLink[];
};

export type BuiltinDoc = {
  name: string;
  signature: string;
  description: string;
  tags?: string[];
  aliases?: string[];
};

export type BuiltinFamily = {
  slug: string;
  title: string;
  description: string;
  builtins: BuiltinDoc[];
};

export type DocsTextPart = string | { code: string };

export type DocsBlock =
  | { kind: "paragraph"; text: DocsTextPart[] }
  | { kind: "code"; code: string };

export type DocsSection = {
  title: string;
  blocks: DocsBlock[];
  tags?: string[];
  aliases?: string[];
};

export type DocsPage = {
  id: string;
  path: string;
  navGroup: string;
  category: string;
  title: string;
  summary: string;
  tags: string[];
  aliases?: string[];
  sections: DocsSection[];
};

export const builtinFamilies: BuiltinFamily[] = [
  {
    slug: "strings",
    title: "Strings",
    description: "String functions inspect, transform, encode, split, search, and compare text.",
    builtins: [
      {
        name: "strlen",
        signature: "strlen(string $string): int",
        description:
          "Returns the number of bytes in a string. This is byte length, not character length, so multibyte text can be longer than the number of visible characters.",
      },
      {
        name: "strtoupper",
        signature: "strtoupper(string $string): string",
        description: "Returns the string with alphabetic characters converted to uppercase.",
      },
      {
        name: "strtolower",
        signature: "strtolower(string $string): string",
        description: "Returns the string with alphabetic characters converted to lowercase.",
      },
      {
        name: "ucwords",
        signature: "ucwords(string $string): string",
        description: "Uppercases the first character of each word in the string.",
      },
      {
        name: "ucfirst",
        signature: "ucfirst(string $string): string",
        description: "Uppercases the first character of the string.",
      },
      {
        name: "lcfirst",
        signature: "lcfirst(string $string): string",
        description: "Lowercases the first character of the string.",
      },
      {
        name: "strrev",
        signature: "strrev(string $string): string",
        description: "Returns the string with its bytes in reverse order.",
      },
      {
        name: "str_rot13",
        signature: "str_rot13(string $string): string",
        description: "Applies the ROT13 substitution cipher to the string.",
      },
      {
        name: "ord",
        signature: "ord(string $character): int",
        description: "Returns the byte value of the first character in the string.",
      },
      {
        name: "chr",
        signature: "chr(int $codepoint): string",
        description: "Returns a one-byte string from an integer byte value.",
      },
      {
        name: "bin2hex",
        signature: "bin2hex(string $string): string",
        description: "Converts bytes to a hexadecimal string.",
      },
      {
        name: "hex2bin",
        signature: "hex2bin(string $string): string|false",
        description: "Converts a hexadecimal string back into bytes.",
      },
      {
        name: "base64_encode",
        signature: "base64_encode(string $string): string",
        description: "Encodes bytes using Base64.",
      },
      {
        name: "base64_decode",
        signature: "base64_decode(string $string): string|false",
        description: "Decodes a Base64 string.",
      },
      {
        name: "trim",
        signature: "trim(string $string): string",
        description: "Removes whitespace from both ends of a string.",
      },
      {
        name: "ltrim",
        signature: "ltrim(string $string): string",
        description: "Removes whitespace from the beginning of a string.",
      },
      {
        name: "rtrim",
        signature: "rtrim(string $string): string",
        description: "Removes whitespace from the end of a string.",
      },
      {
        name: "chop",
        signature: "chop(string $string): string",
        description: "Alias of rtrim; removes whitespace from the end of a string.",
      },
      {
        name: "addslashes",
        signature: "addslashes(string $string): string",
        description: "Escapes characters that need backslashes in quoted PHP strings.",
      },
      {
        name: "stripslashes",
        signature: "stripslashes(string $string): string",
        description: "Unquotes a string quoted with backslashes.",
      },
      {
        name: "quoted_printable_encode",
        signature: "quoted_printable_encode(string $string): string",
        description: "Encodes bytes using quoted-printable transfer encoding.",
      },
      {
        name: "quoted_printable_decode",
        signature: "quoted_printable_decode(string $string): string",
        description: "Decodes quoted-printable transfer encoding back to bytes.",
      },
      {
        name: "nl2br",
        signature: "nl2br(string $string, bool $use_xhtml): string",
        description: "Inserts HTML line break tags before newline bytes.",
      },
      {
        name: "quotemeta",
        signature: "quotemeta(string $string): string",
        description: "Quotes regular expression metacharacters with backslashes.",
      },
      {
        name: "str_contains",
        signature: "str_contains(string $haystack, string $needle): bool",
        description: "Returns true when a string contains another string.",
      },
      {
        name: "str_starts_with",
        signature: "str_starts_with(string $haystack, string $needle): bool",
        description: "Returns true when a string begins with another string.",
      },
      {
        name: "str_ends_with",
        signature: "str_ends_with(string $haystack, string $needle): bool",
        description: "Returns true when a string ends with another string.",
      },
      {
        name: "str_replace",
        signature: "str_replace(array|string $search, array|string $replace, string|array $subject): string|array",
        description: "Replaces fixed string occurrences in another string.",
      },
      {
        name: "str_ireplace",
        signature: "str_ireplace(array|string $search, array|string $replace, string|array $subject): string|array",
        description: "Replaces fixed string occurrences case-insensitively.",
      },
      {
        name: "strtr",
        signature: "strtr(string $string, string $from, string $to): string",
        description: "Translates bytes from one character set to another.",
      },
      {
        name: "str_repeat",
        signature: "str_repeat(string $string, int $times): string",
        description: "Repeats a string a fixed number of times.",
      },
      {
        name: "str_pad",
        signature: "str_pad(string $string, int $length, string $pad_string, int $pad_type): string",
        description: "Pads a string to a requested byte length.",
      },
      {
        name: "str_split",
        signature: "str_split(string $string, int $length): array",
        description: "Splits a string into fixed-size byte chunks.",
      },
      {
        name: "chunk_split",
        signature: "chunk_split(string $string, int $length, string $separator): string",
        description: "Splits a string into chunks and appends a separator after each chunk.",
      },
      {
        name: "substr",
        signature: "substr(string $string, int $offset): string",
        description: "Returns part of a string starting at an offset.",
      },
      {
        name: "strpos",
        signature: "strpos(string $haystack, string $needle): int|false",
        description: "Finds the first occurrence of a string.",
      },
      {
        name: "stripos",
        signature: "stripos(string $haystack, string $needle): int|false",
        description: "Finds the first occurrence of a string case-insensitively.",
      },
      {
        name: "strrpos",
        signature: "strrpos(string $haystack, string $needle): int|false",
        description: "Finds the last occurrence of a string.",
      },
      {
        name: "strripos",
        signature: "strripos(string $haystack, string $needle): int|false",
        description: "Finds the last occurrence of a string case-insensitively.",
      },
      {
        name: "strstr",
        signature: "strstr(string $haystack, string $needle): string|false",
        description: "Finds a string and returns the matching tail.",
      },
      {
        name: "strchr",
        signature: "strchr(string $haystack, string $needle): string|false",
        description: "Alias of strstr.",
      },
      {
        name: "stristr",
        signature: "stristr(string $haystack, string $needle): string|false",
        description: "Case-insensitive strstr.",
      },
      {
        name: "strrchr",
        signature: "strrchr(string $haystack, string $needle): string|false",
        description: "Finds the last occurrence of a character and returns the matching tail.",
      },
      {
        name: "strpbrk",
        signature: "strpbrk(string $string, string $characters): string|false",
        description: "Searches a string for any character from a character set.",
      },
      {
        name: "strspn",
        signature: "strspn(string $string, string $characters): int",
        description: "Counts the initial span containing only characters from a character set.",
      },
      {
        name: "strcspn",
        signature: "strcspn(string $string, string $characters): int",
        description: "Counts the initial span containing no characters from a character set.",
      },
      {
        name: "substr_count",
        signature: "substr_count(string $haystack, string $needle): int",
        description: "Counts substring occurrences.",
      },
      {
        name: "substr_compare",
        signature:
          "substr_compare(string $haystack, string $needle, int $offset, int|null $length, bool $case_insensitive): int",
        description: "Compares a substring against another string.",
      },
      {
        name: "strcmp",
        signature: "strcmp(string $string1, string $string2): int",
        description: "Binary-safe string comparison.",
      },
      {
        name: "strcasecmp",
        signature: "strcasecmp(string $string1, string $string2): int",
        description: "Binary-safe case-insensitive string comparison.",
      },
      {
        name: "strncmp",
        signature: "strncmp(string $string1, string $string2, int $length): int",
        description: "Binary-safe string comparison up to a fixed length.",
      },
      {
        name: "strncasecmp",
        signature: "strncasecmp(string $string1, string $string2, int $length): int",
        description: "Binary-safe case-insensitive string comparison up to a fixed length.",
      },
      {
        name: "explode",
        signature: "explode(string $separator, string $string, int $limit): array",
        description: "Splits a string into an array using a separator.",
      },
      {
        name: "implode",
        signature: "implode(string $separator, array $array): string",
        description: "Joins array values into a string using a separator.",
      },
      {
        name: "join",
        signature: "join(string $separator, array $array): string",
        description: "Alias of implode.",
      },
      {
        name: "rawurlencode",
        signature: "rawurlencode(string $string): string",
        description: "Encodes bytes for RFC 3986 URL path components.",
      },
      {
        name: "rawurldecode",
        signature: "rawurldecode(string $string): string",
        description: "Decodes percent escapes without treating plus as space.",
      },
      {
        name: "urlencode",
        signature: "urlencode(string $string): string",
        description: "Encodes form/query text with spaces as plus signs.",
      },
      {
        name: "urldecode",
        signature: "urldecode(string $string): string",
        description: "Decodes form/query text, including plus signs as spaces.",
      },
    ],
  },
  {
    slug: "arrays",
    title: "Arrays",
    description: "Array functions count, reshape, search, merge, and inspect PHP arrays.",
    builtins: [
      {
        name: "array_is_list",
        signature: "array_is_list(array $array): bool",
        description:
          "Returns true when an array has consecutive integer keys starting at zero. Empty arrays are lists. Associative keys or gaps in numeric keys make an array stop being a list.",
      },
      {
        name: "array_values",
        signature: "array_values(array $array): array",
        description: "Returns the input values reindexed from zero.",
      },
      {
        name: "array_keys",
        signature: "array_keys(array $array, mixed $filter_value, bool $strict): array",
        description: "Returns array keys, optionally filtered by value.",
      },
      {
        name: "array_fill",
        signature: "array_fill(int $start_index, int $count, mixed $value): array",
        description: "Creates an array containing repeated values.",
      },
      {
        name: "array_fill_keys",
        signature: "array_fill_keys(array $keys, mixed $value): array",
        description: "Creates an array by using input values as keys.",
      },
      {
        name: "array_combine",
        signature: "array_combine(array $keys, array $values): array",
        description: "Creates an array from parallel key and value arrays.",
      },
      {
        name: "array_pad",
        signature: "array_pad(array $array, int $length, mixed $value): array",
        description: "Pads an array to a requested length.",
      },
      {
        name: "array_slice",
        signature: "array_slice(array $array, int $offset, ?int $length, bool $preserve_keys): array",
        description: "Extracts a window from an array.",
      },
      {
        name: "array_chunk",
        signature: "array_chunk(array $array, int $length, bool $preserve_keys): array",
        description: "Splits an array into fixed-size chunks.",
      },
      {
        name: "array_merge",
        signature: "array_merge(array ...$arrays): array",
        description: "Merges arrays, appending numeric keys and overwriting duplicate string keys.",
      },
      {
        name: "array_replace",
        signature: "array_replace(array $array, array ...$replacements): array",
        description: "Replaces values in the first array with values from later arrays by key.",
      },
      {
        name: "array_reverse",
        signature: "array_reverse(array $array, bool $preserve_keys): array",
        description: "Returns array elements in reverse order.",
      },
      {
        name: "array_flip",
        signature: "array_flip(array $array): array",
        description: "Exchanges string or integer values with their keys.",
      },
      {
        name: "array_count_values",
        signature: "array_count_values(array $array): array",
        description: "Counts repeated string and integer values.",
      },
      {
        name: "array_key_exists",
        signature: "array_key_exists(mixed $key, array $array): bool",
        description: "Returns true when an array contains a key, even if the value is null.",
      },
      {
        name: "key_exists",
        signature: "key_exists(mixed $key, array $array): bool",
        description: "Alias of array_key_exists.",
      },
      {
        name: "array_key_first",
        signature: "array_key_first(array $array): int|string|null",
        description: "Returns the first array key or null for an empty array.",
      },
      {
        name: "array_key_last",
        signature: "array_key_last(array $array): int|string|null",
        description: "Returns the last array key or null for an empty array.",
      },
      {
        name: "in_array",
        signature: "in_array(mixed $needle, array $haystack, bool $strict): bool",
        description: "Checks whether an array contains a value.",
      },
      {
        name: "array_search",
        signature: "array_search(mixed $needle, array $haystack, bool $strict): int|string|false",
        description: "Returns the first key for a matching value or false.",
      },
      {
        name: "array_sum",
        signature: "array_sum(array $array): int|float",
        description: "Adds numeric array values.",
      },
      {
        name: "array_product",
        signature: "array_product(array $array): int|float",
        description: "Multiplies numeric array values.",
      },
      {
        name: "count",
        signature: "count(Countable|array $value): int",
        description: "Counts the number of elements in an array or countable value.",
      },
      {
        name: "sizeof",
        signature: "sizeof(Countable|array $value): int",
        description: "Alias of count.",
      },
    ],
  },
  {
    slug: "types",
    title: "Types",
    description: "Type functions inspect current values and convert them to scalar forms.",
    builtins: [
      {
        name: "gettype",
        signature: "gettype(mixed $value): string",
        description: "Returns the PHP type name for a value.",
      },
      {
        name: "is_array",
        signature: "is_array(mixed $value): bool",
        description: "Returns true when a value is an array.",
      },
      {
        name: "is_countable",
        signature: "is_countable(mixed $value): bool",
        description: "Returns true when a value can be counted.",
      },
      {
        name: "is_iterable",
        signature: "is_iterable(mixed $value): bool",
        description: "Returns true when a value can be iterated.",
      },
      {
        name: "is_numeric",
        signature: "is_numeric(mixed $value): bool",
        description: "Returns true when a value is numeric or a numeric string.",
      },
      {
        name: "is_null",
        signature: "is_null(mixed $value): bool",
        description: "Returns true when a value is null.",
      },
      {
        name: "is_bool",
        signature: "is_bool(mixed $value): bool",
        description: "Returns true when a value is a boolean.",
      },
      {
        name: "is_int",
        signature: "is_int(mixed $value): bool",
        description: "Returns true when a value is an integer.",
      },
      {
        name: "is_integer",
        signature: "is_integer(mixed $value): bool",
        description: "Alias of is_int.",
      },
      {
        name: "is_long",
        signature: "is_long(mixed $value): bool",
        description: "Alias of is_int.",
      },
      {
        name: "is_float",
        signature: "is_float(mixed $value): bool",
        description: "Returns true when a value is a float.",
      },
      {
        name: "is_double",
        signature: "is_double(mixed $value): bool",
        description: "Alias of is_float.",
      },
      {
        name: "is_finite",
        signature: "is_finite(float $num): bool",
        description: "Returns true when a numeric value is finite.",
      },
      {
        name: "is_infinite",
        signature: "is_infinite(float $num): bool",
        description: "Returns true when a numeric value is infinite.",
      },
      {
        name: "is_nan",
        signature: "is_nan(float $num): bool",
        description: "Returns true when a numeric value is not a number.",
      },
      {
        name: "is_object",
        signature: "is_object(mixed $value): bool",
        description: "Returns true when a value is an object.",
      },
      {
        name: "is_resource",
        signature: "is_resource(mixed $value): bool",
        description: "Returns true when a value is a resource.",
      },
      {
        name: "is_string",
        signature: "is_string(mixed $value): bool",
        description: "Returns true when a value is a string.",
      },
      {
        name: "is_scalar",
        signature: "is_scalar(mixed $value): bool",
        description: "Returns true when a value is a scalar.",
      },
      {
        name: "strval",
        signature: "strval(mixed $value): string",
        description: "Converts a value to a string.",
      },
      {
        name: "boolval",
        signature: "boolval(mixed $value): bool",
        description: "Converts a value to a boolean.",
      },
      {
        name: "intval",
        signature: "intval(mixed $value): int",
        description: "Converts a value to an integer.",
      },
      {
        name: "floatval",
        signature: "floatval(mixed $value): float",
        description: "Converts a value to a float.",
      },
      {
        name: "doubleval",
        signature: "doubleval(mixed $value): float",
        description: "Alias of floatval.",
      },
    ],
  },
  {
    slug: "math",
    title: "Math and Bases",
    description: "Numeric functions calculate simple values and convert integers to base strings.",
    builtins: [
      {
        name: "abs",
        signature: "abs(int|float $num): int|float",
        description: "Returns the absolute value of a number.",
      },
      {
        name: "bindec",
        signature: "bindec(string $binary_string): int|float",
        description: "Converts a binary string to a decimal number.",
      },
      {
        name: "decbin",
        signature: "decbin(int $num): string",
        description: "Converts an integer to a binary string.",
      },
      {
        name: "dechex",
        signature: "dechex(int $num): string",
        description: "Converts an integer to a hexadecimal string.",
      },
      {
        name: "decoct",
        signature: "decoct(int $num): string",
        description: "Converts an integer to an octal string.",
      },
      {
        name: "hexdec",
        signature: "hexdec(string $hex_string): int|float",
        description: "Converts a hexadecimal string to a decimal number.",
      },
      {
        name: "octdec",
        signature: "octdec(string $octal_string): int|float",
        description: "Converts an octal string to a decimal number.",
      },
      {
        name: "base_convert",
        signature: "base_convert(string $num, int $from_base, int $to_base): string",
        description: "Converts a number string between bases.",
      },
      {
        name: "deg2rad",
        signature: "deg2rad(float $num): float",
        description: "Converts degrees to radians.",
      },
      {
        name: "rad2deg",
        signature: "rad2deg(float $num): float",
        description: "Converts radians to degrees.",
      },
      {
        name: "sin",
        signature: "sin(float $num): float",
        description: "Returns the sine of a radian value.",
      },
      {
        name: "cos",
        signature: "cos(float $num): float",
        description: "Returns the cosine of a radian value.",
      },
      {
        name: "tan",
        signature: "tan(float $num): float",
        description: "Returns the tangent of a radian value.",
      },
      {
        name: "asin",
        signature: "asin(float $num): float",
        description: "Returns the arc sine in radians.",
      },
      {
        name: "acos",
        signature: "acos(float $num): float",
        description: "Returns the arc cosine in radians.",
      },
      {
        name: "atan",
        signature: "atan(float $num): float",
        description: "Returns the arc tangent in radians.",
      },
      {
        name: "atan2",
        signature: "atan2(float $y, float $x): float",
        description: "Returns the quadrant-aware arc tangent of y over x.",
      },
      {
        name: "sinh",
        signature: "sinh(float $num): float",
        description: "Returns the hyperbolic sine.",
      },
      {
        name: "cosh",
        signature: "cosh(float $num): float",
        description: "Returns the hyperbolic cosine.",
      },
      {
        name: "tanh",
        signature: "tanh(float $num): float",
        description: "Returns the hyperbolic tangent.",
      },
      {
        name: "asinh",
        signature: "asinh(float $num): float",
        description: "Returns the inverse hyperbolic sine.",
      },
      {
        name: "acosh",
        signature: "acosh(float $num): float",
        description: "Returns the inverse hyperbolic cosine.",
      },
      {
        name: "atanh",
        signature: "atanh(float $num): float",
        description: "Returns the inverse hyperbolic tangent.",
      },
      {
        name: "ceil",
        signature: "ceil(int|float $num): float",
        description: "Rounds a number up.",
      },
      {
        name: "floor",
        signature: "floor(int|float $num): float",
        description: "Rounds a number down.",
      },
      {
        name: "sqrt",
        signature: "sqrt(float $num): float",
        description: "Returns the square root.",
      },
      {
        name: "hypot",
        signature: "hypot(float $x, float $y): float",
        description: "Returns the length of the hypotenuse.",
      },
      {
        name: "exp",
        signature: "exp(float $num): float",
        description: "Returns e raised to a power.",
      },
      {
        name: "expm1",
        signature: "expm1(float $num): float",
        description: "Returns exp(num) minus one with precision near zero.",
      },
      {
        name: "log",
        signature: "log(float $num, float $base): float",
        description: "Returns a natural or base-specific logarithm.",
      },
      {
        name: "log10",
        signature: "log10(float $num): float",
        description: "Returns the base-10 logarithm.",
      },
      {
        name: "log1p",
        signature: "log1p(float $num): float",
        description: "Returns log(1 + num) with precision near zero.",
      },
      {
        name: "pow",
        signature: "pow(mixed $num, mixed $exponent): int|float|object",
        description: "Raises a number to a power.",
      },
      {
        name: "fdiv",
        signature: "fdiv(float $num1, float $num2): float",
        description: "Divides two numbers using IEEE 754 floating-point rules.",
      },
      {
        name: "fpow",
        signature: "fpow(float $num, float $exponent): float",
        description: "Raises a number to a power and returns a float.",
      },
      {
        name: "pi",
        signature: "pi(): float",
        description: "Returns an approximation of pi.",
      },
      {
        name: "fmod",
        signature: "fmod(float $num1, float $num2): float",
        description: "Returns a floating-point remainder.",
      },
    ],
  },
  {
    slug: "hashes",
    title: "Hashes and Checksums",
    description: "Hash and checksum functions create compact identifiers for compatibility workflows.",
    builtins: [
      {
        name: "crc32",
        signature: "crc32(string $string): int",
        description:
          "Calculates a CRC32 checksum over the string bytes. Use it for compact compatibility fingerprints and corruption checks, not for security decisions.",
      },
      {
        name: "md5",
        signature: "md5(string $string, bool $binary): string",
        description:
          "Returns an MD5 digest as lowercase hex by default, or raw bytes when the binary flag is true. Keep it for legacy cache keys and protocol interop rather than password storage.",
      },
      {
        name: "sha1",
        signature: "sha1(string $string, bool $binary): string",
        description:
          "Returns a SHA-1 digest as lowercase hex by default, or raw bytes when the binary flag is true. Use it when existing manifests or protocols require SHA-1, not for new security-sensitive checks.",
      },
    ],
  },
  {
    slug: "filesystem",
    title: "Filesystem",
    description: "Filesystem functions inspect local paths and derive path components.",
    builtins: [
      {
        name: "file_exists",
        signature: "file_exists(string $filename): bool",
        description: "Returns true when a local path exists.",
      },
      {
        name: "chdir",
        signature: "chdir(string $directory): bool",
        description: "Changes the current working directory for the process.",
      },
      {
        name: "getcwd",
        signature: "getcwd(): string|false",
        description: "Returns the current working directory.",
      },
      {
        name: "is_dir",
        signature: "is_dir(string $filename): bool",
        description: "Returns true when a local path exists and is a directory.",
      },
      {
        name: "is_file",
        signature: "is_file(string $filename): bool",
        description: "Returns true when a local path exists and is a regular file.",
      },
      {
        name: "is_link",
        signature: "is_link(string $filename): bool",
        description: "Returns true when a local path exists and is a symbolic link.",
      },
      {
        name: "is_readable",
        signature: "is_readable(string $filename): bool",
        description: "Returns true when a local path can be read by the current process.",
      },
      {
        name: "is_writable",
        signature: "is_writable(string $filename): bool",
        description: "Returns true when a local path can be written by the current process.",
      },
      {
        name: "is_writeable",
        signature: "is_writeable(string $filename): bool",
        description: "Alias of is_writable.",
      },
      {
        name: "is_executable",
        signature: "is_executable(string $filename): bool",
        description: "Returns true when a local file can be executed by the current process.",
      },
      {
        name: "filesize",
        signature: "filesize(string $filename): int|false",
        description: "Returns the size of a local file in bytes, or false when metadata cannot be read.",
      },
      {
        name: "fileatime",
        signature: "fileatime(string $filename): int|false",
        description: "Returns the last access time of a local file as a Unix timestamp, or false when metadata cannot be read.",
      },
      {
        name: "filectime",
        signature: "filectime(string $filename): int|false",
        description: "Returns the inode change time of a local file as a Unix timestamp, or false when metadata cannot be read.",
      },
      {
        name: "filemtime",
        signature: "filemtime(string $filename): int|false",
        description: "Returns the last content modification time of a local file as a Unix timestamp, or false when metadata cannot be read.",
      },
      {
        name: "fileinode",
        signature: "fileinode(string $filename): int|false",
        description: "Returns the inode number for a local file, or false when metadata cannot be read.",
      },
      {
        name: "fileowner",
        signature: "fileowner(string $filename): int|false",
        description: "Returns the numeric owner ID for a local file, or false when metadata cannot be read.",
      },
      {
        name: "filegroup",
        signature: "filegroup(string $filename): int|false",
        description: "Returns the numeric group ID for a local file, or false when metadata cannot be read.",
      },
      {
        name: "fileperms",
        signature: "fileperms(string $filename): int|false",
        description: "Returns the numeric mode bits for a local file, or false when metadata cannot be read.",
      },
      {
        name: "filetype",
        signature: "filetype(string $filename): string|false",
        description: "Returns the local file type, such as file, dir, link, socket, fifo, block, char, or unknown.",
      },
      {
        name: "file_get_contents",
        signature: "file_get_contents(string $filename, bool $use_include_path, ?resource $context, int $offset, ?int $length): string|false",
        description: "Reads a local file into a string, optionally starting at an offset and limiting the number of bytes returned.",
      },
      {
        name: "file_put_contents",
        signature: "file_put_contents(string $filename, mixed $data, int $flags, ?resource $context): int|false",
        description: "Writes data to a local file and returns the number of bytes written, or false on failure.",
      },
      {
        name: "readfile",
        signature: "readfile(string $filename, bool $use_include_path, ?resource $context): int|false",
        description: "Writes a local file to the current output stream and returns the number of bytes read.",
      },
      {
        name: "sys_get_temp_dir",
        signature: "sys_get_temp_dir(): string",
        description: "Returns the directory path used for temporary files.",
      },
      {
        name: "tempnam",
        signature: "tempnam(string $directory, string $prefix): string|false",
        description: "Creates a local temporary file with a unique name and returns its path.",
      },
      {
        name: "readlink",
        signature: "readlink(string $path): string|false",
        description: "Returns the stored target of a local symbolic link, or false when it cannot be read.",
      },
      {
        name: "link",
        signature: "link(string $target, string $link): bool",
        description: "Creates a local hard link to an existing target path.",
      },
      {
        name: "symlink",
        signature: "symlink(string $target, string $link): bool",
        description: "Creates a local symbolic link that points at a target path.",
      },
      {
        name: "touch",
        signature: "touch(string $filename, ?int $mtime, ?int $atime): bool",
        description: "Creates a local file if needed and sets its modification and access timestamps.",
      },
      {
        name: "copy",
        signature: "copy(string $from, string $to, ?resource $context): bool",
        description: "Copies a local file to another path, overwriting an existing destination file.",
      },
      {
        name: "rename",
        signature: "rename(string $from, string $to, ?resource $context): bool",
        description: "Renames or moves a local file or directory.",
      },
      {
        name: "unlink",
        signature: "unlink(string $filename, ?resource $context): bool",
        description: "Deletes a local file name or symbolic link.",
      },
      {
        name: "mkdir",
        signature: "mkdir(string $directory, int $permissions, bool $recursive, ?resource $context): bool",
        description: "Creates a local directory, optionally creating missing parent directories too.",
      },
      {
        name: "rmdir",
        signature: "rmdir(string $directory, ?resource $context): bool",
        description: "Removes an empty local directory.",
      },
      {
        name: "basename",
        signature: "basename(string $path, string $suffix): string",
        description: "Returns the trailing name component of a path.",
      },
      {
        name: "dirname",
        signature: "dirname(string $path, int $levels): string",
        description: "Returns the parent directory portion of a path.",
      },
      {
        name: "realpath",
        signature: "realpath(string $path): string|false",
        description: "Resolves an existing local path to its canonical absolute path, or false when the path cannot be resolved.",
      },
    ],
  },
  {
    slug: "reflection",
    title: "Reflection",
    description: "Reflection functions ask questions about functions and callable values.",
    builtins: [
      {
        name: "function_exists",
        signature: "function_exists(string $function): bool",
        description:
          "Returns true when a name resolves to a function. Language constructs are not functions, so names such as echo and include_once return false.",
      },
      {
        name: "is_callable",
        signature: "is_callable(mixed $value): bool",
        description: "Returns true when a value can be called as a function.",
      },
    ],
  },
  {
    slug: "shell",
    title: "Shell",
    description: "Shell functions quote or escape strings for command-line usage.",
    builtins: [
      {
        name: "escapeshellarg",
        signature: "escapeshellarg(string $arg): string",
        description: "Quotes a string so it can be used as one shell argument.",
      },
      {
        name: "escapeshellcmd",
        signature: "escapeshellcmd(string $command): string",
        description: "Escapes shell metacharacters in a command string.",
      },
    ],
  },
  {
    slug: "output-buffering",
    title: "Output Buffering",
    description: "Output buffering functions control PHP-style buffered output.",
    builtins: [
      {
        name: "flush",
        signature: "flush(): void",
        description: "Flushes system output buffers.",
      },
      {
        name: "ob_start",
        signature: "ob_start(): bool",
        description: "Starts output buffering.",
      },
      {
        name: "ob_flush",
        signature: "ob_flush(): bool",
        description: "Flushes the active output buffer.",
      },
      {
        name: "ob_clean",
        signature: "ob_clean(): bool",
        description: "Cleans the active output buffer.",
      },
      {
        name: "ob_end_flush",
        signature: "ob_end_flush(): bool",
        description: "Flushes and closes the active output buffer.",
      },
      {
        name: "ob_end_clean",
        signature: "ob_end_clean(): bool",
        description: "Cleans and closes the active output buffer.",
      },
      {
        name: "ob_get_clean",
        signature: "ob_get_clean(): string|false",
        description: "Gets the active output buffer contents and closes the buffer.",
      },
      {
        name: "ob_get_contents",
        signature: "ob_get_contents(): string|false",
        description: "Gets the active output buffer contents.",
      },
      {
        name: "ob_get_flush",
        signature: "ob_get_flush(): string|false",
        description: "Gets, flushes, and closes the active output buffer.",
      },
      {
        name: "ob_get_length",
        signature: "ob_get_length(): int|false",
        description: "Gets the active output buffer length.",
      },
      {
        name: "ob_get_level",
        signature: "ob_get_level(): int",
        description: "Gets the current output buffering nesting level.",
      },
      {
        name: "ob_implicit_flush",
        signature: "ob_implicit_flush(bool $enable): void",
        description: "Enables or disables implicit flushing after output calls.",
      },
    ],
  },
  {
    slug: "core",
    title: "Core",
    description: "Core functions define constants and inspect process time.",
    builtins: [
      {
        name: "define",
        signature: "define(string $constant_name, mixed $value): bool",
        description: "Defines a named runtime constant.",
      },
      {
        name: "microtime",
        signature: "microtime(bool $as_float): string|float",
        description: "Returns the current Unix timestamp with microseconds.",
      },
      {
        name: "getenv",
        signature: "getenv(?string $name, bool $local_only): string|array|false",
        description:
          "Reads one environment variable, all environment variables, or false when a named variable is not present.",
      },
      {
        name: "putenv",
        signature: "putenv(string $assignment): bool",
        description:
          "Sets or removes an environment variable for the current process.",
      },
      {
        name: "gethostname",
        signature: "gethostname(): string|false",
        description: "Returns the local machine hostname when it is available.",
      },
      {
        name: "getmypid",
        signature: "getmypid(): int|false",
        description: "Returns the current process ID.",
      },
      {
        name: "uniqid",
        signature: "uniqid(string $prefix, bool $more_entropy): string",
        description:
          "Returns a time-based string identifier. It is useful for compatibility labels, but not for secrets or guaranteed uniqueness.",
      },
    ],
  },
];

export const builtinExamples = new Map<string, string>([
  [
    "abs",
    `let $balance = -42
let $displayBalance = abs($balance)

echo "Balance delta: " . $displayBalance . "\\n"`,
  ],
  [
    "addslashes",
    `let $title = "Alice's notes"
let $sqlPreview = "title='" . addslashes($title) . "'"

echo $sqlPreview . "\\n"`,
  ],
  [
    "array_is_list",
    `let $items = ["draft", "review", "ship"]

if (array_is_list($items)) {
    echo "Render as ordered steps\\n"
}`,
  ],
  [
    "base64_decode",
    `let $encodedToken = "c2lnbmVkOnVzZXItNDI="
let $token = base64_decode($encodedToken)

echo $token . "\\n"`,
  ],
  [
    "base64_encode",
    `let $payload = "user:42:active"
let $header = "X-Session: " . base64_encode($payload)

echo $header . "\\n"`,
  ],
  [
    "basename",
    `let $path = "/var/www/releases/current/index.php"
let $file = basename($path, ".php")

echo $file . "\\n"`,
  ],
  [
    "bin2hex",
    `let $bytes = "OK"
let $traceId = bin2hex($bytes)

echo "trace-" . $traceId . "\\n"`,
  ],
  [
    "boolval",
    `let $enabled = "1"

if (boolval($enabled)) {
    echo "Feature enabled\\n"
}`,
  ],
  [
    "chr",
    `let $lineFeed = chr(10)

echo "first line" . $lineFeed
echo "second line" . $lineFeed`,
  ],
  [
    "chop",
    `let $line = "invoice:1001\\n"
let $id = chop($line)

echo "Import id: " . $id . "\\n"`,
  ],
  [
    "count",
    `let $queue = ["email", "receipt", "webhook"]

echo "Jobs queued: " . count($queue) . "\\n"`,
  ],
  [
    "crc32",
    `let $payload = "invoice:1001:paid"
let $checksum = dechex(crc32($payload))

echo "Export checksum: " . $checksum . "\\n"`,
  ],
  [
    "decbin",
    `let $permissions = 5

echo "Permission bits: " . decbin($permissions) . "\\n"`,
  ],
  [
    "dechex",
    `let $statusColor = 65280

echo "#" . dechex($statusColor) . "\\n"`,
  ],
  [
    "decoct",
    `let $mode = 493

echo "chmod " . decoct($mode) . " storage\\n"`,
  ],
  [
    "define",
    `define("APP_ENV", "production")

echo "Environment configured\\n"`,
  ],
  [
    "dirname",
    `let $path = "/var/www/releases/current/index.php"
let $releaseDir = dirname($path, 1)

echo $releaseDir . "\\n"`,
  ],
  [
    "escapeshellarg",
    `let $path = "/tmp/report final.txt"
let $command = "cat " . escapeshellarg($path)

echo $command . "\\n"`,
  ],
  [
    "escapeshellcmd",
    `let $tool = "deploy; rm -rf /"
let $safeTool = escapeshellcmd($tool)

echo $safeTool . "\\n"`,
  ],
  [
    "explode",
    `let $accept = "text/html,application/json"
let $types = explode(",", $accept, 2)

echo "First accepted type: " . $types[0] . "\\n"`,
  ],
  [
    "file_exists",
    `let $configPath = "config/app.php"

if (file_exists($configPath)) {
    echo "Load application config\\n"
}`,
  ],
  [
    "fileatime",
    `let $report = "storage/report.csv"
let $lastRead = fileatime($report)

if (is_int($lastRead)) {
    echo "Report was read at " . $lastRead . "\\n"
}`,
  ],
  [
    "filectime",
    `let $report = "storage/report.csv"
let $changedAt = filectime($report)

if (is_int($changedAt)) {
    echo "Metadata changed at " . $changedAt . "\\n"
}`,
  ],
  [
    "filegroup",
    `let $report = "storage/report.csv"
let $groupId = filegroup($report)

if (is_int($groupId)) {
    echo "Group id: " . $groupId . "\\n"
}`,
  ],
  [
    "fileinode",
    `let $report = "storage/report.csv"
let $inode = fileinode($report)

if (is_int($inode)) {
    echo "Stable inode: " . $inode . "\\n"
}`,
  ],
  [
    "filemtime",
    `let $asset = "public/app.css"
let $version = filemtime($asset)

if (is_int($version)) {
    echo "/app.css?v=" . $version . "\\n"
}`,
  ],
  [
    "fileowner",
    `let $report = "storage/report.csv"
let $ownerId = fileowner($report)

if (is_int($ownerId)) {
    echo "Owner id: " . $ownerId . "\\n"
}`,
  ],
  [
    "fileperms",
    `let $script = "bin/deploy"
let $mode = fileperms($script)

if (is_int($mode)) {
    echo "Mode: " . decoct($mode) . "\\n"
}`,
  ],
  [
    "filesize",
    `let $upload = "storage/import.csv"
let $bytes = filesize($upload)

if (is_int($bytes)) {
    echo "Upload size: " . $bytes . "\\n"
}`,
  ],
  [
    "filetype",
    `let $target = "storage/current"
let $kind = filetype($target)

if (is_string($kind)) {
    echo "Target kind: " . $kind . "\\n"
}`,
  ],
  [
    "fdiv",
    `let $used = 125
let $capacity = 100
let $load = fdiv($used, $capacity)

echo "Capacity load: " . $load . "\\n"`,
  ],
  [
    "fpow",
    `let $monthlyGrowth = 1.05
let $projected = fpow($monthlyGrowth, 2)

echo "Two-month growth: " . $projected . "\\n"`,
  ],
  [
    "file_get_contents",
    `let $report = "storage/reports/latest.csv"
let $header = file_get_contents($report, false, null, 0, 64)

if (is_string($header)) {
    echo "Report header preview: " . $header . "\\n"
}`,
  ],
  [
    "file_put_contents",
    `let $summary = "storage/reports/summary.txt"
let $bytes = file_put_contents($summary, "rows=125\\nstatus=ready\\n")

if (is_int($bytes)) {
    echo "Summary bytes written: " . $bytes . "\\n"
}`,
  ],
  [
    "readfile",
    `let $download = "storage/exports/report.csv"

if (is_readable($download)) {
    readfile($download)
}`,
  ],
  [
    "sys_get_temp_dir",
    `let $scratchDir = sys_get_temp_dir() . "/echo-import"

if (!is_dir($scratchDir)) {
    mkdir($scratchDir, 0755, true)
}

echo "Scratch directory: " . $scratchDir . "\\n"`,
  ],
  [
    "tempnam",
    `let $staged = tempnam(sys_get_temp_dir(), "export-")

if (is_string($staged)) {
    file_put_contents($staged, "id,status\\n1001,ready\\n")
    rename($staged, "storage/exports/latest.csv")
}`,
  ],
  [
    "readlink",
    `let $current = "releases/current"
let $target = readlink($current)

if (is_string($target)) {
    echo "Current release: " . $target . "\\n"
}`,
  ],
  [
    "link",
    `let $artifact = "storage/releases/app.phar"
let $backup = "storage/releases/app.phar.previous"

if (link($artifact, $backup)) {
    echo "Release backup linked\\n"
}`,
  ],
  [
    "symlink",
    `let $release = "2026-06-21"
let $current = "releases/current"

if (symlink($release, $current)) {
    echo "Current release updated\\n"
}`,
  ],
  [
    "touch",
    `let $marker = "storage/cache/.generated"

if (touch($marker)) {
    echo "Cache marker refreshed\\n"
}`,
  ],
  [
    "copy",
    `let $report = "storage/report.csv"
let $backup = "storage/report.csv.bak"

if (copy($report, $backup)) {
    echo "Report backup ready\\n"
}`,
  ],
  [
    "rename",
    `let $staged = "storage/export.tmp"
let $ready = "storage/export.csv"

if (rename($staged, $ready)) {
    echo "Export published\\n"
}`,
  ],
  [
    "unlink",
    `let $scratch = "storage/export.tmp"

if (is_file($scratch)) {
    unlink($scratch)
}`,
  ],
  [
    "mkdir",
    `let $cacheDir = "storage/cache/daily"

if (mkdir($cacheDir, 0755, true)) {
    echo "Cache directory ready\\n"
}`,
  ],
  [
    "rmdir",
    `let $emptyBatch = "storage/imports/empty-batch"

if (is_dir($emptyBatch)) {
    rmdir($emptyBatch)
}`,
  ],
  [
    "flush",
    `echo "Starting import...\\n"
flush()

echo "Import complete\\n"`,
  ],
  [
    "function_exists",
    `let $storedSession = "c2Vzc2lvbjoxMjM="

if (function_exists("base64_decode")) {
    let $session = base64_decode($storedSession)

    echo "Decoded session: " . $session . "\\n"
} else {
    echo "Session is still encoded\\n"
}`,
  ],
  [
    "gettype",
    `let $payload = ["name", "email"]

echo "Decoded payload type: " . gettype($payload) . "\\n"`,
  ],
  [
    "hex2bin",
    `let $hexToken = "4f4b"
let $token = hex2bin($hexToken)

echo $token . "\\n"`,
  ],
  [
    "intval",
    `let $limit = "25"
let $offset = intval($limit) + 10

echo "Next offset: " . $offset . "\\n"`,
  ],
  [
    "is_array",
    `let $filters = ["active", "verified"]

if (is_array($filters)) {
    echo "Apply " . count($filters) . " filters\\n"
}`,
  ],
  [
    "is_bool",
    `let $published = true

if (is_bool($published)) {
    echo "Publication flag is valid\\n"
}`,
  ],
  [
    "is_callable",
    `let $handler = "strlen"

if (is_callable($handler)) {
    echo "Handler can inspect payloads\\n"
}`,
  ],
  [
    "is_countable",
    `let $batch = ["email", "sms"]

if (is_countable($batch)) {
    echo "Batch size: " . count($batch) . "\\n"
}`,
  ],
  [
    "is_dir",
    `let $cacheDir = "storage/cache"

if (is_dir($cacheDir)) {
    echo "Cache directory is ready\\n"
}`,
  ],
  [
    "is_double",
    `let $ratio = 100

echo "Is floating ratio: " . is_double($ratio) . "\\n"`,
  ],
  [
    "is_file",
    `let $entrypoint = "public/index.php"

if (is_file($entrypoint)) {
    echo "Frontend entrypoint found\\n"
}`,
  ],
  [
    "is_executable",
    `let $tool = "bin/deploy"

if (is_executable($tool)) {
    echo "Deployment tool is runnable\\n"
}`,
  ],
  [
    "is_finite",
    `let $cost = 42

if (is_finite($cost)) {
    echo "Cost can be displayed\\n"
}`,
  ],
  [
    "is_float",
    `let $price = 19

echo "Is decimal price: " . is_float($price) . "\\n"`,
  ],
  [
    "is_infinite",
    `let $score = 100

echo "Is unbounded score: " . is_infinite($score) . "\\n"`,
  ],
  [
    "is_int",
    `let $attempts = 3

if (is_int($attempts)) {
    echo "Retry count accepted\\n"
}`,
  ],
  [
    "is_integer",
    `let $page = 2

if (is_integer($page)) {
    echo "Load page " . $page . "\\n"
}`,
  ],
  [
    "is_iterable",
    `let $rows = ["alpha", "beta"]

if (is_iterable($rows)) {
    echo "Rows can be rendered\\n"
}`,
  ],
  [
    "is_link",
    `let $currentRelease = "current"

if (is_link($currentRelease)) {
    echo "Deployment symlink is active\\n"
}`,
  ],
  [
    "is_readable",
    `let $source = "storage/import.csv"

if (is_readable($source)) {
    echo "Import can start\\n"
}`,
  ],
  [
    "is_writable",
    `let $cacheDir = "storage/cache"

if (is_writable($cacheDir)) {
    echo "Cache can be refreshed\\n"
}`,
  ],
  [
    "is_long",
    `let $invoiceId = 1001

if (is_long($invoiceId)) {
    echo "Invoice id is numeric\\n"
}`,
  ],
  [
    "is_nan",
    `let $rating = 5

echo "Is invalid rating: " . is_nan($rating) . "\\n"`,
  ],
  [
    "is_null",
    `let $deletedAt = null

if (is_null($deletedAt)) {
    echo "Record is active\\n"
}`,
  ],
  [
    "is_numeric",
    `let $rawLimit = "25"

if (is_numeric($rawLimit)) {
    echo "Limit: " . intval($rawLimit) . "\\n"
}`,
  ],
  [
    "is_object",
    `let $user = { name: "Ava", role: "admin" }

if (is_object($user)) {
    echo "Render user profile\\n"
}`,
  ],
  [
    "is_resource",
    `let $connection = null

echo "Has open connection: " . is_resource($connection) . "\\n"`,
  ],
  [
    "is_scalar",
    `let $cacheKey = "users:active"

if (is_scalar($cacheKey)) {
    echo "Cache key accepted\\n"
}`,
  ],
  [
    "is_string",
    `let $email = "admin@example.com"

if (is_string($email)) {
    echo strtolower($email) . "\\n"
}`,
  ],
  [
    "lcfirst",
    `let $className = "UserProfile"
let $propertyName = lcfirst($className)

echo $propertyName . "\\n"`,
  ],
  [
    "ltrim",
    `let $path = "/admin/settings"
let $routeKey = ltrim($path)

echo $routeKey . "\\n"`,
  ],
  [
    "microtime",
    `let $started = microtime(true)

echo "Request started at " . $started . "\\n"`,
  ],
  [
    "getenv",
    `let $mode = getenv("APP_ENV")

if (!$mode) {
    $mode = "local"
}

echo "Booting " . $mode . " configuration\\n"`,
  ],
  [
    "putenv",
    `putenv("ECHO_WORKER_MODE=batch")

let $mode = getenv("ECHO_WORKER_MODE")
echo "Worker mode: " . $mode . "\\n"`,
  ],
  [
    "gethostname",
    `let $host = gethostname()

if (!$host) {
    $host = "unknown-host"
}

echo "Processing import on " . $host . "\\n"`,
  ],
  [
    "getmypid",
    `let $pid = getmypid()
let $statusPath = sys_get_temp_dir() . "/echo-worker-" . $pid . ".status"

echo "Status file: " . $statusPath . "\\n"`,
  ],
  [
    "nl2br",
    `let $plain = "first line\\nsecond line"
let $html = nl2br($plain, false)

echo str_replace("\\n", "", $html) . "\\n"`,
  ],
  [
    "md5",
    `let $payload = "user:42:settings"
let $cacheKey = "settings:" . md5($payload)

echo $cacheKey . "\\n"`,
  ],
  [
    "sha1",
    `let $manifest = "asset:app.css:42"
let $digest = sha1($manifest)

echo "Manifest digest: " . substr($digest, 0, 12) . "\\n"`,
  ],
  [
    "ob_clean",
    `ob_start()
echo "draft response"
ob_clean()

echo "final response"`,
  ],
  [
    "ob_end_clean",
    `ob_start()
echo "debug banner"
ob_end_clean()

echo "clean response"`,
  ],
  [
    "ob_end_flush",
    `ob_start()
echo "rendered template"

ob_end_flush()`,
  ],
  [
    "ob_flush",
    `ob_start()
echo "streamed chunk\\n"
ob_flush()

echo "buffer continues\\n"`,
  ],
  [
    "ob_get_clean",
    `ob_start()
echo "welcome email"
let $body = ob_get_clean()

echo "Captured: " . $body . "\\n"`,
  ],
  [
    "ob_get_contents",
    `ob_start()
echo "partial page"
let $preview = ob_get_contents()

echo "Preview bytes: " . strlen($preview) . "\\n"
ob_end_clean()`,
  ],
  [
    "ob_get_flush",
    `ob_start()
echo "template output"
let $sent = ob_get_flush()

echo "Sent bytes: " . strlen($sent) . "\\n"`,
  ],
  [
    "ob_get_length",
    `ob_start()
echo "hello"

echo "Buffered bytes: " . ob_get_length() . "\\n"
ob_end_clean()`,
  ],
  [
    "ob_get_level",
    `ob_start()
ob_start()

echo "Buffer depth: " . ob_get_level() . "\\n"
ob_end_clean()
ob_end_clean()`,
  ],
  [
    "ob_implicit_flush",
    `ob_implicit_flush(true)

echo "Progress: 50%\\n"
echo "Progress: 100%\\n"`,
  ],
  [
    "ob_start",
    `ob_start()
echo "rendered card"
let $html = ob_get_clean()

echo "Captured card: " . $html . "\\n"`,
  ],
  [
    "ord",
    `let $prefix = "A-100"

echo "Prefix byte: " . ord($prefix) . "\\n"`,
  ],
  [
    "quotemeta",
    `let $literal = "user@example.com"
let $pattern = "/" . quotemeta($literal) . "/"

echo $pattern . "\\n"`,
  ],
  [
    "quoted_printable_encode",
    `let $body = "token=a=b\\nnext"
let $encoded = quoted_printable_encode($body)

echo "Mail body: " . $encoded . "\\n"`,
  ],
  [
    "quoted_printable_decode",
    `let $stored = "token=3Da=3Db=0Anext"
let $body = quoted_printable_decode($stored)

echo $body . "\\n"`,
  ],
  [
    "realpath",
    `let $report = realpath("storage/../storage/report.csv")

if (is_string($report)) {
    echo "Serving " . basename($report) . "\\n"
}`,
  ],
  [
    "rtrim",
    `let $line = "status=ok\\n"
let $clean = rtrim($line)

echo $clean . "\\n"`,
  ],
  [
    "sizeof",
    `let $recipients = ["ops@example.com", "dev@example.com"]

echo "Recipients: " . sizeof($recipients) . "\\n"`,
  ],
  [
    "strcasecmp",
    `let $submitted = "Admin@Example.com"
let $known = "admin@example.com"
let $result = strcasecmp($submitted, $known)

echo "Case-insensitive compare: " . $result . "\\n"`,
  ],
  [
    "strcmp",
    `let $expected = "sha256"
let $actual = "sha256"
let $result = strcmp($expected, $actual)

echo "Algorithm compare: " . $result . "\\n"`,
  ],
  [
    "strchr",
    `let $header = "Content-Type: text/html"
let $value = strchr($header, ":")

echo $value . "\\n"`,
  ],
  [
    "str_contains",
    `let $path = "/admin/settings"

if (str_contains($path, "/admin")) {
    echo "Require admin session\\n"
}`,
  ],
  [
    "str_ends_with",
    `let $filename = "report.csv"

if (str_ends_with($filename, ".csv")) {
    echo "Use CSV importer\\n"
}`,
  ],
  [
    "str_ireplace",
    `let $message = "Token TOKEN"
let $safe = str_ireplace("token", "redacted", $message)

echo $safe . "\\n"`,
  ],
  [
    "str_replace",
    `let $template = "Hello {{name}}, status: pending"
let $message = str_replace("{{name}}", "Ada", $template)

echo str_replace("pending", "ready", $message) . "\\n"`,
  ],
  [
    "str_repeat",
    `let $label = "section"
let $divider = str_repeat("-", strlen($label))

echo $label . "\\n" . $divider . "\\n"`,
  ],
  [
    "str_rot13",
    `let $stored = "uryyb"
let $decoded = str_rot13($stored)

echo $decoded . "\\n"`,
  ],
  [
    "str_starts_with",
    `let $command = "deploy:production"

if (str_starts_with($command, "deploy:")) {
    echo "Deployment command\\n"
}`,
  ],
  [
    "strcspn",
    `let $slug = "docs/install#quickstart"
let $pathLength = strcspn($slug, "#?")

echo substr($slug, 0, $pathLength) . "\\n"`,
  ],
  [
    "stripos",
    `let $userAgent = "EchoBot/1.0"
let $offset = stripos($userAgent, "bot")

echo "Bot marker offset: " . $offset . "\\n"`,
  ],
  [
    "stripslashes",
    `let $stored = "Alice\\\\'s notes"
let $title = stripslashes($stored)

echo $title . "\\n"`,
  ],
  [
    "stristr",
    `let $header = "Content-Type: text/html"
let $value = stristr($header, "content-type")

echo $value . "\\n"`,
  ],
  [
    "strlen",
    `let $password = "correct horse"
let $length = strlen($password)

echo "Password bytes: " . $length . "\\n"`,
  ],
  [
    "strncasecmp",
    `let $submitted = "Bearer token"
let $result = strncasecmp($submitted, "bearer", 6)

echo "Authorization scheme compare: " . $result . "\\n"`,
  ],
  [
    "strncmp",
    `let $version = "v1.orders"
let $result = strncmp($version, "v1.", 3)

echo "API version compare: " . $result . "\\n"`,
  ],
  [
    "strpos",
    `let $email = "admin@example.com"
let $domainMarker = strpos($email, "@")

echo "Domain marker offset: " . $domainMarker . "\\n"`,
  ],
  [
    "strpbrk",
    `let $password = "hunter2!"
let $symbol = strpbrk($password, "!@#$%")

echo "First punctuation: " . $symbol . "\\n"`,
  ],
  [
    "strrchr",
    `let $filename = "archive.tar.gz"
let $extension = strrchr($filename, ".")

echo $extension . "\\n"`,
  ],
  [
    "strrev",
    `let $token = "abc123"
let $mirrored = strrev($token)

echo $mirrored . "\\n"`,
  ],
  [
    "strripos",
    `let $path = "/Assets/Images/logo.PNG"
let $extensionAt = strripos($path, ".png")

echo "Extension offset: " . $extensionAt . "\\n"`,
  ],
  [
    "strrpos",
    `let $path = "/var/log/app.log"
let $slash = strrpos($path, "/")

echo substr($path, $slash + 1) . "\\n"`,
  ],
  [
    "strspn",
    `let $duration = "120ms"
let $digits = strspn($duration, "0123456789")

echo substr($duration, 0, $digits) . "\\n"`,
  ],
  [
    "strstr",
    `let $email = "support@example.com"
let $domain = strstr($email, "@")

echo $domain . "\\n"`,
  ],
  [
    "strtolower",
    `let $email = "ADMIN@EXAMPLE.COM"
let $normalized = strtolower($email)

echo $normalized . "\\n"`,
  ],
  [
    "strtr",
    `let $source = "abc-123"
let $label = strtr($source, "abc123", "xyz789")

echo $label . "\\n"`,
  ],
  [
    "strtoupper",
    `let $method = "post"
let $normalized = strtoupper($method)

echo $normalized . "\\n"`,
  ],
  [
    "strval",
    `let $invoiceId = 1001
let $cacheKey = "invoice:" . strval($invoiceId)

echo $cacheKey . "\\n"`,
  ],
  [
    "substr",
    `let $bearer = "Bearer token-value"
let $token = substr($bearer, 7)

echo $token . "\\n"`,
  ],
  [
    "substr_compare",
    `let $path = "/api/users"
let $result = substr_compare($path, "/api", 0, 4, false)

echo "API prefix compare: " . $result . "\\n"`,
  ],
  [
    "substr_count",
    `let $route = "/teams/42/projects/9"
let $depth = substr_count($route, "/")

echo "Route depth: " . $depth . "\\n"`,
  ],
  [
    "trim",
    `let $rawEmail = " admin@example.com "
let $email = trim($rawEmail)

echo strtolower($email) . "\\n"`,
  ],
  [
    "uniqid",
    `let $jobId = uniqid("import_", true)
let $logPath = "storage/jobs/" . $jobId . ".log"

echo "Job log: " . $logPath . "\\n"`,
  ],
  [
    "ucfirst",
    `let $status = "pending"
let $label = ucfirst($status)

echo $label . "\\n"`,
  ],
  [
    "ucwords",
    `let $title = "release notes"
let $heading = ucwords($title)

echo $heading . "\\n"`,
  ],
  [
    "str_pad",
    `let $invoice = "42"
let $display = str_pad($invoice, 6, "0", 0)

echo "Invoice " . $display . "\\n"`,
  ],
  [
    "str_split",
    `let $token = "A1B2C3"
let $pairs = str_split($token, 2)

echo implode("-", $pairs) . "\\n"`,
  ],
  [
    "chunk_split",
    `let $key = "abcdef123456"
let $grouped = chunk_split($key, 4, "-")

echo rtrim($grouped, "-") . "\\n"`,
  ],
  [
    "implode",
    `let $parts = ["api", "v1", "users"]
let $path = "/" . implode("/", $parts)

echo $path . "\\n"`,
  ],
  [
    "join",
    `let $tags = ["php", "echo", "runtime"]
let $label = join(", ", $tags)

echo $label . "\\n"`,
  ],
  [
    "rawurlencode",
    `let $segment = "Quarter 1/report"
let $url = "/files/" . rawurlencode($segment)

echo $url . "\\n"`,
  ],
  [
    "rawurldecode",
    `let $segment = "Quarter%201%2Freport"
let $name = rawurldecode($segment)

echo $name . "\\n"`,
  ],
  [
    "urlencode",
    `let $query = "status: ready"
let $url = "/search?q=" . urlencode($query)

echo $url . "\\n"`,
  ],
  [
    "urldecode",
    `let $raw = "status%3A+ready"
let $query = urldecode($raw)

echo $query . "\\n"`,
  ],
  [
    "array_values",
    `let $row = ["id": "A-42", "qty": "3"]
let $cells = array_values($row)

echo implode(",", $cells) . "\\n"`,
  ],
  [
    "array_keys",
    `let $row = ["id": "A-42", "qty": "3"]
let $columns = array_keys($row)

echo implode(",", $columns) . "\\n"`,
  ],
  [
    "array_fill",
    `let $slots = array_fill(0, 3, "pending")

echo implode(",", $slots) . "\\n"`,
  ],
  [
    "array_fill_keys",
    `let $columns = ["id", "status"]
let $row = array_fill_keys($columns, "missing")

echo $row["status"] . "\\n"`,
  ],
  [
    "array_combine",
    `let $columns = ["id", "status"]
let $values = ["A-42", "ready"]
let $row = array_combine($columns, $values)

echo $row["status"] . "\\n"`,
  ],
  [
    "array_pad",
    `let $codes = ["A", "B"]
let $normalized = array_pad($codes, 4, "N/A")

echo implode(",", $normalized) . "\\n"`,
  ],
  [
    "array_slice",
    `let $queue = ["draft", "review", "ship", "archive"]
let $active = array_slice($queue, 1, 2, false)

echo implode(",", $active) . "\\n"`,
  ],
  [
    "array_chunk",
    `let $ids = ["A1", "A2", "A3", "A4"]
let $batches = array_chunk($ids, 2, false)

echo implode(",", $batches[0]) . "\\n"`,
  ],
  [
    "array_merge",
    `let $base = ["status": "draft"]
let $extra = ["owner": "Ava"]
let $record = array_merge($base, $extra)

echo $record["owner"] . "\\n"`,
  ],
  [
    "array_replace",
    `let $defaults = ["status": "draft", "owner": "unassigned"]
let $override = ["status": "ready"]
let $record = array_replace($defaults, $override)

echo $record["status"] . "\\n"`,
  ],
  [
    "array_reverse",
    `let $events = ["queued", "running", "done"]
let $latestFirst = array_reverse($events, false)

echo $latestFirst[0] . "\\n"`,
  ],
  [
    "array_flip",
    `let $roles = ["admin", "editor"]
let $lookup = array_flip($roles)

echo "editor index: " . $lookup["editor"] . "\\n"`,
  ],
  [
    "array_count_values",
    `let $statuses = ["ready", "ready", "failed"]
let $counts = array_count_values($statuses)

echo "ready: " . $counts["ready"] . "\\n"`,
  ],
  [
    "array_key_exists",
    `let $row = ["id": "A-42", "notes": null]

if (array_key_exists("notes", $row)) {
    echo "notes column present\\n"
}`,
  ],
  [
    "key_exists",
    `let $row = ["id": "A-42"]

if (key_exists("id", $row)) {
    echo "row has id\\n"
}`,
  ],
  [
    "array_key_first",
    `let $row = ["id": "A-42", "status": "ready"]
let $first = array_key_first($row)

echo "first column: " . $first . "\\n"`,
  ],
  [
    "array_key_last",
    `let $row = ["id": "A-42", "status": "ready"]
let $last = array_key_last($row)

echo "last column: " . $last . "\\n"`,
  ],
  [
    "in_array",
    `let $allowed = ["draft", "ready"]
let $status = "ready"

if (in_array($status, $allowed, true)) {
    echo "status accepted\\n"
}`,
  ],
  [
    "array_search",
    `let $columns = ["id", "status", "owner"]
let $index = array_search("status", $columns, true)

echo "status column: " . $index . "\\n"`,
  ],
  [
    "array_sum",
    `let $lineTotals = [12, 8, 5]
let $total = array_sum($lineTotals)

echo "Total: " . $total . "\\n"`,
  ],
  [
    "array_product",
    `let $dimensions = [2, 3, 4]
let $volume = array_product($dimensions)

echo "Volume: " . $volume . "\\n"`,
  ],
  [
    "floatval",
    `let $raw = "12.50"
let $amount = floatval($raw)

echo "Amount: " . $amount . "\\n"`,
  ],
  [
    "doubleval",
    `let $raw = "3.5"
let $ratio = doubleval($raw)

echo "Ratio: " . $ratio . "\\n"`,
  ],
  [
    "bindec",
    `let $flags = "1010"
let $mask = bindec($flags)

echo "Mask: " . $mask . "\\n"`,
  ],
  [
    "hexdec",
    `let $color = "ff"
let $channel = hexdec($color)

echo "Channel: " . $channel . "\\n"`,
  ],
  [
    "octdec",
    `let $mode = "755"
let $perms = octdec($mode)

echo "Mode: " . $perms . "\\n"`,
  ],
  [
    "base_convert",
    `let $id = "ff"
let $decimal = base_convert($id, 16, 10)

echo "Decimal id: " . $decimal . "\\n"`,
  ],
  [
    "deg2rad",
    `let $degrees = 90
let $radians = deg2rad($degrees)

echo "Radians: " . $radians . "\\n"`,
  ],
  [
    "rad2deg",
    `let $degrees = rad2deg(pi())

echo "Degrees: " . $degrees . "\\n"`,
  ],
  [
    "sin",
    `let $wave = sin(deg2rad(90))

echo "Wave peak: " . $wave . "\\n"`,
  ],
  [
    "cos",
    `let $x = cos(deg2rad(0))

echo "Unit x: " . $x . "\\n"`,
  ],
  [
    "tan",
    `let $slope = tan(deg2rad(45))

echo "Slope: " . $slope . "\\n"`,
  ],
  [
    "asin",
    `let $angle = rad2deg(asin(1))

echo "Angle: " . $angle . "\\n"`,
  ],
  [
    "acos",
    `let $angle = rad2deg(acos(0))

echo "Angle: " . $angle . "\\n"`,
  ],
  [
    "atan",
    `let $angle = rad2deg(atan(1))

echo "Angle: " . $angle . "\\n"`,
  ],
  [
    "atan2",
    `let $heading = rad2deg(atan2(1, 1))

echo "Heading: " . $heading . "\\n"`,
  ],
  [
    "sinh",
    `let $growth = sinh(1)

echo "Growth curve: " . $growth . "\\n"`,
  ],
  [
    "cosh",
    `let $growth = cosh(1)

echo "Growth baseline: " . $growth . "\\n"`,
  ],
  [
    "tanh",
    `let $normalized = tanh(1)

echo "Normalized score: " . $normalized . "\\n"`,
  ],
  [
    "asinh",
    `let $value = asinh(2)

echo "Inverse hyperbolic value: " . $value . "\\n"`,
  ],
  [
    "acosh",
    `let $value = acosh(2)

echo "Inverse hyperbolic value: " . $value . "\\n"`,
  ],
  [
    "atanh",
    `let $value = atanh(0.5)

echo "Inverse hyperbolic value: " . $value . "\\n"`,
  ],
  [
    "ceil",
    `let $pages = ceil(41 / 20)

echo "Pages: " . $pages . "\\n"`,
  ],
  [
    "floor",
    `let $completeBatches = floor(41 / 20)

echo "Complete batches: " . $completeBatches . "\\n"`,
  ],
  [
    "sqrt",
    `let $distance = sqrt(3 * 3 + 4 * 4)

echo "Distance: " . $distance . "\\n"`,
  ],
  [
    "hypot",
    `let $distance = hypot(3, 4)

echo "Distance: " . $distance . "\\n"`,
  ],
  [
    "exp",
    `let $growth = exp(0.05)

echo "Growth factor: " . $growth . "\\n"`,
  ],
  [
    "expm1",
    `let $delta = expm1(0.05)

echo "Growth delta: " . $delta . "\\n"`,
  ],
  [
    "log",
    `let $scale = log(100, 10)

echo "Scale: " . $scale . "\\n"`,
  ],
  [
    "log10",
    `let $digits = log10(1000)

echo "Magnitude: " . $digits . "\\n"`,
  ],
  [
    "log1p",
    `let $adjusted = log1p(0.05)

echo "Adjusted growth: " . $adjusted . "\\n"`,
  ],
  [
    "pow",
    `let $capacity = pow(2, 10)

echo "Capacity: " . $capacity . "\\n"`,
  ],
  [
    "pi",
    `let $circumference = 2 * pi() * 10

echo "Circumference: " . $circumference . "\\n"`,
  ],
  [
    "fmod",
    `let $remainder = fmod(17, 5)

echo "Remainder: " . $remainder . "\\n"`,
  ],
  [
    "chdir",
    `let $start = getcwd()

if (chdir(sys_get_temp_dir())) {
    echo "Working in temp\\n"
    chdir($start)
}`,
  ],
  [
    "getcwd",
    `let $cwd = getcwd()

echo "Running from " . basename($cwd) . "\\n"`,
  ],
  [
    "is_writeable",
    `let $target = sys_get_temp_dir()

if (is_writeable($target)) {
    echo "Temp directory accepts output\\n"
}`,
  ],
]);

export const builtinExampleNotes = new Map<string, string>([
  [
    "basename",
    "Use `basename()` when you need the public-facing name from a full path, such as a download filename, import label, or audit-log entry, while keeping server directories out of the output.",
  ],
  [
    "function_exists",
    "Use this pattern when compatibility code can take a better path if a helper is available, while still leaving a clear place for a fallback when the helper is absent.",
  ],
  [
    "is_callable",
    "Use this when a string or value comes from configuration and you need to verify it can be used as a callable before dispatching work to it.",
  ],
  [
    "escapeshellarg",
    "Use this for untrusted path or argument values before composing a shell command; it keeps the value as one argument instead of letting spaces or quotes change the command shape.",
  ],
  [
    "escapeshellcmd",
    "Use this only for command strings that must be displayed or passed through a shell boundary; prefer escaping individual arguments with escapeshellarg when possible.",
  ],
  [
    "ob_start",
    "Use this to capture output from rendering code so it can be stored, transformed, or sent later as a single string.",
  ],
  [
    "ob_get_clean",
    "Use this when the buffer is only an intermediate value and should not leak to stdout before you decide what to do with it.",
  ],
  [
    "ob_get_contents",
    "Use this when you need to inspect the current buffer while keeping it active for later output or cleanup.",
  ],
  [
    "ob_get_flush",
    "Use this when the captured content should both be returned to the program and sent onward to the next output layer.",
  ],
  [
    "chop",
    "Use `chop()` as the PHP alias for `rtrim()` when importing line-oriented data where trailing newlines should be removed before IDs, statuses, or codes are compared.",
  ],
  [
    "quoted_printable_encode",
    "Use `quoted_printable_encode()` when a mail or MIME workflow needs mostly readable text while still escaping bytes such as `=` and line breaks for transfer.",
  ],
  [
    "quoted_printable_decode",
    "Use `quoted_printable_decode()` at the input boundary for stored mail parts or MIME payloads so the rest of the workflow sees the original byte string.",
  ],
  [
    "nl2br",
    "Use `nl2br()` when plain-text notes, comments, or logs need an HTML preview while preserving where the original newline boundaries were.",
  ],
  [
    "str_replace",
    "Use `str_replace()` for fixed-token rewrites such as filling template placeholders, normalizing status labels, or replacing known separators without invoking pattern matching.",
  ],
  [
    "str_ireplace",
    "Use `str_ireplace()` for case-insensitive fixed-token rewrites such as redacting headers or user-provided labels where capitalization is not meaningful.",
  ],
  [
    "strtr",
    "Use the three-argument `strtr()` form for byte-for-byte translation tables, such as compact label encodings or legacy character maps where each source byte maps to one target byte.",
  ],
  [
    "file_exists",
    "Use this before loading optional local files so missing configuration can be handled deliberately instead of failing deeper in the workflow.",
  ],
  [
    "crc32",
    "Use `crc32()` when an existing PHP workflow expects a compact checksum for duplicate detection, export validation, or quick corruption checks. It is intentionally small and fast, so keep it out of security-sensitive decisions.",
  ],
  [
    "md5",
    "Use `md5()` for legacy cache keys, fixture identifiers, or protocol fields that already require MD5. The example scopes it to a cache key, which is a safer fit than passwords or trust decisions.",
  ],
  [
    "sha1",
    "Use `sha1()` when interoperating with an existing manifest, checksum field, or API that names SHA-1. Truncating it for display is fine for labels, but do not treat it as a new security primitive.",
  ],
  [
    "is_readable",
    "Use `is_readable()` before starting an import or report job so the program can fail with a domain-specific message instead of discovering the unreadable path during parsing.",
  ],
  [
    "is_writable",
    "Use `is_writable()` before refreshing caches, writing exports, or creating generated files so setup problems are reported before work is performed.",
  ],
  [
    "is_executable",
    "Use `is_executable()` before dispatching a local tool from a deployment or maintenance script, especially when the path comes from configuration.",
  ],
  [
    "filesize",
    "Use `filesize()` to enforce upload limits, show import sizes, or decide whether a file is large enough to stream instead of loading all at once.",
  ],
  [
    "fileatime",
    "Use `fileatime()` for maintenance tasks that care when a local artifact was last read, such as pruning old reports. Some filesystems disable access-time updates, so treat it as operational metadata.",
  ],
  [
    "filectime",
    "Use `filectime()` when permission, owner, or other inode metadata changes matter to an audit or cache invalidation workflow. It is not a portable creation timestamp.",
  ],
  [
    "filemtime",
    "Use `filemtime()` for stale-cache checks and asset version strings, where changing file contents should produce a new timestamp-backed URL or rebuild decision.",
  ],
  [
    "fileinode",
    "Use `fileinode()` when compatibility code needs to compare filesystem entries at the metadata level, such as detecting whether two paths refer to the same local file on Unix-like systems.",
  ],
  [
    "fileowner",
    "Use `fileowner()` in diagnostics or deployment checks where a numeric owner ID is enough to explain why a generated file cannot be updated.",
  ],
  [
    "filegroup",
    "Use `filegroup()` beside `fileowner()` when deployment or shared-directory scripts need to report the group that controls a file.",
  ],
  [
    "fileperms",
    "Use `fileperms()` when scripts need to display or validate mode bits, such as showing why a deployment helper is not executable.",
  ],
  [
    "filetype",
    "Use `filetype()` before choosing a path-handling branch, such as treating directories, regular files, and symlinks differently in a cleanup or deployment script.",
  ],
  [
    "fdiv",
    "Use `fdiv()` when a metric should keep IEEE floating-point behavior at boundary values, such as reporting `INF` for a saturated ratio instead of throwing away the rest of a reporting workflow.",
  ],
  [
    "fpow",
    "Use `fpow()` for projections or scaling formulas that should always stay in floating-point space, even when the inputs happen to be whole numbers.",
  ],
  [
    "file_get_contents",
    "Use `file_get_contents()` when a workflow needs the whole local file or a bounded slice in memory, such as previewing a report header, loading a small JSON config, or checking the tail of a log.",
  ],
  [
    "file_put_contents",
    "Use `file_put_contents()` for simple generated files where opening a stream would add ceremony, such as writing a cache artifact, summary report, or small export manifest.",
  ],
  [
    "readfile",
    "Use `readfile()` when the useful action is sending file bytes to the current output stream, such as returning a generated export after checking that the local path is readable.",
  ],
  [
    "sys_get_temp_dir",
    "Use `sys_get_temp_dir()` when a workflow needs a scratch location without hard-coding `/tmp`, such as staging uploads, exports, or generated reports before moving them into application storage.",
  ],
  [
    "tempnam",
    "Use `tempnam()` when multiple workers might stage files in the same directory and each needs a distinct path before an atomic `rename()` publishes the finished artifact.",
  ],
  [
    "readlink",
    "Use `readlink()` when a deployment, cache, or storage workflow represents the active version as a symbolic link and needs to report or validate the stored target.",
  ],
  [
    "link",
    "Use `link()` when two directory entries should refer to the same local file contents, such as keeping a previous artifact name without copying the bytes again.",
  ],
  [
    "symlink",
    "Use `symlink()` for lightweight pointers such as `current` release directories, shared asset locations, or generated aliases that should move independently from the target.",
  ],
  [
    "touch",
    "Use `touch()` when a workflow needs a marker file or a controlled modification timestamp, such as recording that generated cache contents are fresh.",
  ],
  [
    "copy",
    "Use `copy()` when the original file should remain in place, such as taking a backup before rewriting a report or staging an export for later publication.",
  ],
  [
    "rename",
    "Use `rename()` to publish staged files atomically within the same filesystem, moving a completed temporary export into the path that readers consume.",
  ],
  [
    "unlink",
    "Use `unlink()` for cleanup of files that are no longer needed, usually after checking the path points to the expected generated file.",
  ],
  [
    "mkdir",
    "Use `mkdir()` before writing generated output into a new directory tree. Recursive creation is useful for cache, export, and upload paths derived from dates or tenants.",
  ],
  [
    "getenv",
    "Use `getenv()` for process configuration that should come from deployment settings, such as choosing a local, staging, or production mode without editing source files.",
  ],
  [
    "putenv",
    "Use `putenv()` when a current process needs to pass a derived setting to later environment-aware work. It changes process environment state, so keep the assignment close to the workflow that needs it.",
  ],
  [
    "gethostname",
    "Use `gethostname()` to add host context to logs, diagnostics, or generated status records. Keep a fallback because some hosts may not report a name.",
  ],
  [
    "getmypid",
    "Use `getmypid()` for operational labels such as status-file names or logs tied to a running worker. Do not use process IDs as secret tokens or security entropy.",
  ],
  [
    "rmdir",
    "Use `rmdir()` when cleanup should remove only an empty directory, leaving non-empty directories intact so accidental recursive deletion is avoided.",
  ],
  [
    "uniqid",
    "Use `uniqid()` for compatibility labels such as job IDs, temp names, or log filenames where the value only needs to be convenient and time-based. It is not a secret token or a uniqueness guarantee.",
  ],
  [
    "realpath",
    "Use `realpath()` to collapse relative segments before logging, serving, or comparing paths. The example keeps internal directory traversal out of the final display name by pairing it with `basename()`.",
  ],
]);

export const builtinFamilyBySlug = new Map(builtinFamilies.map((family) => [family.slug, family]));

export function builtinExample(name: string) {
  const example = builtinExamples.get(name);

  if (!example) {
    throw new Error(`Missing documentation example for PHP builtin: ${name}`);
  }

  return example;
}

export function builtinExampleNote(builtin: BuiltinDoc) {
  const note = builtinExampleNotes.get(builtin.name);

  if (note) {
    return note;
  }

  return `Use \`${builtin.name}()\` when a compatibility workflow needs to ${appliedDescription(
    builtin.description,
  )}. The example shows that behavior in context so the result feeds validation, formatting, or a follow-up decision instead of standing alone as a probe.`;
}

function appliedDescription(description: string) {
  const firstSentence = description.split(".")[0] ?? description;
  return firstSentence.replace(/^(Returns?|Checks?|Converts?|Uppercases?|Lowercases?|Applies?|Escapes?|Pads?|Splits?|Joins?|Calculates?|Changes?|Gets?|Sets?|Starts?|Ends?)\b/, (verb) => {
    const lower = verb.toLowerCase();

    if (lower.endsWith("s")) {
      return lower.slice(0, -1);
    }

    return lower;
  });
}

export function headingId(heading: string) {
  return heading.toLowerCase().replaceAll(" ", "-");
}

export const docsPages: DocsPage[] = [
  {
    id: "installation",
    path: "/docs",
    navGroup: "Getting Started",
    category: "Getting Started",
    title: "Installation",
    summary:
      "Install Echo, run PHP-compatible programs, compile native binaries, and understand the current project status.",
    tags: ["install", "xo", "run", "build", "compiler", "binary"],
    aliases: ["getting started", "install xo", "run a program"],
    sections: [
      {
        title: "Meet Echo",
        tags: ["php superset", "compiler", "runtime"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo is a Rust implementation of a PHP superset. Existing PHP should stay familiar, while Echo adds compiler tooling, native concurrency, parallel execution, and a path toward compiled binaries with predictable performance gains.",
            ],
          },
          {
            kind: "paragraph",
            text: [
              "The command line entrypoint is ",
              { code: "xo" },
              ". Echo is early-stage software, so unsupported PHP behavior should fail explicitly rather than silently approximate semantics.",
            ],
          },
        ],
      },
      {
        title: "Installation",
        tags: ["install", "path", "source build"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Install the ",
              { code: "xo" },
              " command and keep it on your path. The public installer flow is still being designed, so current releases are source-built by contributors.",
            ],
          },
          { kind: "code", code: "xo --help\nxo run app.php\nxo build app.php -o app" },
        ],
      },
      {
        title: "Run a Program",
        tags: ["run", "cli", "php"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Use ",
              { code: "xo run" },
              " to execute an Echo-compatible PHP file directly from the command line.",
            ],
          },
          { kind: "code", code: "xo run examples/hello.php" },
        ],
      },
      {
        title: "Compile a Program",
        tags: ["compile", "build", "binary", "llvm"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Use ",
              { code: "xo build" },
              " to compile a supported program into a native binary. The current backend lowers through LLVM IR and links through the project build path while Echo's native toolchain matures.",
            ],
          },
          { kind: "code", code: "xo build examples/hello.php -o /tmp/hello\n/tmp/hello" },
        ],
      },
      {
        title: "Project Status",
        tags: ["status", "supported behavior", "fixtures"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo currently supports a small but growing PHP-compatible slice across parsing, AST generation, LLVM IR codegen, runtime behavior, and CLI execution. The docs should make that boundary visible as the language grows.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "source-modes",
    path: "/docs/source-modes",
    navGroup: "Getting Started",
    category: "Getting Started",
    title: "Source Modes",
    summary: "Understand how file extensions choose Echo superset mode or strict mode.",
    tags: ["source", "mode", "strict", "php", "echo", "xo"],
    aliases: ["strict mode", "superset mode", "file extension"],
    sections: [
      {
        title: "Source Modes",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Files ending in ",
              { code: ".php" },
              " use Echo superset mode by default. Files ending in ",
              { code: ".echo" },
              " or ",
              { code: ".xo" },
              " use strict mode by default.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "standard-library",
    path: "/docs/standard-library",
    navGroup: "Language",
    category: "Language",
    title: "Standard Library",
    summary:
      "Use Echo's packaged standard library modules for networking, HTTP responses, timing, reflection, and assertions.",
    tags: ["standard library", "stdlib", "std", "net", "http", "time", "reflect", "assert"],
    aliases: ["stdlib", "std packages", "standard packages", "echo standard library"],
    sections: [
      {
        title: "Standard Library Imports",
        tags: ["imports", "namespace", "std"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo packages standard library modules under the ",
              { code: "std" },
              " namespace. Import a package when program behavior should come from Echo's runtime-owned APIs rather than from PHP compatibility built-ins.",
            ],
          },
          {
            kind: "code",
            code: "use std time\n\nlet $started = microtime(true)\ntime.sleep(25)\nlet $elapsed = microtime(true) - $started\n\necho \"Elapsed seconds: \" . $elapsed . \"\\n\"",
          },
          {
            kind: "paragraph",
            text: [
              "Use standard library imports for Echo-native capabilities such as scheduling, networking, and introspection. PHP built-ins remain available for compatibility workflows, while ",
              { code: "std" },
              " modules mark code that intentionally targets Echo's runtime surface.",
            ],
          },
        ],
      },
      {
        title: "std.net",
        tags: ["tcp", "network", "listen", "connect"],
        aliases: ["networking", "tcp server", "tcp connection"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "std.net" },
              " exposes TCP listener and connection APIs. Use it when an Echo program owns socket IO instead of shelling out to another process.",
            ],
          },
          {
            kind: "code",
            code: "use std net\n\nlet $server = net.listen(\"127.0.0.1:8080\")\nlet $connection = net.accept($server)\nlet $request = net.read($connection, 4096)\n\nnet.write($connection, \"received \" . strlen($request) . \" bytes\\n\")\nnet.close($connection)",
          },
          {
            kind: "paragraph",
            text: [
              "This pattern keeps the listener, accepted connection, read buffer, response write, and close operation in one workflow. Prefer it for low-level TCP services where the program needs direct control over connection lifetime.",
            ],
          },
        ],
      },
      {
        title: "std.http",
        tags: ["http", "response", "request"],
        aliases: ["http response", "http request"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "std.http" },
              " contains HTTP helpers built on Echo runtime types. The first supported surface formats plain text responses and reads requests from ",
              { code: "std.net" },
              " connections.",
            ],
          },
          {
            kind: "code",
            code: "use std http\nuse std net\n\nlet $connection = net.connect(\"127.0.0.1:8080\")\nlet $response = http.responseText(\"ok\\n\")\n\nnet.write($connection, $response)\nnet.close($connection)",
          },
          {
            kind: "paragraph",
            text: [
              "Use ",
              { code: "http.responseText()" },
              " when a small service needs correctly framed response bytes without hand-building the HTTP status line and headers for every endpoint.",
            ],
          },
        ],
      },
      {
        title: "std.time",
        tags: ["sleep", "time", "scheduling"],
        aliases: ["sleep", "delay", "timer"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "std.time" },
              " provides scheduling helpers such as millisecond sleep. Use it to express runtime delays in Echo code instead of busy waiting.",
            ],
          },
          {
            kind: "code",
            code: "use std time\n\nlet $attempt = 1\necho \"Polling attempt \" . $attempt . \"\\n\"\ntime.sleep(250)\n\n$attempt = $attempt + 1\necho \"Polling attempt \" . $attempt . \"\\n\"\ntime.sleep(250)",
          },
          {
            kind: "paragraph",
            text: [
              "The delay is explicit at the point where retry behavior happens, so the polling loop stays readable and avoids consuming CPU while waiting for external work.",
            ],
          },
        ],
      },
      {
        title: "std.reflect",
        tags: ["reflection", "type", "metadata"],
        aliases: ["introspection", "function metadata"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "std.reflect" },
              " inspects Echo-visible functions and values. It can see Echo standard library and userland metadata in addition to PHP compatibility functions.",
            ],
          },
          {
            kind: "code",
            code: "use std reflect\n\nlet $name = \"std.time.sleep\"\n\nif (reflect.exists($name)) {\n    echo $name . \" returns \" . reflect.returnType($name) . \"\\n\"\n}",
          },
          {
            kind: "paragraph",
            text: [
              "Use reflection for diagnostics, documentation tooling, and compatibility checks that need to explain what the runtime knows about a symbol before calling it.",
            ],
          },
        ],
      },
      {
        title: "std.assert",
        tags: ["assert", "testing", "validation"],
        aliases: ["assertions", "test helpers"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "std.assert" },
              " provides assertion helpers for Echo test-style programs and small runtime checks.",
            ],
          },
          {
            kind: "code",
            code: "use std assert\n\nlet $payload = \"signed:user-42\"\nlet $parts = explode(\":\", $payload)\n\nassert.equals(count($parts), 2)\nassert.ok($parts[0] == \"signed\")",
          },
          {
            kind: "paragraph",
            text: [
              "Assertions are useful at the edge of examples and fixtures where a program should fail clearly if a parsed or transformed value no longer matches the expected shape.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "source-builds",
    path: "/docs/source-builds",
    navGroup: "Tooling",
    category: "Tooling",
    title: "Source Builds",
    summary: "Build the Echo command line from source and run the workspace verification commands.",
    tags: ["source", "build", "cargo", "llvm", "clang", "php", "test"],
    aliases: ["build from source", "cargo build", "workspace tests"],
    sections: [
      {
        title: "Source Builds",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Contributors can build the current command line from source. Full workspace builds require Rust, LLVM 22, clang, and PHP for compatibility fixture generation.",
            ],
          },
          {
            kind: "code",
            code: "git clone https://github.com/modoterra/echo.git\ncd echo\ncargo build -p xo\ncargo test --workspace\ncargo run -p xo -- run examples/hello.php",
          },
        ],
      },
    ],
  },
];

export const docsPageByPath = new Map(docsPages.map((page) => [page.path, page]));

export const docsNavigation: DocsNavGroup[] = [
  {
    title: "Getting Started",
    links: [
      { label: "Installation", to: "/docs" },
      { label: "Quickstart", to: "/docs/quickstart", disabled: true },
      { label: "Source Modes", to: "/docs/source-modes" },
    ],
  },
  {
    title: "Language",
    links: [
      {
        label: "PHP Built-ins",
        to: "/docs/php-built-ins",
        children: builtinFamilies.map((family) => ({
          label: family.title,
          to: "/docs/php-built-ins/" + family.slug,
        })),
      },
      {
        label: "Standard Library",
        to: "/docs/standard-library",
        children: [
          { label: "net", to: "/docs/standard-library#std.net" },
          { label: "http", to: "/docs/standard-library#std.http" },
          { label: "time", to: "/docs/standard-library#std.time" },
          { label: "reflect", to: "/docs/standard-library#std.reflect" },
          { label: "assert", to: "/docs/standard-library#std.assert" },
        ],
      },
      { label: "PHP Compatibility", to: "/docs/php-compatibility", disabled: true },
      { label: "Strict Mode", to: "/docs/strict-mode", disabled: true },
      { label: "Imports", to: "/docs/imports", disabled: true },
    ],
  },
  {
    title: "Tooling",
    links: [
      { label: "Command Line", to: "/docs/command-line", disabled: true },
      { label: "Language Server", to: "/docs/language-server", disabled: true },
      { label: "Testing", to: "/docs/testing", disabled: true },
      { label: "Source Builds", to: "/docs/source-builds" },
    ],
  },
];
