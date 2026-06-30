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
  | { kind: "code"; code: string; language?: "php" | "shellscript" };

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
        name: "soundex",
        signature: "soundex(string $string): string",
        description: "Calculates a four-character phonetic Soundex key.",
      },
      {
        name: "wordwrap",
        signature:
          "wordwrap(string $string, int $width, string $break, bool $cut_long_words): string",
        description: "Wraps a string at word boundaries.",
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
        name: "stripcslashes",
        signature: "stripcslashes(string $string): string",
        description: "Unquotes C-style escaped byte sequences.",
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
        name: "htmlspecialchars",
        signature: "htmlspecialchars(string $string): string",
        description: "Escapes special HTML characters in a string.",
      },
      {
        name: "htmlspecialchars_decode",
        signature: "htmlspecialchars_decode(string $string): string",
        description: "Decodes special HTML character entities in a string.",
      },
      {
        name: "strip_tags",
        signature: "strip_tags(string $string): string",
        description: "Removes HTML and PHP tags from a string.",
      },
      {
        name: "str_replace",
        signature:
          "str_replace(array|string $search, array|string $replace, string|array $subject): string|array",
        description: "Replaces fixed string occurrences in another string.",
      },
      {
        name: "str_ireplace",
        signature:
          "str_ireplace(array|string $search, array|string $replace, string|array $subject): string|array",
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
        signature:
          "str_pad(string $string, int $length, string $pad_string, int $pad_type): string",
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
        name: "substr_replace",
        signature:
          "substr_replace(string $string, string $replace, int $offset, int|null $length): string",
        description: "Replaces part of a string at a byte offset.",
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
        name: "strnatcmp",
        signature: "strnatcmp(string $string1, string $string2): int",
        description: "Natural-order string comparison.",
      },
      {
        name: "strnatcasecmp",
        signature: "strnatcasecmp(string $string1, string $string2): int",
        description: "Case-insensitive natural-order string comparison.",
      },
      {
        name: "levenshtein",
        signature:
          "levenshtein(string $string1, string $string2, int $insertion_cost, int $replacement_cost, int $deletion_cost): int",
        description: "Calculates edit distance between two strings.",
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
        signature:
          "array_slice(array $array, int $offset, ?int $length, bool $preserve_keys): array",
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
        name: "intdiv",
        signature: "intdiv(int $num1, int $num2): int",
        description: "Returns the integer quotient of a division.",
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
        name: "round",
        signature: "round(int|float $num, int $precision): float",
        description: "Rounds a number to a requested decimal precision.",
      },
      {
        name: "number_format",
        signature:
          "number_format(float $num, int $decimals, ?string $decimal_separator, ?string $thousands_separator): string",
        description: "Formats a number with grouped thousands and decimal separators.",
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
    description: "Hash and checksum functions create compact identifiers for compatibility code.",
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
        name: "clearstatcache",
        signature: "clearstatcache(bool $clear_realpath_cache, ?string $filename): void",
        description: "Clears PHP's cached filesystem metadata for later stat calls.",
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
        description:
          "Returns the size of a local file in bytes, or false when metadata cannot be read.",
      },
      {
        name: "fileatime",
        signature: "fileatime(string $filename): int|false",
        description:
          "Returns the last access time of a local file as a Unix timestamp, or false when metadata cannot be read.",
      },
      {
        name: "filectime",
        signature: "filectime(string $filename): int|false",
        description:
          "Returns the inode change time of a local file as a Unix timestamp, or false when metadata cannot be read.",
      },
      {
        name: "filemtime",
        signature: "filemtime(string $filename): int|false",
        description:
          "Returns the last content modification time of a local file as a Unix timestamp, or false when metadata cannot be read.",
      },
      {
        name: "fileinode",
        signature: "fileinode(string $filename): int|false",
        description:
          "Returns the inode number for a local file, or false when metadata cannot be read.",
      },
      {
        name: "fileowner",
        signature: "fileowner(string $filename): int|false",
        description:
          "Returns the numeric owner ID for a local file, or false when metadata cannot be read.",
      },
      {
        name: "filegroup",
        signature: "filegroup(string $filename): int|false",
        description:
          "Returns the numeric group ID for a local file, or false when metadata cannot be read.",
      },
      {
        name: "fileperms",
        signature: "fileperms(string $filename): int|false",
        description:
          "Returns the numeric mode bits for a local file, or false when metadata cannot be read.",
      },
      {
        name: "filetype",
        signature: "filetype(string $filename): string|false",
        description:
          "Returns the local file type, such as file, dir, link, socket, fifo, block, char, or unknown.",
      },
      {
        name: "file_get_contents",
        signature:
          "file_get_contents(string $filename, bool $use_include_path, ?resource $context, int $offset, ?int $length): string|false",
        description:
          "Reads a local file into a string, optionally starting at an offset and limiting the number of bytes returned.",
      },
      {
        name: "file_put_contents",
        signature:
          "file_put_contents(string $filename, mixed $data, int $flags, ?resource $context): int|false",
        description:
          "Writes data to a local file and returns the number of bytes written, or false on failure.",
      },
      {
        name: "readfile",
        signature:
          "readfile(string $filename, bool $use_include_path, ?resource $context): int|false",
        description:
          "Writes a local file to the current output stream and returns the number of bytes read.",
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
        description:
          "Returns the stored target of a local symbolic link, or false when it cannot be read.",
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
        description:
          "Creates a local file if needed and sets its modification and access timestamps.",
      },
      {
        name: "copy",
        signature: "copy(string $from, string $to, ?resource $context): bool",
        description:
          "Copies a local file to another path, overwriting an existing destination file.",
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
        signature:
          "mkdir(string $directory, int $permissions, bool $recursive, ?resource $context): bool",
        description:
          "Creates a local directory, optionally creating missing parent directories too.",
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
        description:
          "Resolves an existing local path to its canonical absolute path, or false when the path cannot be resolved.",
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
        name: "gettimeofday",
        signature: "gettimeofday(bool $as_float): array|float",
        description: "Returns the current wall-clock time as a timeval array or float.",
      },
      {
        name: "hrtime",
        signature: "hrtime(bool $as_number): array|int|float",
        description: "Returns a high-resolution timestamp as parts or nanoseconds.",
      },
      {
        name: "phpversion",
        signature: "phpversion(?string $extension): string|false",
        description: "Returns the PHP compatibility version or false for unknown extensions.",
      },
      {
        name: "php_sapi_name",
        signature: "php_sapi_name(): string|false",
        description: "Returns the PHP Server API name for the current runtime.",
      },
      {
        name: "zend_version",
        signature: "zend_version(): string",
        description: "Returns the Zend Engine compatibility version.",
      },
      {
        name: "extension_loaded",
        signature: "extension_loaded(string $extension): bool",
        description: "Reports whether a named PHP extension is available.",
      },
      {
        name: "get_loaded_extensions",
        signature: "get_loaded_extensions(bool $zend_extensions): array",
        description: "Returns the names of loaded PHP extensions.",
      },
      {
        name: "get_extension_funcs",
        signature: "get_extension_funcs(string $extension): array|false",
        description: "Returns function names for a loaded PHP extension.",
      },
      {
        name: "get_cfg_var",
        signature: "get_cfg_var(string $option): string|array|false",
        description: "Returns a PHP configuration option value, or false when it is unavailable.",
      },
      {
        name: "ini_get",
        signature: "ini_get(string $option): string|false",
        description: "Returns a PHP ini option value, or false when it is unavailable.",
      },
      {
        name: "ini_get_all",
        signature: "ini_get_all(?string $extension, bool $details): array|false",
        description: "Returns PHP ini option metadata, or false for an unavailable extension.",
      },
      {
        name: "ini_parse_quantity",
        signature: "ini_parse_quantity(string $shorthand): int",
        description: "Parses a PHP ini shorthand quantity into bytes.",
      },
      {
        name: "get_include_path",
        signature: "get_include_path(): string|false",
        description: "Returns PHP's current include_path configuration value.",
      },
      {
        name: "connection_aborted",
        signature: "connection_aborted(): int",
        description: "Reports whether the client connection has aborted.",
      },
      {
        name: "connection_status",
        signature: "connection_status(): int",
        description: "Returns PHP's current client connection status bitfield.",
      },
      {
        name: "ignore_user_abort",
        signature: "ignore_user_abort(?bool $enable): int",
        description: "Gets or sets whether execution continues after a client disconnects.",
      },
      {
        name: "headers_list",
        signature: "headers_list(): array",
        description: "Returns the list of queued HTTP response headers.",
      },
      {
        name: "headers_sent",
        signature: "headers_sent(): bool",
        description: "Reports whether HTTP headers have already been sent.",
      },
      {
        name: "header",
        signature: "header(string $header, bool $replace, int $response_code): void",
        description: "Queues an HTTP response header.",
      },
      {
        name: "header_remove",
        signature: "header_remove(?string $name): void",
        description: "Removes a queued HTTP response header, or all headers when no name is given.",
      },
      {
        name: "http_response_code",
        signature: "http_response_code(?int $response_code): int|bool",
        description: "Gets or sets the HTTP response status code.",
      },
      {
        name: "ini_set",
        signature: "ini_set(string $option, string $value): string|false",
        description: "Sets a PHP ini option and returns its previous value, or false on failure.",
      },
      {
        name: "ini_alter",
        signature: "ini_alter(string $option, string $value): string|false",
        description: "Alias of ini_set().",
      },
      {
        name: "ini_restore",
        signature: "ini_restore(string $option): void",
        description: "Restores a PHP ini option to its original value.",
      },
      {
        name: "php_ini_loaded_file",
        signature: "php_ini_loaded_file(): string|false",
        description: "Returns the loaded PHP configuration file path, or false when none is loaded.",
      },
      {
        name: "php_ini_scanned_files",
        signature: "php_ini_scanned_files(): string|false",
        description: "Returns scanned PHP configuration file paths, or false when none are scanned.",
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
        description: "Sets or removes an environment variable for the current process.",
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
    "stripcslashes",
    `let $encoded = "\\\\n\\\\t\\\\x41"
let $decoded = stripcslashes($encoded)

echo bin2hex($decoded) . "\\n"`,
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
    "clearstatcache",
    `clearstatcache(true, "storage/report.csv")

if (file_exists("storage/report.csv")) {
    echo "report present\\n"
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
    "gettimeofday",
    `let $time = gettimeofday()

echo "timestamp seconds: " . $time["sec"] . "\\n"`,
  ],
  [
    "hrtime",
    `let $stamp = hrtime()

echo "hrtime parts: " . count($stamp) . "\\n"`,
  ],
  [
    "phpversion",
    `let $version = phpversion()

echo "PHP compatibility: " . $version . "\\n"`,
  ],
  [
    "php_sapi_name",
    `if (php_sapi_name() === "cli") {
    echo "running command-line bootstrap\\n"
}`,
  ],
  [
    "zend_version",
    `let $engine = zend_version()

echo "Zend compatibility: " . $engine . "\\n"`,
  ],
  [
    "extension_loaded",
    `if (!extension_loaded("json")) {
    echo "JSON extension is not available\\n"
}`,
  ],
  [
    "get_loaded_extensions",
    `let $extensions = get_loaded_extensions()

echo "Loaded PHP extensions: " . count($extensions) . "\\n"`,
  ],
  [
    "get_extension_funcs",
    `let $functions = get_extension_funcs("json")

if ($functions === false) {
    echo "JSON extension functions are not available\\n"
}`,
  ],
  [
    "php_ini_loaded_file",
    `let $ini = php_ini_loaded_file()

if ($ini === false) {
    echo "No php.ini file is loaded\\n"
}`,
  ],
  [
    "get_cfg_var",
    `let $includePath = get_cfg_var("include_path")

if ($includePath === false) {
    echo "No include_path config value\\n"
}`,
  ],
  [
    "ini_get",
    `let $memoryLimit = ini_get("memory_limit")

if ($memoryLimit === false) {
    echo "No memory_limit config value\\n"
}`,
  ],
  [
    "ini_get_all",
    `let $options = ini_get_all()

echo "ini options: " . count($options) . "\\n"`,
  ],
  [
    "ini_parse_quantity",
    `let $bytes = ini_parse_quantity("256M")

echo "memory bytes: " . $bytes . "\\n"`,
  ],
  [
    "get_include_path",
    `let $path = get_include_path()

if ($path === false) {
    echo "No include_path config value\\n"
}`,
  ],
  [
    "connection_aborted",
    `if (connection_aborted() === 0 && connection_status() === 0) {
    echo "connection normal\\n"
}`,
  ],
  [
    "connection_status",
    `if (connection_status() === 0) {
    echo "connection normal\\n"
}`,
  ],
  [
    "ignore_user_abort",
    `let $previous = ignore_user_abort(true)

echo "previous setting: " . $previous . "\\n"`,
  ],
  [
    "headers_list",
    `let $headers = headers_list()

echo "headers: " . count($headers) . "\\n"`,
  ],
  [
    "headers_sent",
    `if (headers_sent() === false) {
    echo "headers can still be queued\\n"
}`,
  ],
  [
    "header",
    `if (headers_sent() === false) {
    header("X-Debug: off")
}

echo "response body\\n"`,
  ],
  [
    "header_remove",
    `header_remove("X-Debug")

echo "debug header cleared\\n"`,
  ],
  [
    "http_response_code",
    `if (http_response_code() === false) {
    http_response_code(404)
}

echo "status: " . http_response_code() . "\\n"`,
  ],
  [
    "ini_set",
    `let $previous = ini_set("memory_limit", "128M")

if ($previous === false) {
    echo "memory_limit could not be changed\\n"
}`,
  ],
  [
    "ini_alter",
    `let $previous = ini_alter("memory_limit", "128M")

if ($previous === false) {
    echo "memory_limit could not be changed\\n"
}`,
  ],
  [
    "ini_restore",
    `ini_set("memory_limit", "128M")
ini_restore("memory_limit")

echo "memory_limit restored\\n"`,
  ],
  [
    "php_ini_scanned_files",
    `let $scanned = php_ini_scanned_files()

if ($scanned === false) {
    echo "No scanned php.ini files\\n"
}`,
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
    "str_word_count",
    `let $summary = strip_tags("<p>O'Reilly-Smith shipped invoice A-100.</p>")
let $words = str_word_count($summary)

echo "Summary words: " . $words . "\\n"`,
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
    "strnatcmp",
    `let $before = strnatcmp("file9", "file10")

echo "Natural compare: " . $before . "\\n"`,
  ],
  [
    "strnatcasecmp",
    `let $same = strnatcasecmp("Image2", "image2")

echo "Natural case compare: " . $same . "\\n"`,
  ],
  [
    "levenshtein",
    `let $submitted = "kitten"
let $known = "sitting"
let $distance = levenshtein($submitted, $known)

echo "Edit distance: " . $distance . "\\n"`,
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
    "htmlspecialchars",
    `let $name = "Tom & Jerry"
let $html = "<strong>" . htmlspecialchars($name) . "</strong>"

echo $html . "\\n"`,
  ],
  [
    "htmlspecialchars_decode",
    `let $stored = "&lt;strong&gt;Tom &amp; Jerry&lt;/strong&gt;"
let $preview = htmlspecialchars_decode($stored)

echo $preview . "\\n"`,
  ],
  [
    "strip_tags",
    `let $html = "<p>Hello <strong>Ada</strong></p>"
let $plain = strip_tags($html)

echo $plain . "\\n"`,
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
    "soundex",
    `let $left = soundex("Euler")
let $right = soundex("Ellery")

echo "Same bucket: " . ($left === $right ? "yes" : "no") . "\\n"`,
  ],
  [
    "wordwrap",
    `let $body = "The quick brown fox jumps"
let $wrapped = wordwrap($body, 10)

echo $wrapped . "\\n"`,
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
    "substr_replace",
    `let $filename = "invoice-2026-draft.txt"
let $finalName = substr_replace($filename, "-final", -4, 0)

echo $finalName . "\\n"`,
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
    "intdiv",
    `let $items = 47
let $boxSize = 12
let $fullBoxes = intdiv($items, $boxSize)

echo "Full boxes: " . $fullBoxes . "\\n"`,
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
    "round",
    `let $subtotal = 12.345
let $display = round($subtotal, 2)

echo "Subtotal: " . $display . "\\n"`,
  ],
  [
    "number_format",
    `let $total = 1234.567
let $display = number_format($total, 2, ".", ",")

echo "Invoice total: $" . $display . "\\n"`,
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
    "Use `quoted_printable_encode()` when a mail or MIME payload needs mostly readable text while still escaping bytes such as `=` and line breaks for transfer.",
  ],
  [
    "quoted_printable_decode",
    "Use `quoted_printable_decode()` at the input boundary for stored mail parts or MIME payloads so later code sees the original byte string.",
  ],
  [
    "nl2br",
    "Use `nl2br()` when plain-text notes, comments, or logs need an HTML preview while preserving where the original newline boundaries were.",
  ],
  [
    "htmlspecialchars",
    "Use `htmlspecialchars()` before inserting plain text into an HTML fragment so names, labels, or messages cannot be interpreted as markup.",
  ],
  [
    "htmlspecialchars_decode",
    "Use `htmlspecialchars_decode()` when compatibility code receives text that was escaped with `htmlspecialchars()` and needs the original display text back.",
  ],
  [
    "strip_tags",
    "Use `strip_tags()` when a compatibility path needs a plain-text summary from trusted HTML-like content before indexing, logging, or comparing labels.",
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
    "str_word_count",
    "Use the default scalar `str_word_count()` form for simple plain-text validation thresholds, such as detecting empty summaries after tags are removed. Echo currently implements the count return mode; PHP's array return modes and extra character list are separate compatibility work.",
  ],
  [
    "file_exists",
    "Use this before loading optional local files so missing configuration can be handled deliberately instead of failing later.",
  ],
  [
    "clearstatcache",
    "Use `clearstatcache()` when compatibility code deliberately refreshes filesystem metadata after creating or replacing files. Echo currently treats it as a no-op because it does not model PHP's stat or realpath cache.",
  ],
  [
    "crc32",
    "Use `crc32()` when existing PHP code expects a compact checksum for duplicate detection, export validation, or quick corruption checks. It is intentionally small and fast, so keep it out of security-sensitive decisions.",
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
    "Use `filectime()` when permission, owner, or other inode metadata changes matter to an audit or cache invalidation. It is not a portable creation timestamp.",
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
    "Use `fdiv()` when a metric should keep IEEE floating-point behavior at boundary values, such as reporting `INF` for a saturated ratio instead of throwing away the rest of a reporting path.",
  ],
  [
    "fpow",
    "Use `fpow()` for projections or scaling formulas that should always stay in floating-point space, even when the inputs happen to be whole numbers.",
  ],
  [
    "file_get_contents",
    "Use `file_get_contents()` when code needs the whole local file or a bounded slice in memory, such as previewing a report header, loading a small JSON config, or checking the tail of a log.",
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
    "Use `sys_get_temp_dir()` when code needs a scratch location without hard-coding `/tmp`, such as staging uploads, exports, or generated reports before moving them into application storage.",
  ],
  [
    "tempnam",
    "Use `tempnam()` when multiple workers might stage files in the same directory and each needs a distinct path before an atomic `rename()` publishes the finished artifact.",
  ],
  [
    "readlink",
    "Use `readlink()` when a deployment, cache, or storage layout represents the active version as a symbolic link and needs to report or validate the stored target.",
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
    "Use `touch()` when code needs a marker file or a controlled modification timestamp, such as recording that generated cache contents are fresh.",
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
    "Use `putenv()` when the current process needs to pass a derived setting to later environment-aware work. It changes process environment state, so keep the assignment close to the code that needs it.",
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
  [
    "strlen",
    "Use `strlen()` when byte length matters for protocol frames, upload limits, or fixed-width validation rather than user-visible character count.",
  ],
  [
    "strtoupper",
    "Use `strtoupper()` to normalize ASCII-style identifiers, status codes, or case-insensitive labels before comparison or display.",
  ],
  [
    "strtolower",
    "Use `strtolower()` to canonicalize email domains, flags, or lookup keys before storing or comparing them.",
  ],
  [
    "ucwords",
    "Use `ucwords()` for human-facing titles where each word should begin with an uppercase letter, such as headings or generated labels.",
  ],
  [
    "ucfirst",
    "Use `ucfirst()` when a single status, sentence, or label needs an initial capital without changing the rest of the string.",
  ],
  [
    "lcfirst",
    "Use `lcfirst()` to turn class-like or title-cased labels into lower-camel-style field names while leaving later characters untouched.",
  ],
  [
    "strrev",
    "Use `strrev()` for byte-order transformations such as simple palindrome checks, suffix handling, or legacy formats that store text reversed.",
  ],
  [
    "str_rot13",
    "Use `str_rot13()` only for PHP-compatible ROT13 text transformations such as old fixtures, examples, or reversible non-secret obfuscation.",
  ],
  [
    "soundex",
    "Use `soundex()` as a coarse PHP-compatible phonetic key for short ASCII names in legacy matching workflows. It is not a general fuzzy search algorithm.",
  ],
  [
    "wordwrap",
    "Use `wordwrap()` when text needs fixed-width lines for terminal output, legacy files, or previews before it leaves the runtime.",
  ],
  [
    "ord",
    "Use `ord()` when a parser or binary format needs the numeric value of the first byte in a string.",
  ],
  [
    "chr",
    "Use `chr()` to build a one-byte string from a numeric code, such as constructing separators, control bytes, or protocol markers.",
  ],
  [
    "bin2hex",
    "Use `bin2hex()` when raw bytes need a printable lowercase hexadecimal representation for logs, manifests, or fixtures.",
  ],
  [
    "hex2bin",
    "Use `hex2bin()` when a hexadecimal fixture, digest, or protocol field must be converted back to raw bytes.",
  ],
  [
    "base64_encode",
    "Use `base64_encode()` to carry binary data through text-only fields such as JSON payloads, headers, or form values.",
  ],
  [
    "base64_decode",
    "Use `base64_decode()` at input boundaries where text payloads need to become bytes again before validation or parsing.",
  ],
  [
    "trim",
    "Use `trim()` to remove surrounding whitespace from input values such as emails, IDs, or CSV cells before validation.",
  ],
  [
    "ltrim",
    "Use `ltrim()` when only leading padding or indentation should be removed while preserving meaningful trailing characters.",
  ],
  [
    "rtrim",
    "Use `rtrim()` for line-oriented input where trailing newlines or separators should be removed without touching leading spacing.",
  ],
  [
    "addslashes",
    "Use `addslashes()` for PHP-compatible escaping of quotes, backslashes, and NUL bytes when legacy code expects that exact format.",
  ],
  [
    "stripslashes",
    "Use `stripslashes()` to decode strings that were previously escaped with PHP slash escaping before comparison or display.",
  ],
  [
    "stripcslashes",
    "Use `stripcslashes()` at input boundaries for legacy configuration or fixture values that encode control bytes with C-style escapes. Inspect decoded control bytes with `bin2hex()` when logs or tests need stable visible output.",
  ],
  [
    "quotemeta",
    "Use `quotemeta()` when user text should be treated as literal regex text by escaping regexp metacharacters.",
  ],
  [
    "str_contains",
    "Use `str_contains()` to make substring gates read directly, such as checking whether a token, delimiter, or marker is present before parsing further.",
  ],
  [
    "str_starts_with",
    "Use `str_starts_with()` for prefix routing, feature flags, or path checks where only the beginning of the string should decide the branch.",
  ],
  [
    "str_ends_with",
    "Use `str_ends_with()` for suffix checks such as file extensions, generated IDs, or sentinel markers at the end of a value.",
  ],
  [
    "str_repeat",
    "Use `str_repeat()` to build predictable padding, separators, masks, or fixed-width placeholders from a known unit string.",
  ],
  [
    "str_pad",
    "Use `str_pad()` when values need a fixed display width, such as report columns, numeric codes, or aligned console output.",
  ],
  [
    "str_split",
    "Use `str_split()` when a byte string needs to be processed in equal chunks, such as grouping short codes or fixed-width records.",
  ],
  [
    "chunk_split",
    "Use `chunk_split()` for compatibility formatting that inserts separators at regular byte intervals, such as wrapped encoded payloads.",
  ],
  [
    "substr",
    "Use `substr()` when a field is located by byte offset, such as protocol headers, fixed-width exports, or known prefixes.",
  ],
  [
    "strpos",
    "Use `strpos()` when the first byte position of a case-sensitive token decides how to split or validate a string.",
  ],
  [
    "stripos",
    "Use `stripos()` when token position matters but user capitalization should not affect the result.",
  ],
  [
    "strrpos",
    "Use `strrpos()` to find the last occurrence of a separator, such as splitting a path, identifier, or dotted name at its final boundary.",
  ],
  [
    "strripos",
    "Use `strripos()` for the last position of a token when capitalization varies, such as case-insensitive extension or header parsing.",
  ],
  [
    "strstr",
    "Use `strstr()` to keep the tail of a string from the first delimiter onward, especially when the delimiter itself remains meaningful.",
  ],
  [
    "strchr",
    "Use `strchr()` as the PHP alias for `strstr()` when maintaining code that already uses the C-style name.",
  ],
  [
    "stristr",
    "Use `stristr()` when extracting a tail after a marker but the marker may arrive in different capitalization.",
  ],
  [
    "strrchr",
    "Use `strrchr()` to keep the final suffix beginning at a separator, such as the last extension or namespace segment.",
  ],
  [
    "strpbrk",
    "Use `strpbrk()` when any one of several delimiter characters can start the interesting part of a string.",
  ],
  [
    "strspn",
    "Use `strspn()` to measure a leading run of allowed bytes before validating or slicing a token.",
  ],
  [
    "strcspn",
    "Use `strcspn()` to measure how much text appears before the first forbidden byte or delimiter.",
  ],
  [
    "substr_count",
    "Use `substr_count()` to count fixed markers such as separators, placeholders, or line tokens before choosing a parser branch.",
  ],
  [
    "substr_compare",
    "Use `substr_compare()` when a specific byte range must be compared without first allocating a separate substring.",
  ],
  [
    "substr_replace",
    "Use `substr_replace()` for byte-position rewrites such as inserting a suffix before an extension, redacting a known token range, or replacing a fixed-width field.",
  ],
  [
    "strcmp",
    "Use `strcmp()` for exact binary-safe ordering or equality checks where PHP's string comparison result is part of compatibility behavior.",
  ],
  [
    "strcasecmp",
    "Use `strcasecmp()` for case-insensitive ordering of ASCII-style labels such as headers, command names, or status values.",
  ],
  [
    "strnatcmp",
    "Use `strnatcmp()` when labels contain numeric suffixes and `file9` should sort before `file10`. Treat only the sign of the return value as meaningful.",
  ],
  [
    "strnatcasecmp",
    "Use `strnatcasecmp()` for natural ordering when ASCII capitalization should not affect label order.",
  ],
  [
    "levenshtein",
    "Use `levenshtein()` for small compatibility checks where byte-based edit distance is enough to flag close spellings or labels.",
  ],
  [
    "strncmp",
    "Use `strncmp()` when only a fixed leading byte range should participate in an exact comparison.",
  ],
  [
    "strncasecmp",
    "Use `strncasecmp()` for case-insensitive prefix comparison without slicing the input first.",
  ],
  [
    "explode",
    "Use `explode()` to turn a delimited field into parts before validating count, order, and required values.",
  ],
  [
    "implode",
    "Use `implode()` to serialize already validated values into a delimiter-separated label, path segment, or report row.",
  ],
  [
    "join",
    "Use `join()` as the PHP alias for `implode()` when compatibility code already describes concatenating an array of parts.",
  ],
  [
    "rawurlencode",
    "Use `rawurlencode()` for RFC 3986 path or query components where spaces should become `%20` rather than `+`.",
  ],
  [
    "rawurldecode",
    "Use `rawurldecode()` when RFC 3986 percent-encoded data should be decoded without treating plus signs as spaces.",
  ],
  [
    "urlencode",
    "Use `urlencode()` for classic form-style query values where spaces are encoded as `+` for PHP compatibility.",
  ],
  [
    "urldecode",
    "Use `urldecode()` for form-style query values where both percent escapes and `+` space encoding must be interpreted.",
  ],
  [
    "array_is_list",
    "Use `array_is_list()` to distinguish sequential list-shaped arrays from keyed maps before encoding, merging, or validating payload shape.",
  ],
  [
    "array_values",
    "Use `array_values()` to reindex a filtered or keyed array when downstream code expects consecutive numeric positions.",
  ],
  [
    "array_keys",
    "Use `array_keys()` when the key set itself is the data, such as validating required fields or building a column header list.",
  ],
  [
    "array_fill",
    "Use `array_fill()` to initialize a positional array with the same default value before filling selected slots.",
  ],
  [
    "array_fill_keys",
    "Use `array_fill_keys()` to turn a known key list into a lookup table with a shared initial value.",
  ],
  [
    "array_combine",
    "Use `array_combine()` when separate key and value columns have already been validated to have matching lengths.",
  ],
  [
    "array_pad",
    "Use `array_pad()` to extend short positional data to a required width while preserving the original values.",
  ],
  [
    "array_slice",
    "Use `array_slice()` to take a page, preview, or bounded segment from an array without mutating the original collection.",
  ],
  [
    "array_chunk",
    "Use `array_chunk()` to batch values into fixed-size groups for reports, queue dispatch, or paginated display.",
  ],
  [
    "array_merge",
    "Use `array_merge()` when appending list values or layering associative defaults should follow PHP's merge rules.",
  ],
  [
    "array_replace",
    "Use `array_replace()` when later arrays should override values at the same keys while preserving unrelated entries.",
  ],
  [
    "array_reverse",
    "Use `array_reverse()` for newest-first displays, backtracking steps, or stack-like presentation without changing the source array.",
  ],
  [
    "array_flip",
    "Use `array_flip()` to convert a list of allowed scalar values into a fast membership lookup by key.",
  ],
  [
    "array_count_values",
    "Use `array_count_values()` to summarize repeated scalar values such as tags, statuses, or codes into frequency counts.",
  ],
  [
    "array_key_exists",
    "Use `array_key_exists()` when the distinction between a missing key and a present key with `null` matters.",
  ],
  [
    "key_exists",
    "Use `key_exists()` as the PHP alias for `array_key_exists()` when maintaining older code that uses the shorter name.",
  ],
  [
    "array_key_first",
    "Use `array_key_first()` to inspect insertion order without rewinding or mutating the array pointer.",
  ],
  [
    "array_key_last",
    "Use `array_key_last()` to find the most recently appended or final configured key without iterating manually.",
  ],
  [
    "in_array",
    "Use `in_array()` for membership checks against a small value list, especially when strict comparison protects numeric strings from coercion.",
  ],
  [
    "array_search",
    "Use `array_search()` when membership is not enough and the matching key must be used for a later update or diagnostic.",
  ],
  [
    "array_sum",
    "Use `array_sum()` to total numeric columns such as quantities, durations, or invoice lines after validating the array contents.",
  ],
  [
    "array_product",
    "Use `array_product()` for multiplicative totals such as scale factors, probabilities, or repeated quantity multipliers.",
  ],
  [
    "count",
    "Use `count()` to validate collection size, enforce limits, or explain how many records will be processed.",
  ],
  [
    "sizeof",
    "Use `sizeof()` as the PHP alias for `count()` when compatibility code uses that spelling for collection length.",
  ],
  [
    "gettype",
    "Use `gettype()` in diagnostics and compatibility branches where PHP's textual type name is the expected output.",
  ],
  [
    "is_array",
    "Use `is_array()` before array-specific operations so scalar inputs can be rejected with a clear validation message.",
  ],
  [
    "is_countable",
    "Use `is_countable()` before calling `count()` on dynamic input that might be a scalar, object, or collection.",
  ],
  [
    "is_iterable",
    "Use `is_iterable()` before a `foreach`-style path so arrays and traversable values are accepted while scalars are rejected.",
  ],
  [
    "is_numeric",
    "Use `is_numeric()` to accept strings that PHP can treat as numbers while still rejecting arbitrary text before conversion.",
  ],
  [
    "is_null",
    "Use `is_null()` when a present `null` has different meaning from false, zero, an empty string, or a missing field.",
  ],
  [
    "is_bool",
    "Use `is_bool()` to keep configuration flags or decoded payload fields from accepting stringy truth values accidentally.",
  ],
  [
    "is_int",
    "Use `is_int()` when fractional numbers and numeric strings must be rejected before arithmetic or indexing.",
  ],
  [
    "is_integer",
    "Use `is_integer()` as the PHP alias for `is_int()` in codebases that use the longer integer spelling.",
  ],
  [
    "is_long",
    "Use `is_long()` as the PHP alias for `is_int()` when porting older PHP code that names integer checks this way.",
  ],
  [
    "is_float",
    "Use `is_float()` when decimal numeric values should be accepted but integers, strings, and booleans should not.",
  ],
  [
    "is_double",
    "Use `is_double()` as the PHP alias for `is_float()` in compatibility code that still uses double terminology.",
  ],
  [
    "is_finite",
    "Use `is_finite()` to reject `INF`, `-INF`, and `NAN` before persisting metrics or using values in comparisons.",
  ],
  [
    "is_infinite",
    "Use `is_infinite()` to branch on saturated calculations such as divide-by-zero ratios without confusing them with ordinary large values.",
  ],
  [
    "is_nan",
    "Use `is_nan()` to catch invalid floating-point results that are not equal to themselves and should not be sorted or serialized as normal numbers.",
  ],
  [
    "is_object",
    "Use `is_object()` when dynamic data must expose object behavior before property access or method dispatch.",
  ],
  [
    "is_resource",
    "Use `is_resource()` for legacy PHP APIs that return handles and need to be checked before reads, writes, or cleanup.",
  ],
  [
    "is_string",
    "Use `is_string()` before string parsing, trimming, or length checks on mixed input from configuration or decoded payloads.",
  ],
  [
    "is_scalar",
    "Use `is_scalar()` when only simple printable values such as booleans, integers, floats, and strings should pass through.",
  ],
  [
    "strval",
    "Use `strval()` when PHP-compatible string coercion is required before concatenation, logging, or building keys.",
  ],
  [
    "boolval",
    "Use `boolval()` when compatibility code needs PHP truthiness as an explicit value rather than an implicit branch condition.",
  ],
  [
    "intval",
    "Use `intval()` to apply PHP integer conversion deliberately before indexing, limiting, or serializing a numeric setting.",
  ],
  [
    "floatval",
    "Use `floatval()` when a numeric string or scalar should enter floating-point math using PHP conversion rules.",
  ],
  [
    "doubleval",
    "Use `doubleval()` as the PHP alias for `floatval()` where older code uses double wording for floating conversion.",
  ],
  [
    "abs",
    "Use `abs()` to normalize signed differences, offsets, or deltas before comparing magnitudes.",
  ],
  [
    "bindec",
    "Use `bindec()` to decode binary text fields from flags, permissions, or protocol fixtures into numeric values.",
  ],
  [
    "decbin",
    "Use `decbin()` when a number needs a binary text representation for debugging masks, permissions, or protocol examples.",
  ],
  [
    "dechex",
    "Use `dechex()` to display numeric IDs, colors, masks, or digests in lowercase hexadecimal form.",
  ],
  ["decoct", "Use `decoct()` when permissions or legacy fields should be shown in octal notation."],
  [
    "hexdec",
    "Use `hexdec()` to parse hexadecimal user input, color components, or protocol values into numbers.",
  ],
  [
    "octdec",
    "Use `octdec()` to parse octal permission strings or legacy base-8 fields before numeric comparison.",
  ],
  [
    "base_convert",
    "Use `base_convert()` when compatibility code needs to move textual numbers between uncommon bases without custom parsing.",
  ],
  [
    "deg2rad",
    "Use `deg2rad()` before passing user-facing degree values into trigonometric functions that expect radians.",
  ],
  [
    "rad2deg",
    "Use `rad2deg()` after trigonometric or geometry calculations when the displayed result should be in degrees.",
  ],
  [
    "sin",
    "Use `sin()` for periodic calculations such as wave positions, rotations, or geometry where the input angle is in radians.",
  ],
  [
    "cos",
    "Use `cos()` for horizontal components, projections, or periodic offsets based on a radian angle.",
  ],
  [
    "tan",
    "Use `tan()` for slope or tangent calculations from a radian angle, with caller checks near vertical asymptotes.",
  ],
  [
    "asin",
    "Use `asin()` to recover an angle from a sine ratio after ensuring the input is inside `-1..1`.",
  ],
  [
    "acos",
    "Use `acos()` to recover an angle from a cosine ratio, commonly after clamping floating-point drift back into range.",
  ],
  [
    "atan",
    "Use `atan()` to convert a slope or ratio into a radian angle without needing a separate quadrant.",
  ],
  [
    "atan2",
    "Use `atan2()` when both coordinates are available and the result must preserve the correct quadrant.",
  ],
  [
    "sinh",
    "Use `sinh()` for hyperbolic geometry, curves, or compatibility calculations that specifically require hyperbolic sine.",
  ],
  [
    "cosh",
    "Use `cosh()` for hyperbolic cosine calculations such as catenary-style curves or PHP parity tests.",
  ],
  [
    "tanh",
    "Use `tanh()` when a value should be smoothly compressed toward `-1..1`, such as activation-like math or normalized curves.",
  ],
  [
    "asinh",
    "Use `asinh()` to invert hyperbolic sine for values across the real number line without a restricted input range.",
  ],
  ["acosh", "Use `acosh()` to invert hyperbolic cosine after checking the input is at least `1`."],
  ["atanh", "Use `atanh()` to invert hyperbolic tangent for inputs strictly between `-1` and `1`."],
  [
    "intdiv",
    "Use `intdiv()` when only complete integer groups should count, such as full boxes, consumed pages, or batch slots, and fractional remainders must be discarded toward zero.",
  ],
  [
    "ceil",
    "Use `ceil()` when partial units must round upward, such as pages, billing blocks, or chunk counts.",
  ],
  [
    "floor",
    "Use `floor()` when partial units should be discarded, such as whole completed steps or lower-bound bucket numbers.",
  ],
  [
    "round",
    "Use `round()` when a measurement, subtotal, or score needs a fixed decimal precision for display or reporting while retaining PHP's default half-away-from-zero behavior.",
  ],
  [
    "number_format",
    "Use `number_format()` at display boundaries for invoices, reports, and summaries that need stable thousands grouping. Keep calculations numeric and format only when building output text.",
  ],
  [
    "sqrt",
    "Use `sqrt()` for square-root calculations after validating that negative inputs are not meaningful for the domain.",
  ],
  [
    "hypot",
    "Use `hypot()` to compute Euclidean distance from components without manually squaring and summing them.",
  ],
  [
    "exp",
    "Use `exp()` for exponential growth, decay, or probability formulas based on Euler's number.",
  ],
  [
    "expm1",
    "Use `expm1()` when calculating `exp(x) - 1` for small values where direct subtraction would lose precision.",
  ],
  [
    "log",
    "Use `log()` for natural logarithms or explicit-base calculations after checking the value is positive.",
  ],
  [
    "log10",
    "Use `log10()` for orders of magnitude, decimal scaling, or digit-count style estimates.",
  ],
  [
    "log1p",
    "Use `log1p()` when calculating `log(1 + x)` for small values where direct addition would lose precision.",
  ],
  [
    "pow",
    "Use `pow()` for exponentiation in formulas where PHP-compatible numeric coercion and result typing are expected.",
  ],
  [
    "pi",
    "Use `pi()` when geometry or trigonometry code needs PHP's built-in constant value rather than a hand-written approximation.",
  ],
  [
    "fmod",
    "Use `fmod()` for floating-point remainders such as wrapping phases, positions, or cyclic measurements.",
  ],
  [
    "chdir",
    "Use `chdir()` when a script must run a group of relative-path operations from a known directory, then restore its original location.",
  ],
  [
    "getcwd",
    "Use `getcwd()` to anchor relative file operations, build diagnostics, or restore the process directory after a temporary change.",
  ],
  [
    "is_dir",
    "Use `is_dir()` before directory-specific traversal, cleanup, or creation logic so files with the same path are rejected.",
  ],
  [
    "is_file",
    "Use `is_file()` before reading or serving a path when directories, sockets, and special files are not valid inputs.",
  ],
  [
    "is_link",
    "Use `is_link()` when symlink entries need separate handling from their targets, such as release pointers or shared upload aliases.",
  ],
  [
    "is_writeable",
    "Use `is_writeable()` as the PHP alias for `is_writable()` when preserving older spelling in deployment or cache checks.",
  ],
  [
    "dirname",
    "Use `dirname()` to derive the parent directory for generated files, uploads, or path validation before creating or checking the directory.",
  ],
  [
    "flush",
    "Use `flush()` when buffered output should be pushed toward the client or next output layer during a long-running response.",
  ],
  [
    "ob_flush",
    "Use `ob_flush()` to pass the current output buffer onward while keeping the buffer active for later writes.",
  ],
  [
    "ob_clean",
    "Use `ob_clean()` to discard buffered output without ending the active buffer, useful after a render branch is rejected.",
  ],
  [
    "ob_end_flush",
    "Use `ob_end_flush()` when a buffer is complete and should be sent onward while removing that buffering layer.",
  ],
  [
    "ob_end_clean",
    "Use `ob_end_clean()` when captured output should be discarded and the buffering layer should be removed.",
  ],
  [
    "ob_get_length",
    "Use `ob_get_length()` to enforce response-size limits or decide whether a buffer has produced any bytes yet.",
  ],
  [
    "ob_get_level",
    "Use `ob_get_level()` to debug nested buffering or restore the output stack to a known depth after rendering.",
  ],
  [
    "ob_implicit_flush",
    "Use `ob_implicit_flush()` when streaming-style output should flush after each write instead of waiting for manual flush calls.",
  ],
  [
    "define",
    "Use `define()` for PHP-compatible constants that must be named dynamically or declared outside class and namespace syntax.",
  ],
  [
    "microtime",
    "Use `microtime()` for PHP-compatible wall-clock timing labels; prefer monotonic timers for measuring elapsed durations in new Echo code.",
  ],
  [
    "gettimeofday",
    "Use `gettimeofday()` when compatibility code expects PHP's structured wall-clock timestamp with named seconds and microseconds fields.",
  ],
  [
    "hrtime",
    "Use `hrtime()` when compatibility code expects PHP's high-resolution timestamp shape. New Echo code should prefer `std.time` timers for elapsed-duration measurement.",
  ],
  [
    "phpversion",
    "Use `phpversion()` in compatibility bootstraps and diagnostics that need to report the PHP surface Echo is targeting.",
  ],
  [
    "php_sapi_name",
    "Use `php_sapi_name()` for legacy compatibility branches that distinguish command-line execution from other PHP Server API names.",
  ],
  [
    "zend_version",
    "Use `zend_version()` for legacy diagnostics and version banners that expect a Zend Engine version label.",
  ],
  [
    "extension_loaded",
    "Use `extension_loaded()` for compatibility branches around optional PHP extensions. Echo currently returns false until extension metadata is modeled.",
  ],
  [
    "get_loaded_extensions",
    "Use `get_loaded_extensions()` when compatibility diagnostics need an extension list. Echo currently returns an empty array until extension metadata is modeled.",
  ],
  [
    "get_extension_funcs",
    "Use `get_extension_funcs()` for compatibility checks that inspect functions exposed by optional PHP extensions. Echo currently returns false until extension function metadata is modeled.",
  ],
  [
    "php_ini_loaded_file",
    "Use `php_ini_loaded_file()` for diagnostics that need to report PHP configuration input. Echo currently returns false because it does not load php.ini files.",
  ],
  [
    "get_cfg_var",
    "Use `get_cfg_var()` for compatibility checks around PHP configuration options. Echo currently returns false because it does not load PHP configuration values.",
  ],
  [
    "ini_get",
    "Use `ini_get()` for compatibility checks around PHP ini options. Echo currently returns false because it does not model PHP ini option values.",
  ],
  [
    "ini_get_all",
    "Use `ini_get_all()` when compatibility diagnostics summarize available PHP ini options. Echo currently returns an empty array for the core registry and false for named extensions.",
  ],
  [
    "ini_parse_quantity",
    "Use `ini_parse_quantity()` to normalize PHP shorthand quantities such as memory limits into byte counts. Echo supports PHP's integer bases and K/M/G multipliers.",
  ],
  [
    "get_include_path",
    "Use `get_include_path()` when compatibility code checks PHP's include search path before resolving legacy includes. Echo currently returns false because it does not model PHP ini option values.",
  ],
  [
    "connection_aborted",
    "Use `connection_aborted()` when compatibility code checks whether a web client disconnected before doing more response work. Echo currently returns 0 because it does not model an abortable client connection.",
  ],
  [
    "connection_status",
    "Use `connection_status()` when compatibility code checks PHP's connection status bitfield. Echo currently returns 0 because it does not model an abortable client connection.",
  ],
  [
    "ignore_user_abort",
    "Use `ignore_user_abort()` when compatibility code records whether work should continue after a web client disconnects. Echo tracks the setting as process-local state but does not model an abortable client connection.",
  ],
  [
    "headers_list",
    "Use `headers_list()` when compatibility diagnostics inspect queued HTTP response headers. Echo currently returns an empty array because it does not model an HTTP header layer.",
  ],
  [
    "headers_sent",
    "Use `headers_sent()` before compatibility code queues response headers after output may have started. Echo currently returns false because it does not model an HTTP header layer.",
  ],
  [
    "header",
    "Use `header()` when compatibility code queues HTTP response headers before writing a body. Echo currently treats it as a no-op because it does not model an HTTP header layer.",
  ],
  [
    "header_remove",
    "Use `header_remove()` when compatibility code clears queued response headers before switching response paths. Echo currently treats it as a no-op because it does not model an HTTP header layer.",
  ],
  [
    "http_response_code",
    "Use `http_response_code()` when compatibility code records or inspects an HTTP status before writing a body. Echo tracks this process-local status value but does not send a status line because it does not model an HTTP header layer.",
  ],
  [
    "ini_set",
    "Use `ini_set()` when compatibility code attempts to change a PHP ini option and needs a fallback. Echo currently returns false because it does not model mutable PHP ini option values.",
  ],
  [
    "ini_alter",
    "Use `ini_alter()` for legacy compatibility code that calls PHP's alias of `ini_set()`. Echo currently returns false because it does not model mutable PHP ini option values.",
  ],
  [
    "ini_restore",
    "Use `ini_restore()` to preserve compatibility cleanup around localized PHP ini overrides. Echo currently treats it as a no-op because it does not model mutable PHP ini option values.",
  ],
  [
    "php_ini_scanned_files",
    "Use `php_ini_scanned_files()` for diagnostics that need to report extra PHP configuration files. Echo currently returns false because it does not scan PHP configuration directories.",
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

  if (!note) {
    throw new Error(`Missing documentation note for PHP builtin: ${builtin.name}`);
  }

  return note;
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
        tags: ["install", "path"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Install the ",
              { code: "xo" },
              " command and keep it on your path. Echo is still early, so the public installer and release flow are evolving.",
            ],
          },
          {
            kind: "code",
            code: "xo --help\nxo run app.php\nxo build app.php -o app",
            language: "shellscript",
          },
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
          { kind: "code", code: "xo run examples/hello.php", language: "shellscript" },
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
          {
            kind: "code",
            code: "xo build examples/hello.php -o /tmp/hello\n/tmp/hello",
            language: "shellscript",
          },
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
    id: "quickstart",
    path: "/docs/quickstart",
    navGroup: "Getting Started",
    category: "Getting Started",
    title: "Quickstart",
    summary: "Run a PHP-compatible file, inspect the compiler output, and build a native binary.",
    tags: ["quickstart", "run", "build", "ast", "ir", "php"],
    aliases: ["first program", "hello world", "getting started"],
    sections: [
      {
        title: "Create a Program",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Start with ordinary PHP syntax. Echo accepts PHP-compatible source while the stricter Echo surface grows around it.",
            ],
          },
          {
            kind: "code",
            code: '<?php\n\necho "Hello from Echo\\n";',
          },
          {
            kind: "paragraph",
            text: [
              "This keeps the first program inside the PHP-compatible lane, which is the safest starting point when checking Echo against existing PHP habits.",
            ],
          },
        ],
      },
      {
        title: "Run It",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Use ",
              { code: "xo run" },
              " when you want the supported program to execute through Echo's compiler and runtime path.",
            ],
          },
          { kind: "code", code: "xo run hello.php", language: "shellscript" },
          {
            kind: "paragraph",
            text: [
              "Running through ",
              { code: "xo" },
              " exercises Echo's parser, lowering, and runtime instead of delegating to the system PHP binary.",
            ],
          },
        ],
      },
      {
        title: "Source Identity",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo tracks source identity separately from language mode. A registered source can be a PHP file, an Echo file, an ",
              { code: "xo" },
              " input, a standard-library module, a REPL snippet, or an anonymous test source.",
            ],
          },
          {
            kind: "code",
            code: "xo ast app.php\nxo ast app.echo\nxo ast app.xo",
            language: "shellscript",
          },
          {
            kind: "paragraph",
            text: [
              "These commands preserve where diagnostics and editor locations should point without changing which parser or semantic rules are used.",
            ],
          },
        ],
      },
      {
        title: "Inspect and Build",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Use the inspection commands while developing language features or checking how a program moves through the compiler pipeline.",
            ],
          },
          {
            kind: "code",
            code: "xo ast hello.php\nxo ir hello.php\nxo build hello.php -o /tmp/hello\n/tmp/hello",
            language: "shellscript",
          },
          {
            kind: "paragraph",
            text: [
              "The inspection commands are useful when a behavior is parsed correctly but still needs verification in lowering, runtime execution, or native build output.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "single-language-mode",
    path: "/docs/single-language-mode",
    navGroup: "Getting Started",
    category: "Getting Started",
    title: "Single Language Mode",
    summary: "Understand why .php, .echo, and .xo use one shared Echo/PHP language pipeline.",
    tags: ["source", "single language", "php", "echo", "xo"],
    aliases: ["single language", "shared parser", "php echo xo"],
    sections: [
      {
        title: "One Parser",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo compiles ",
              { code: ".php" },
              ", ",
              { code: ".echo" },
              ", and ",
              { code: ".xo" },
              " files as the same language. File extension affects ecosystem expectations and editor tooling, not parser rules or semantic validity.",
            ],
          },
          {
            kind: "code",
            code: "xo run app.php\nxo run app.echo\nxo run app.xo",
            language: "shellscript",
          },
          {
            kind: "paragraph",
            text: [
              "These commands enter the same compiler pipeline. A ",
              { code: ".php" },
              " file that uses Echo-only syntax may no longer run on stock PHP, but valid PHP remains valid Echo.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "roadmap",
    path: "/docs/roadmap",
    navGroup: "Getting Started",
    category: "Getting Started",
    title: "Roadmap",
    summary:
      "Track the near-term Echo work across PHP compatibility, Echo-native syntax, standard library modules, and compiler pipeline depth.",
    tags: ["roadmap", "status", "php compatibility", "compiler", "standard library"],
    aliases: ["project status", "planned work", "future work"],
    sections: [
      {
        title: "Compatibility First",
        tags: ["php", "fixtures", "builtins"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo grows by proving vertical slices through parser, AST, semantic analysis, lowering, runtime behavior, CLI execution, docs, and tests. PHP compatibility remains the floor while Echo-native features are added on top.",
            ],
          },
        ],
      },
      {
        title: "Near-Term Language Work",
        tags: ["hir", "mir", "ast", "control flow", "imports"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "The next language slices focus on keeping imports, typed bindings, Echo collections, receiver calls, HIR, MIR, and codegen models clean enough for more PHP built-ins, standard library modules, and Echo control flow.",
            ],
          },
          {
            kind: "code",
            code: 'from std use time\n\nlet $timer = time.timer()\n\nif ($timer.elapsed() > 16ms) {\n    echo "slow frame\\n"\n}',
          },
          {
            kind: "paragraph",
            text: [
              "This small program crosses the import model, duration literals, receiver calls, comparison rules, and conditional control flow, which makes it a useful shape for future vertical implementation work.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "data-structures",
    path: "/docs/data-structures",
    navGroup: "Language",
    category: "Language",
    title: "Data Structures",
    summary: "Choose the right collection, record, enum, range, or byte-buffer shape.",
    tags: [
      "data structures",
      "collections",
      "list",
      "array",
      "fixed array",
      "object",
      "class",
      "map",
      "set",
      "tuple",
      "enum",
      "range",
      "buffer",
    ],
    aliases: ["collections", "records", "values"],
    sections: [
      {
        title: "Choosing a Shape",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo keeps collection and record forms distinct. Use ",
              { code: "List" },
              " for ordered Echo values, ",
              { code: "Array" },
              " and ",
              { code: "Fixed Array" },
              " for contiguous storage, ",
              { code: "Map" },
              " and ",
              { code: "Set" },
              " for keyed and unique Echo collections, ",
              { code: "Object" },
              " for structural records, ",
              { code: "Class" },
              " for nominal class instances, ",
              { code: "Tuple" },
              " for inferred positional products, ",
              { code: "Enum" },
              " for nominal choices, ",
              { code: "Range" },
              " for iterable spans, and ",
              { code: "Buffer" },
              " for byte-oriented values.",
            ],
          },
          {
            kind: "code",
            code: 'let $items: list<string> = {}\n$items.push("draft")\n\nlet $fixedArray: array<number>[3] = [1, 2, 3]\nlet $tuple = (1, "draft", true)\nlet $range = 1..30\nlet $bytes = x"AABBEE"\n\nlet $user = {\n    id: 42\n    email: "admin@example.com"\n}: User',
          },
          {
            kind: "paragraph",
            text: [
              "Keep these shapes separate when designing APIs. Echo list mutation goes through receiver functions, structural objects use named fields, class instances come from ",
              { code: "new" },
              ", and PHP arrays stay available as the compatibility floor rather than the shape for every collection.",
            ],
          },
          {
            kind: "paragraph",
            text: [
              "PHP-compatible enums remain nominal singleton or backed enum cases. Echo-native enums extend that model with generic and payload-carrying variants for results, options, typed errors, parser states, and protocol states.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "data-structures-list",
    path: "/docs/data-structures/list",
    navGroup: "Language",
    category: "Data Structures",
    title: "List",
    summary: "Create Echo lists with brace literals and mutate them with list receiver functions.",
    tags: ["data structures", "list", "push", "collection"],
    aliases: ["lists", "linked list", "list push"],
    sections: [
      {
        title: "List Literals",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo lists use unkeyed brace literals. Add an explicit type after the symbol when the initializer cannot carry enough element information.",
            ],
          },
          {
            kind: "code",
            code: 'let $items: list<string> = {}\n$items.push("first")\n$items.push("second")\n\necho count($items)\necho "\\n"',
          },
          {
            kind: "paragraph",
            text: [
              "Use this form for empty lists or when the element type matters at the boundary. When the initializer is clear, prefer inference and write ",
              { code: 'let $items = {"first", "second"}' },
              ".",
            ],
          },
        ],
      },
      {
        title: "push",
        tags: ["push", "append", "mutation"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "push(T $value): list<T>" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "push()" },
              " appends a value to a list and updates the receiver when it is a local variable.",
            ],
          },
          {
            kind: "code",
            code: 'let $users: list<User> = {}\n$users.push({\n    id: 1\n    email: "first@example.test"\n}: User)\n\necho count($users)\necho "\\n"',
          },
          {
            kind: "paragraph",
            text: [
              "Use ",
              { code: "push()" },
              " for Echo lists instead of PHP ",
              { code: "$value[] = item" },
              ". The PHP append form is reserved for non-fixed PHP arrays, so it does not define list growth.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "data-structures-object",
    path: "/docs/data-structures/object",
    navGroup: "Language",
    category: "Data Structures",
    title: "Object",
    summary: "Use structural objects for named-field values.",
    tags: ["data structures", "object", "record", "fields"],
    aliases: ["structural object", "record"],
    sections: [
      {
        title: "Structural Values",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo structural objects are named-field values. They are useful when the data shape matters more than PHP class identity.",
            ],
          },
          {
            kind: "code",
            code: 'type User = {\n    const id: int\n    email: string\n}\n\nlet $user = User {\n    id: 1\n    email: "first@example.test"\n}\n\necho $user.email',
          },
          {
            kind: "paragraph",
            text: [
              "Use structural objects for request payloads, configuration records, and typed data that should be easy to construct and inspect.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "data-structures-class",
    path: "/docs/data-structures/class",
    navGroup: "Language",
    category: "Data Structures",
    title: "Class",
    summary: "Use classes for PHP-compatible class declarations and method surfaces.",
    tags: ["data structures", "class", "php compatibility", "methods"],
    aliases: ["classes", "methods"],
    sections: [
      {
        title: "PHP-Compatible Classes",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Classes preserve PHP-compatible declaration shape. Use them when existing PHP code, method lookup, or class identity is part of the API contract.",
            ],
          },
          {
            kind: "code",
            code: 'class ReportFormatter {\n    pub fn title($name) {\n        echo "Report: " . $name . "\\n"\n    }\n}',
          },
          {
            kind: "paragraph",
            text: [
              "Prefer structural objects for plain data and classes for method surfaces or interoperability boundaries. In Echo class bodies, unprefixed methods are private by default; add ",
              { code: "pub fn" },
              " for public methods.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "data-structures-array",
    path: "/docs/data-structures/array",
    navGroup: "Language",
    category: "Data Structures",
    title: "Array",
    summary: "Use PHP arrays for PHP-compatible indexed and keyed array behavior.",
    tags: ["data structures", "array", "php array", "compatibility"],
    aliases: ["arrays", "php arrays"],
    sections: [
      {
        title: "PHP Arrays",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Square brackets create PHP-compatible arrays. Use them when code depends on PHP array built-ins, keyed rows, or compatibility with existing PHP programs.",
            ],
          },
          {
            kind: "code",
            code: 'let $row = ["id" => "A-42", "status" => "ready"]\nlet $columns = array_keys($row)\n\necho join(", ", $columns)\necho "\\n"',
          },
          {
            kind: "paragraph",
            text: [
              "Use arrays for PHP compatibility code. Use Echo lists when you want an ordered Echo collection with list-specific receiver functions.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "data-structures-enum",
    path: "/docs/data-structures/enum",
    navGroup: "Language",
    category: "Data Structures",
    title: "Enum",
    summary:
      "Use nominal enum types for PHP-compatible singleton/backed cases and Echo-native payload variants.",
    tags: ["data structures", "enum", "backed enum", "payload enum", "match"],
    aliases: ["enums", "algebraic enum", "result", "option"],
    sections: [
      {
        title: "Pure Enums",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "A pure enum is a nominal type with singleton cases. This is the Echo form of PHP's pure enum model.",
            ],
          },
          {
            kind: "code",
            code: "enum Status {\n    Draft\n    Published\n    Archived\n}\n\nlet $status = Status::Draft",
          },
          {
            kind: "paragraph",
            text: [
              "Use pure enums when the case identity is the value and no scalar backing or payload data is needed.",
            ],
          },
        ],
      },
      {
        title: "Backed Enums",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "A backed enum gives every case a string or int identity for PHP-compatible scalar conversion.",
            ],
          },
          {
            kind: "code",
            code: 'enum Status: string {\n    Draft = "draft"\n    Published = "published"\n    Archived = "archived"\n}\n\nlet $status = Status::from("draft")\necho $status->value',
          },
          {
            kind: "paragraph",
            text: [
              "Backed enums are for compatibility with storage formats, HTTP payloads, and PHP APIs that exchange stable string or integer values.",
            ],
          },
        ],
      },
      {
        title: "Payload Enums",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo-native enums may carry payload data. A backed enum cannot have payload cases because scalar backing is identity while payload data is runtime state.",
            ],
          },
          {
            kind: "code",
            code: "enum Result<T, E> {\n    Ok(T)\n    Err(E)\n}\n\nenum FileError {\n    NotFound(path: string)\n    PermissionDenied(path: string)\n    InvalidEncoding(path: string, encoding: string)\n}",
          },
          {
            kind: "paragraph",
            text: [
              "Use payload enums for recoverable errors, parser results, protocol states, and AST-like values where each case may need different data.",
            ],
          },
        ],
      },
      {
        title: "Matching Enums",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Enum matches can destructure payloads and should become exhaustive under an explicit semantic profile.",
            ],
          },
          {
            kind: "code",
            code: "match result {\n    Ok(value) => compile(value)\n    Err(error) => report(error)\n}",
          },
          {
            kind: "paragraph",
            text: [
              "Exhaustive matching makes enum APIs useful for typed errors and state machines because adding a case surfaces the handling sites that must be updated.",
            ],
          },
        ],
      },
      {
        title: "PHP Enum Compatibility",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "PHP-compatible enums are nominal singleton enums with optional string or int backing. They may have methods and expose ",
              { code: "from" },
              ", ",
              { code: "tryFrom" },
              ", and ",
              { code: "value" },
              " for backed enums.",
            ],
          },
          {
            kind: "code",
            code: 'enum Status: string {\n    Draft = "draft"\n    Published = "published"\n\n    pub fn label(): string {\n        return match ($this) {\n            self::Draft => "Draft"\n            self::Published => "Published"\n        }\n    }\n}',
          },
          {
            kind: "paragraph",
            text: [
              "Treat PHP enums as the compatibility floor. Echo's ceiling includes generic and payload-carrying enums, but those forms are Echo-native and need lowering when targeting PHP-compatible output.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "standard-library",
    path: "/docs/std",
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
              " module root. Import a module when program behavior should come from Echo's standard library rather than PHP compatibility built-ins.",
            ],
          },
          {
            kind: "code",
            code: 'from std use time\n\nlet $timer = time.timer()\ntime.sleep(25ms)\nlet $elapsed = $timer.elapsed()\n\necho "Elapsed milliseconds: " . $elapsed.total_millis() . "\\n"',
          },
          {
            kind: "paragraph",
            text: [
              "Use standard library imports for Echo-native capabilities such as scheduling, networking, and introspection. A std API can be regular Echo source compiled through the normal pipeline or a trusted intrinsic that lowers to an approved runtime ABI. PHP built-ins remain available for compatibility code, while ",
              { code: "std" },
              " modules mark code that intentionally targets Echo's standard library surface.",
            ],
          },
        ],
      },
      {
        title: "net",
        tags: ["tcp", "network", "listen", "connect"],
        aliases: ["networking", "tcp server", "tcp connection"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "net" },
              " exposes TCP listener and connection APIs. Use it when an Echo program owns socket IO instead of shelling out to another process.",
            ],
          },
          {
            kind: "code",
            code: 'from std use net\n\nlet $server = net.listen("127.0.0.1:8080")\nlet $connection = net.accept($server)\nlet $request = net.read($connection, 4096)\n\nnet.write($connection, "received " . strlen($request) . " bytes\\n")\nnet.close($connection)',
          },
          {
            kind: "paragraph",
            text: [
              "This pattern keeps the listener, accepted connection, read buffer, response write, and close operation in one request path. Prefer it for low-level TCP services where the program needs direct control over connection lifetime.",
            ],
          },
        ],
      },
      {
        title: "http",
        tags: ["http", "response", "request"],
        aliases: ["http response", "http request"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "http" },
              " contains HTTP helpers built on Echo runtime types. The first supported surface formats plain text responses and reads requests from ",
              { code: "net" },
              " connections.",
            ],
          },
          {
            kind: "code",
            code: 'from std use http\nfrom std use net\n\nlet $connection = net.connect("127.0.0.1:8080")\nlet $response = http.responseText("ok\\n")\n\nnet.write($connection, $response)\nnet.close($connection)',
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
        title: "time",
        tags: ["sleep", "time", "scheduling"],
        aliases: ["sleep", "delay", "timer"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "time" },
              " is the planned home for exact time, monotonic timing, durations, periods, timers, and sleep. Module functions construct or access time values; receiver methods operate on those values.",
            ],
          },
          {
            kind: "code",
            code: 'from std use time\n\ntime.sleep(500ms)\n\nlet $timer = time.timer()\nrender()\n\nif ($timer.elapsed() > 16ms) {\n    echo "slow frame"\n}',
          },
          {
            kind: "paragraph",
            text: [
              "Use duration literals or constructors such as ",
              { code: "time.milliseconds(500)" },
              " and ",
              { code: "time.duration(milliseconds: 500)" },
              ". Raw numeric sleeps such as ",
              { code: "time.sleep(500)" },
              " are intentionally invalid because the unit is ambiguous.",
            ],
          },
        ],
      },
      {
        title: "reflect",
        tags: ["reflection", "type", "metadata"],
        aliases: ["introspection", "function metadata"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "reflect" },
              " inspects Echo-visible functions and values. It can see Echo standard library and userland metadata in addition to PHP compatibility functions.",
            ],
          },
          {
            kind: "code",
            code: 'from std use reflect\n\nlet $name = "time.sleep"\n\nif (reflect.exists($name)) {\n    echo $name . " returns " . reflect.returnType($name) . "\\n"\n}',
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
        title: "assert",
        tags: ["assert", "testing", "validation"],
        aliases: ["assertions", "test helpers"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "assert" },
              " provides assertion helpers for Echo test-style programs and small runtime checks.",
            ],
          },
          {
            kind: "code",
            code: 'from std use assert\n\nlet $payload = "signed:user-42"\nlet $parts = explode(":", $payload)\n\nassert.equals(count($parts), 2)\nassert.ok($parts[0] == "signed")',
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
    id: "standard-library-net",
    path: "/docs/std/net",
    navGroup: "Language",
    category: "Standard Library",
    title: "net",
    summary: "Open TCP listeners and connections, exchange bytes, and close sockets.",
    tags: ["standard library", "stdlib", "std", "net", "tcp", "network"],
    aliases: ["std.net", "networking", "tcp server", "tcp connection"],
    sections: [
      {
        title: "listen",
        tags: ["tcp", "network", "listen", "server"],
        aliases: ["tcp server", "listener"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "listen(string $address): TcpServer" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "listen()" },
              " opens a TCP listener bound to an address and returns a server handle that can accept inbound connections.",
            ],
          },
          {
            kind: "code",
            code: 'from std use net\n\nlet $server = net.listen("127.0.0.1:8080")\necho "Listening on 127.0.0.1:8080\\n"',
          },
          {
            kind: "paragraph",
            text: [
              "Use this when the Echo program owns the socket server. Keep the bound address explicit so local development and deployed listeners are easy to audit.",
            ],
          },
        ],
      },
      {
        title: "connect",
        tags: ["tcp", "network", "connect", "client"],
        aliases: ["tcp client", "connection"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "connect(string $address): TcpConnection" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "connect()" },
              " opens an outbound TCP connection to an address and returns a connection handle for reads and writes.",
            ],
          },
          {
            kind: "code",
            code: 'from std use net\n\nlet $connection = net.connect("127.0.0.1:8080")\nnet.write($connection, "ping\\n")\nnet.close($connection)',
          },
          {
            kind: "paragraph",
            text: [
              "Use this for clients that speak directly to another TCP service. Close the connection when the exchange is complete so resources are released predictably.",
            ],
          },
        ],
      },
      {
        title: "accept",
        tags: ["tcp", "network", "accept", "server"],
        aliases: ["accept connection", "inbound connection"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "accept(TcpServer $server): TcpConnection" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "accept()" },
              " waits for the next inbound connection on a server handle and returns a connection handle for that client.",
            ],
          },
          {
            kind: "code",
            code: 'from std use net\n\nlet $server = net.listen("127.0.0.1:8080")\nlet $connection = net.accept($server)\nnet.write($connection, "hello\\n")\nnet.close($connection)',
          },
          {
            kind: "paragraph",
            text: [
              "Use this after creating a listener when the program is ready to handle one client connection. Pair accepted connections with explicit writes, reads, and close calls.",
            ],
          },
        ],
      },
      {
        title: "read",
        tags: ["tcp", "network", "read", "bytes"],
        aliases: ["read bytes", "connection read"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "read(TcpConnection $connection, int $maxBytes): bytes" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "read()" },
              " reads up to the requested number of bytes from a connection and returns the bytes received.",
            ],
          },
          {
            kind: "code",
            code: 'from std use net\n\nlet $connection = net.connect("127.0.0.1:8080")\nlet $chunk = net.read($connection, 4096)\necho "Read " . strlen($chunk) . " bytes\\n"\nnet.close($connection)',
          },
          {
            kind: "paragraph",
            text: [
              "Use a bounded read size that fits the protocol or framing layer you expect. The returned bytes can then feed parsing, logging, or response decisions.",
            ],
          },
        ],
      },
      {
        title: "write",
        tags: ["tcp", "network", "write", "bytes"],
        aliases: ["write bytes", "connection write"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "write(TcpConnection $connection, bytes|string $data): int" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "write()" },
              " sends bytes or a string to a connection and returns the number of bytes written.",
            ],
          },
          {
            kind: "code",
            code: 'from std use net\n\nlet $connection = net.connect("127.0.0.1:8080")\nlet $written = net.write($connection, "status=ready\\n")\necho "Wrote " . $written . " bytes\\n"\nnet.close($connection)',
          },
          {
            kind: "paragraph",
            text: [
              "Use the returned byte count when a protocol needs to confirm that a complete message was sent or record how much data left the process.",
            ],
          },
        ],
      },
      {
        title: "close",
        tags: ["tcp", "network", "close", "connection"],
        aliases: ["close connection", "release socket"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "close(TcpConnection $connection): void" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "close()" },
              " closes a TCP connection and releases the underlying runtime resource.",
            ],
          },
          {
            kind: "code",
            code: 'from std use net\n\nlet $connection = net.connect("127.0.0.1:8080")\nnet.write($connection, "done\\n")\nnet.close($connection)',
          },
          {
            kind: "paragraph",
            text: [
              "Close connections at the end of each exchange so long-running programs do not keep sockets open after their work is finished.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "standard-library-http",
    path: "/docs/std/http",
    navGroup: "Language",
    category: "Standard Library",
    title: "http",
    summary: "Format HTTP response bytes for small services.",
    tags: ["standard library", "stdlib", "std", "http", "response", "request"],
    aliases: ["std.http", "http response", "http request"],
    sections: [
      {
        title: "responseText",
        tags: ["http", "response", "request"],
        aliases: ["http response", "http request"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "responseText(string $body): bytes" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "responseText()" },
              " wraps a string body in a plain HTTP response, including the status line and headers needed before writing the bytes to a connection.",
            ],
          },
          {
            kind: "code",
            code: 'from std use http\nfrom std use net\n\nlet $connection = net.connect("127.0.0.1:8080")\nlet $response = http.responseText("ok\\n")\n\nnet.write($connection, $response)\nnet.close($connection)',
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
        title: "readRequest",
        tags: ["http", "request", "read"],
        aliases: ["http request", "read request"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "readRequest(TcpConnection $connection): Request" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "readRequest()" },
              " reads and parses an HTTP request from an open TCP connection.",
            ],
          },
          {
            kind: "code",
            code: 'from std use http\nfrom std use net\n\nlet $server = net.listen("127.0.0.1:8080")\nlet $connection = net.accept($server)\nlet $request = http.readRequest($connection)\n\nnet.close($connection)',
          },
          {
            kind: "paragraph",
            text: [
              "Use this at the server boundary when request parsing should happen before routing, validation, or response generation.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "standard-library-time",
    path: "/docs/std/time",
    navGroup: "Language",
    category: "Standard Library",
    title: "time",
    summary: "Work with exact time, durations, monotonic timers, periods, and sleep.",
    tags: [
      "standard library",
      "stdlib",
      "std",
      "time",
      "sleep",
      "scheduling",
      "duration",
      "timer",
      "instant",
      "period",
    ],
    aliases: ["std.time", "sleep", "delay", "timer", "duration", "instant", "period"],
    sections: [
      {
        title: "Core Types",
        tags: ["instant", "duration", "period", "timer", "monotonic"],
        aliases: ["Instant", "MonoInstant", "Duration", "Period", "Timer"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "time" },
              " defines opaque values for wall-clock instants, monotonic instants, exact durations, calendar periods, and timers.",
            ],
          },
          {
            kind: "code",
            code: "namespace time\n\npub type Instant\npub type MonoInstant\npub type Duration\npub type Period\npub type Timer",
          },
          {
            kind: "paragraph",
            text: [
              "Construct these values through module functions and literals. Do not construct core time values by writing raw fields; their representation is a stdlib implementation detail.",
            ],
          },
          {
            kind: "paragraph",
            text: [
              { code: "Instant" },
              " is wall-clock Unix timeline time. ",
              { code: "MonoInstant" },
              " is monotonic runtime time for elapsed measurement. ",
              { code: "Duration" },
              " is exact elapsed machine time. ",
              { code: "Period" },
              " is calendar-relative human time.",
            ],
          },
        ],
      },
      {
        title: "Construction",
        tags: ["duration", "constructors", "literal"],
        aliases: ["duration literal", "time.duration", "time.milliseconds"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Use module-level functions for clocks, constructors, and runtime interaction. Duration values can be written as literals, built from a single unit, or built from named compound units.",
            ],
          },
          {
            kind: "paragraph",
            text: [
              "Echo standard library time calls use dot notation such as ",
              { code: "time.sleep(...)" },
              ". Do not use PHP namespace-call spelling such as ",
              { code: "time\\sleep(...)" },
              " for Echo-owned stdlib modules.",
            ],
          },
          {
            kind: "code",
            code: "let $literal = 500ms\nlet $single = time.milliseconds(500)\nlet $compound = time.duration(milliseconds: 500)\n\nlet $now = time.now()\nlet $monotonic = time.monotonic()\nlet $timer = time.timer()",
          },
          {
            kind: "paragraph",
            text: [
              "Use literals for fixed values, single-unit constructors for dynamic values, and named compound constructors when several units need to be combined.",
            ],
          },
          {
            kind: "paragraph",
            text: [
              "Supported duration literal suffixes are ",
              { code: "ns" },
              ", ",
              { code: "us" },
              ", ",
              { code: "ms" },
              ", ",
              { code: "s" },
              ", ",
              { code: "min" },
              ", ",
              { code: "h" },
              ", ",
              { code: "d" },
              ", and ",
              { code: "w" },
              ". Use ",
              { code: "min" },
              " for minutes; ",
              { code: "10m" },
              ", ",
              { code: "1mo" },
              ", and ",
              { code: "1y" },
              " are invalid duration literals.",
            ],
          },
          {
            kind: "code",
            code: "5.seconds() // invalid\n10m         // invalid; use 10min\n1mo         // invalid; use time.period(months: 1)\n1y          // invalid; use time.period(years: 1)",
          },
          {
            kind: "paragraph",
            text: [
              "Numeric literals are not objects in Echo. Duration units must be expressed with duration literals or ",
              { code: "time" },
              " constructors, and months or years must be calendar periods.",
            ],
          },
          {
            kind: "paragraph",
            text: [
              { code: "time.duration(...)" },
              " accepts optional named parameters for ",
              { code: "weeks" },
              ", ",
              { code: "days" },
              ", ",
              { code: "hours" },
              ", ",
              { code: "minutes" },
              ", ",
              { code: "seconds" },
              ", ",
              { code: "milliseconds" },
              ", ",
              { code: "microseconds" },
              ", and ",
              { code: "nanoseconds" },
              "; omitted units default to zero.",
            ],
          },
          {
            kind: "code",
            code: "let $window = time.duration(\n    minutes: 1,\n    seconds: 30,\n)\n\nlet $zero = time.duration()",
          },
        ],
      },
      {
        title: "Instants",
        tags: ["instant", "monotonic", "unix"],
        aliases: ["time.now", "time.monotonic", "unix time"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "time.now()" },
              " returns a wall-clock ",
              { code: "Instant" },
              " for creation times, expirations, event timestamps, and serialization. ",
              { code: "time.monotonic()" },
              " returns a ",
              { code: "MonoInstant" },
              " that is only for elapsed timing.",
            ],
          },
          {
            kind: "code",
            code: 'let $created_at = time.now()\nlet $expires_at = $created_at + 30d\n\nif (time.now() >= $expires_at) {\n    echo "expired"\n}\n\nlet $start = time.monotonic()\nwork()\nlet $elapsed = time.monotonic() - $start',
          },
          {
            kind: "paragraph",
            text: [
              "Subtracting two ",
              { code: "Instant" },
              " values or two ",
              { code: "MonoInstant" },
              " values returns a ",
              { code: "Duration" },
              ". Mixing wall-clock and monotonic instants is invalid.",
            ],
          },
          {
            kind: "code",
            code: "time.now() - time.monotonic() // invalid",
          },
          {
            kind: "paragraph",
            text: [
              { code: "Instant" },
              " does not expose calendar fields such as ",
              { code: "year" },
              " or ",
              { code: "hour" },
              "; those depend on a future timezone-aware ",
              { code: "DateTime" },
              " value.",
            ],
          },
        ],
      },
      {
        title: "Receiver Methods",
        tags: ["facet", "receiver", "method"],
        aliases: ["timer elapsed", "duration total_millis", "instant unix"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Receiver methods operate on existing time values and are defined with ",
              { code: "facet" },
              ". Do not model this behavior as module functions like ",
              { code: "time.elapsed($timer)" },
              ".",
            ],
          },
          {
            kind: "code",
            code: "facet Instant as $instant {\n    pub fn to_unix(): i64 {\n        // seconds since Unix epoch\n    }\n}\n\nlet $unix = time.now().to_unix()\nlet $elapsed = time.timer().elapsed()",
          },
          {
            kind: "paragraph",
            text: [
              "Planned receiver methods include ",
              { code: "$instant.to_unix()" },
              ", ",
              { code: "$duration.total_millis()" },
              ", ",
              { code: "$timer.elapsed()" },
              ", and ",
              { code: "$timer.reset()" },
              ".",
            ],
          },
          {
            kind: "code",
            code: "facet Duration as $duration {\n    pub fn total_millis(): i128 {\n        // total milliseconds\n    }\n\n    pub fn whole_seconds(): i64 {\n        // whole elapsed seconds\n    }\n}\n\nlet $elapsed = time.timer().elapsed()\necho $elapsed.total_millis()",
          },
        ],
      },
      {
        title: "sleep",
        tags: ["sleep", "time", "scheduling"],
        aliases: ["sleep", "delay", "timer"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "sleep(Duration $duration): void" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "sleep()" },
              " pauses the current task for an explicit duration before continuing execution.",
            ],
          },
          {
            kind: "code",
            code: "from std use time\n\ntime.sleep(500ms)\ntime.sleep(time.milliseconds(500))\ntime.sleep(time.duration(seconds: 5))",
          },
          {
            kind: "paragraph",
            text: [
              { code: "time.sleep(500)" },
              " is invalid because the unit is unclear. Use a duration literal like ",
              { code: "500ms" },
              " or a constructor like ",
              { code: "time.milliseconds(500)" },
              ".",
            ],
          },
        ],
      },
      {
        title: "Timer",
        tags: ["timer", "elapsed", "reset", "monotonic"],
        aliases: ["time.timer", "Timer.elapsed", "Timer.reset"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "Timer" },
              " stores a ",
              { code: "MonoInstant" },
              " internally and is the preferred API for measuring elapsed time.",
            ],
          },
          {
            kind: "code",
            code: 'let $timer = time.timer()\n\nrender()\n\nif ($timer.elapsed() > 16ms) {\n    echo "slow frame"\n}\n\nlet $elapsed = $timer.reset()',
          },
          {
            kind: "paragraph",
            text: [
              { code: "$timer.elapsed()" },
              " returns the duration since the timer started. ",
              { code: "$timer.reset()" },
              " returns the elapsed duration and resets the stored start to the current monotonic time.",
            ],
          },
        ],
      },
      {
        title: "Period",
        tags: ["period", "calendar", "duration"],
        aliases: ["calendar period", "months", "years"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "Duration" },
              " is exact elapsed machine time. ",
              { code: "Period" },
              " is calendar-relative human time for months, years, billing cycles, and future date-time movement.",
            ],
          },
          {
            kind: "code",
            code: "let $exactly_24_hours = 1d\nlet $calendar_tomorrow = time.period(days: 1)\nlet $next_month = time.period(months: 1)\nlet $next_year = time.period(years: 1)",
          },
          {
            kind: "paragraph",
            text: [
              "Do not add ",
              { code: "time.months(1)" },
              " or ",
              { code: "time.years(1)" },
              " as duration constructors. Months and years belong to ",
              { code: "time.period(...)" },
              ".",
            ],
          },
          {
            kind: "paragraph",
            text: [
              { code: "time.period(...)" },
              " accepts optional named parameters for ",
              { code: "years" },
              ", ",
              { code: "months" },
              ", ",
              { code: "weeks" },
              ", and ",
              { code: "days" },
              "; omitted units default to zero.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "standard-library-reflect",
    path: "/docs/std/reflect",
    navGroup: "Language",
    category: "Standard Library",
    title: "reflect",
    summary:
      "Inspect available symbols, function signatures, return types, and runtime value types.",
    tags: ["standard library", "stdlib", "std", "reflect", "reflection", "metadata"],
    aliases: ["std.reflect", "introspection", "function metadata"],
    sections: [
      {
        title: "exists",
        tags: ["reflection", "exists", "metadata"],
        aliases: ["symbol exists", "function exists"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "exists(string $name): bool" }],
          },
          {
            kind: "paragraph",
            text: [{ code: "exists()" }, " checks whether a symbol is known to Echo reflection."],
          },
          {
            kind: "code",
            code: 'from std use reflect\n\nlet $name = "time.sleep"\n\nif (reflect.exists($name)) {\n    echo $name . " is available\\n"\n}',
          },
          {
            kind: "paragraph",
            text: [
              "Use this before reporting or calling optional functionality so diagnostics can distinguish a missing symbol from a later runtime failure.",
            ],
          },
        ],
      },
      {
        title: "params",
        tags: ["reflection", "params", "signature", "metadata"],
        aliases: ["function parameters", "parameter metadata"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "params(string $name): string" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "params()" },
              " returns a string description of a function's parameters.",
            ],
          },
          {
            kind: "code",
            code: 'from std use reflect\n\nlet $name = "time.sleep"\n\nif (reflect.exists($name)) {\n    echo $name . " params: " . reflect.params($name) . "\\n"\n}',
          },
          {
            kind: "paragraph",
            text: [
              "Use this for documentation and debugging tools that need to display how a reflected function should be called.",
            ],
          },
        ],
      },
      {
        title: "returnType",
        tags: ["reflection", "return type", "signature", "metadata"],
        aliases: ["function return type", "return metadata"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "returnType(string $name): string" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "returnType()" },
              " returns a string description of a function's return type.",
            ],
          },
          {
            kind: "code",
            code: 'from std use reflect\n\nlet $name = "time.sleep"\n\nif (reflect.exists($name)) {\n    echo $name . " returns " . reflect.returnType($name) . "\\n"\n}',
          },
          {
            kind: "paragraph",
            text: [
              "Use this when a diagnostic, generated reference, or compatibility check needs to explain what a function returns.",
            ],
          },
        ],
      },
      {
        title: "typeOf",
        tags: ["reflection", "type", "value"],
        aliases: ["runtime type", "value type"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "typeOf(mixed $value): string" }],
          },
          {
            kind: "paragraph",
            text: [{ code: "typeOf()" }, " reports the runtime type of a value."],
          },
          {
            kind: "code",
            code: 'from std use reflect\n\nlet $value = "signed:user-42"\necho "Value type: " . reflect.typeOf($value) . "\\n"',
          },
          {
            kind: "paragraph",
            text: [
              "Use this when an example or diagnostic needs to show the type Echo sees at runtime instead of relying on a source-level guess.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "standard-library-assert",
    path: "/docs/std/assert",
    navGroup: "Language",
    category: "Standard Library",
    title: "assert",
    summary: "Fail clearly when a condition or expected value does not hold.",
    tags: ["standard library", "stdlib", "std", "assert", "testing", "validation"],
    aliases: ["std.assert", "assertions", "test helpers"],
    sections: [
      {
        title: "ok",
        tags: ["assert", "testing", "validation", "ok"],
        aliases: ["assert condition", "assert true"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "ok(bool $condition): bool" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "ok()" },
              " asserts that a condition is true and fails clearly when the condition is false.",
            ],
          },
          {
            kind: "code",
            code: 'from std use assert\n\nlet $payload = "signed:user-42"\nassert.ok(str_contains($payload, ":"))',
          },
          {
            kind: "paragraph",
            text: [
              "Use this for invariant checks where the exact value matters less than whether a condition holds before later code depends on it.",
            ],
          },
        ],
      },
      {
        title: "equals",
        tags: ["assert", "testing", "validation", "equals"],
        aliases: ["assert equals", "expected value"],
        blocks: [
          {
            kind: "paragraph",
            text: [{ code: "equals(mixed $actual, mixed $expected): bool" }],
          },
          {
            kind: "paragraph",
            text: [
              { code: "equals()" },
              " asserts that an actual value matches the expected value.",
            ],
          },
          {
            kind: "code",
            code: 'from std use assert\n\nlet $payload = "signed:user-42"\nlet $parts = explode(":", $payload)\n\nassert.equals(count($parts), 2)\nassert.equals($parts[0], "signed")',
          },
          {
            kind: "paragraph",
            text: [
              "Use this when the expected value is concrete and a mismatch should stop the example, fixture, or check at the point of failure.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "php-compatibility",
    path: "/docs/php-compatibility",
    navGroup: "Language",
    category: "Language",
    title: "PHP Compatibility",
    summary:
      "Understand Echo's PHP compatibility floor, supported built-ins, and explicit unsupported behavior.",
    tags: ["php", "compatibility", "builtins", "fixtures", "runtime"],
    aliases: ["php parity", "compatibility inventory", "supported php"],
    sections: [
      {
        title: "Compatibility Floor",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo is a PHP superset. Existing PHP syntax and behavior should remain valid unless Echo explicitly documents an unsupported edge or a stricter Echo-only mode.",
            ],
          },
          {
            kind: "code",
            code: '<?php\n\n$name = "Echo";\necho "Hello, " . $name . "\\n";',
          },
          {
            kind: "paragraph",
            text: [
              "Compatibility work is tracked through fixtures and the PHP built-in inventory. Unsupported behavior should produce a clear diagnostic instead of silently taking a near match.",
            ],
          },
        ],
      },
      {
        title: "Built-ins",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "PHP built-ins live in the compatibility surface. Use the PHP Built-ins pages to see current support, signatures, examples, and semantic notes for search.",
            ],
          },
          {
            kind: "code",
            code: 'let $payload = "signed:user-42"\n\nif (str_contains($payload, ":")) {\n    echo strtoupper($payload) . "\\n"\n}',
          },
          {
            kind: "paragraph",
            text: [
              "This example combines string search and transformation so the built-ins appear in the kind of validation-and-output path users search for.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "examples",
    path: "/docs/examples",
    navGroup: "Language",
    category: "Language",
    title: "Examples",
    summary:
      "Use small Echo and PHP-compatible examples to understand the supported surface and the intended style.",
    tags: ["examples", "echo", "php", "snippets", "standard library"],
    aliases: ["sample code", "program examples", "recipes"],
    sections: [
      {
        title: "PHP-Compatible Program",
        tags: ["php", "run", "compatibility"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Start compatibility examples as normal PHP. This keeps the source valid for PHP while Echo proves the same behavior through its compiler and runtime path.",
            ],
          },
          {
            kind: "code",
            code: '<?php\n\n$payload = "signed:user-42";\n$parts = explode(":", $payload);\n\nif (count($parts) === 2) {\n    echo strtoupper($parts[1]) . "\\n";\n}',
          },
          {
            kind: "paragraph",
            text: [
              "The example validates the input shape before transforming it, so it exercises string search, array output, count, branching, and output in a realistic compatibility path.",
            ],
          },
        ],
      },
      {
        title: "Echo Standard Library",
        tags: ["std", "time", "semantic profiles"],
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo-native examples can use ",
              { code: "from ... use ..." },
              " imports, duration literals, and receiver methods. Std imports bind Echo modules, not PHP namespaces.",
            ],
          },
          {
            kind: "code",
            code: 'from std use time\n\nlet $timer = time.timer()\ntime.sleep(25ms)\n\nlet $elapsed = $timer.elapsed()\necho "elapsed: " . $elapsed.total_millis() . "ms\\n"',
          },
          {
            kind: "paragraph",
            text: [
              "This shows the intended split between module functions that create or access values and receiver methods that operate on existing values.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "semantic-profiles",
    path: "/docs/semantic-profiles",
    navGroup: "Language",
    category: "Language",
    title: "Semantic Profiles",
    summary: "Plan explicit source declarations for stricter Echo semantics without file-extension modes.",
    tags: ["semantics", "strict", "echo", "types", "let"],
    aliases: ["semantic profiles", "modernization profiles", "explicit semantics"],
    sections: [
      {
        title: "Explicit Profiles",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo no longer chooses strictness from file extension. Future modernization policies should be explicit source declarations consumed by semantic analysis, while the base parser remains the shared PHP-compatible Echo superset.",
            ],
          },
          {
            kind: "code",
            code: "module app.orders\n\nsemantics {\n    strict\n}\n\nuse std.time\n\nlet started_at = time.now()\necho started_at.format()",
          },
          {
            kind: "paragraph",
            text: [
              "The profile declaration is Echo syntax. It gives the compiler a place to enforce stronger rules and expose better optimization facts without reviving ",
              { code: "--strict" },
              " flags or extension-driven modes.",
            ],
          },
        ],
      },
      {
        title: "Typed Bindings",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Types are written after the symbol. Prefer inference with ",
              { code: "let" },
              " and add an explicit type when it documents a boundary or constrains an empty literal.",
            ],
          },
          {
            kind: "code",
            code: 'let $users: list<User> = {}\n\n$users.push({\n    id: 1,\n    email: "first@example.test",\n}: User)',
          },
          {
            kind: "paragraph",
            text: [
              "The explicit list type gives the empty literal enough information for later pushes, while the object literal stays readable at the call site.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "imports",
    path: "/docs/imports",
    navGroup: "Language",
    category: "Language",
    title: "Imports",
    summary: "Use PHP namespace imports and Echo-owned from ... use imports without mixing them.",
    tags: ["imports", "use", "from", "std", "package", "namespace"],
    aliases: ["from std use", "vendor imports", "php use"],
    sections: [
      {
        title: "Import Lanes",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Plain ",
              { code: "use" },
              " remains PHP namespace syntax. Echo-owned imports use ",
              { code: "from ... use ..." },
              " for standard library modules, package modules, local Echo modules, and data files.",
            ],
          },
          {
            kind: "code",
            code: 'use Psr\\Log\\LoggerInterface\n\nfrom std use time\nfrom illuminate/http use Request\nfrom "./routes.echo" use route',
          },
          {
            kind: "paragraph",
            text: [
              "The source prefix decides resolution. ",
              { code: "from std use ..." },
              " binds compiler-known standard library modules and never exposes Rust ABI symbols directly. Project-wide import and package lookup belongs in Echo's resolver layer so LSP, semantic analysis, xo, and codegen consume the same resolved symbols.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "compilation-graph",
    path: "/docs/compilation-graph",
    navGroup: "Language",
    category: "Language",
    title: "Compilation Graph",
    summary: "Declare the closed set of files and packages that an Echo program may compile and require.",
    tags: ["compile", "graph", "require", "include", "composer", "packages"],
    sections: [
      {
        title: "Closed Program Boundary",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo compiles a closed graph of source files and packages. Static ",
              { code: "require" },
              " and ",
              { code: "include" },
              " statements add graph edges automatically. When runtime code must choose a file dynamically, a ",
              { code: "compile { ... }" },
              " declaration admits the possible targets before execution.",
            ],
          },
          {
            kind: "code",
            code: 'compile {\n    "./routes/*.php"\n    "./plugins/**/*.php"\n    "modoterra/laravel-echo"\n}\n\nlet $name = $_GET["plugin"] ?? "default"\nrequire_once __DIR__ . "/plugins/" . $name . ".php"',
          },
          {
            kind: "paragraph",
            text: [
              "The dynamic ",
              { code: "require_once" },
              " can only execute a file already admitted by the compile block. If the requested plugin file is outside the graph, Echo reports an error instead of searching the filesystem at runtime.",
            ],
          },
        ],
      },
      {
        title: "Entry Resolution",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "A compile block entry is a compile-time path or package reference. ",
              { code: "\"./relative\"" },
              " resolves from the declaring file's ",
              { code: "__DIR__" },
              "; ",
              { code: "\"/absolute\"" },
              " is a host filesystem path; ",
              { code: "\"name/package\"" },
              " loads that whole package through package metadata.",
            ],
          },
          {
            kind: "code",
            code: 'compile {\n    "./config/*.php"\n    "/srv/app/shared/bootstrap.php"\n    "psr/log"\n}',
          },
          {
            kind: "paragraph",
            text: [
              "This gives Echo a static whole-program boundary without requiring a separate manifest. Composer can still acquire packages and provide compatibility metadata, but compiled Echo programs should not depend on Composer's generated runtime autoload file for discovery.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "command-line",
    path: "/docs/command-line",
    navGroup: "Tooling",
    category: "Tooling",
    title: "Command Line",
    summary: "Use xo to inspect, run, and build Echo-compatible programs.",
    tags: ["cli", "xo", "ast", "ir", "run", "build"],
    aliases: ["xo command", "compiler cli", "run build"],
    sections: [
      {
        title: "Core Commands",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "The ",
              { code: "xo" },
              " command is the current compiler and runtime entrypoint. Use it to inspect parser output, inspect LLVM IR, run a supported program, or build a binary.",
            ],
          },
          {
            kind: "code",
            code: "xo ast examples/hello.php\nxo ir examples/hello.php\nxo run examples/hello.php\nxo build examples/hello.php -o /tmp/hello",
            language: "shellscript",
          },
          {
            kind: "paragraph",
            text: [
              "Use the narrower inspection commands before running or building when a change needs to prove which compiler stage owns the behavior.",
            ],
          },
        ],
      },
      {
        title: "Help",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Use ",
              { code: "--help" },
              " directly or the command-style ",
              { code: "help" },
              " alias. Both forms print the same clap-generated help output.",
            ],
          },
          {
            kind: "code",
            code: "xo --help\nxo help\nxo run --help\nxo help run",
            language: "shellscript",
          },
          {
            kind: "paragraph",
            text: [
              "The alias is useful when exploring commands from memory, while ",
              { code: "--help" },
              " remains the canonical flag form for scripts and standard CLI expectations.",
            ],
          },
        ],
      },
      {
        title: "CLI Behavior",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "The CLI compiles the same language regardless of extension. Use the extension that fits package conventions and stock-PHP expectations, not to select parser or semantic policy.",
            ],
          },
          {
            kind: "code",
            code: "xo run app.php\nxo run app.echo",
            language: "shellscript",
          },
          {
            kind: "paragraph",
            text: [
              "This pairing is useful when comparing source styles. Both commands use the same parser, semantic pipeline, HIR/MIR lowering, LLVM backend, and Rust runtime.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "language-server",
    path: "/docs/language-server",
    navGroup: "Tooling",
    category: "Tooling",
    title: "Language Server",
    summary:
      "Track the Echo language server direction and the shared compiler facts it should consume.",
    tags: ["lsp", "language server", "diagnostics", "semantics", "editor"],
    aliases: ["editor support", "lsp"],
    sections: [
      {
        title: "Current Status",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "The language server is a tooling surface over the shared compiler pipeline. Parser diagnostics, semantic facts, type information, and source spans should come from the same crates used by ",
              { code: "xo" },
              " rather than from an editor-only implementation.",
            ],
          },
          {
            kind: "paragraph",
            text: [
              "Diagnostic codes, severity, primary spans, and related spans belong in ",
              { code: "echo_diagnostics" },
              ". The language server should translate that shared model to LSP diagnostics instead of inventing editor-only categories.",
            ],
          },
        ],
      },
    ],
  },
];

export const bookPages: DocsPage[] = [
  {
    id: "book",
    path: "/book",
    navGroup: "Book",
    category: "The Echo Book",
    title: "The Echo Language",
    summary: "A readable walkthrough of Echo syntax, values, modules, and strict language rules.",
    tags: ["book", "language", "syntax", "echo"],
    aliases: ["echo book", "language book", "syntax guide"],
    sections: [
      {
        title: "What Echo Is",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo is a PHP-compatible language path with a stricter Echo surface layered on top. Existing PHP remains part of the language story, while Echo syntax gives programs explicit modules, typed bindings, closed compilation graphs, structural data, classes, facets, effects, and checked numeric behavior.",
            ],
          },
          {
            kind: "code",
            code: 'module app.orders\n\nsemantics {\n    strict\n}\n\nuse std.time\n\nlet $started_at = time.now()\necho "started {$started_at.format()}"',
          },
          {
            kind: "paragraph",
            text: [
              "The rest of this book walks through the canonical strict Echo style. The reference remains useful when you need a precise rule; the book is meant to be read front to back.",
            ],
          },
        ],
      },
      {
        title: "Chapters",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Start with files and modules, then move into values, types, control flow, and program boundaries. Each chapter introduces the surface syntax through complete examples and explains the design pressure behind the rule.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "book-files-and-modules",
    path: "/book/files-and-modules",
    navGroup: "Book",
    category: "The Echo Book",
    title: "Files And Modules",
    summary: "Name Echo files and modules, order the prelude, and import code canonically.",
    tags: ["book", "module", "imports", "compile", "semantics"],
    aliases: ["modules", "imports", "file order"],
    sections: [
      {
        title: "Module Identity",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo files that are imported by module identity declare a module. Module names use lowercase snake_case segments separated by dots, which gives packages one stable spelling independent of PHP namespace casing.",
            ],
          },
          {
            kind: "code",
            code: "module app.http.router\n\npub fn route($request: Request): Response {\n    return Response.ok()\n}",
          },
          {
            kind: "paragraph",
            text: [
              "Entry scripts may omit a module declaration. Importable package files should include one so the resolver can identify the source without guessing from paths.",
            ],
          },
        ],
      },
      {
        title: "Prelude Order",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "A strict Echo file starts with its declarations about the file itself, then imports, then declarations and executable statements. The canonical order is module, semantics, compile, imports, declarations, and finally executable code.",
            ],
          },
          {
            kind: "code",
            code: 'module app.orders\n\nsemantics {\n    strict\n}\n\ncompile {\n    "./routes/*.php"\n    "modoterra/laravel-echo"\n}\n\nuse std.time\nfrom app.orders use Order\n\nlet $started_at = time.now()',
          },
          {
            kind: "paragraph",
            text: [
              "Keeping the prelude stable makes the top of a file scannable and gives tools a predictable place to find semantic and graph declarations.",
            ],
          },
        ],
      },
      {
        title: "Imports",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Use direct imports for one symbol and grouped imports when several symbols come from the same module. Filesystem paths stay quoted; bare names belong to the module system.",
            ],
          },
          {
            kind: "code",
            code: 'use std.time\nuse illuminate.console.Command\nfrom app.orders use Order, OrderStatus\nfrom "./contracts.echo" use HandlesOrder',
          },
        ],
      },
    ],
  },
  {
    id: "book-bindings-and-types",
    path: "/book/bindings-and-types",
    navGroup: "Book",
    category: "The Echo Book",
    title: "Bindings And Types",
    summary: "Use let, const, primitive types, nullable values, unknown, and safe numeric conversion.",
    tags: ["book", "let", "const", "types", "unknown", "numeric"],
    aliases: ["bindings", "primitive types", "const"],
    sections: [
      {
        title: "Variables",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo keeps PHP's ",
              { code: "$" },
              " sigil for runtime variables. Use ",
              { code: "let" },
              " to introduce a binding and plain assignment to reassign it. Use local ",
              { code: "const" },
              " when the variable must keep pointing at the same value location.",
            ],
          },
          {
            kind: "code",
            code: 'let $count = 0\n$count = $count + 1\n\nconst $config = load_config()\n$config.cache.enabled = true',
          },
          {
            kind: "paragraph",
            text: [
              "Local ",
              { code: "const" },
              " is a binding rule, not a deep freeze. Fields inside the value still follow the value's normal mutation rules.",
            ],
          },
        ],
      },
      {
        title: "Primitive Types",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "The ergonomic numeric defaults are ",
              { code: "int" },
              ", ",
              { code: "uint" },
              ", and ",
              { code: "float" },
              ". They alias ",
              { code: "int64" },
              ", ",
              { code: "uint64" },
              ", and ",
              { code: "float64" },
              ". Use sized types at binary and protocol boundaries.",
            ],
          },
          {
            kind: "code",
            code: "let $ok: bool = true\nlet $name: string = \"Ada\"\nlet $payload: bytes = b'hello'\nlet $port: uint16 = 443\nlet $exact = 340282366920938463463374607431768211456n",
          },
          {
            kind: "paragraph",
            text: [
              "Echo has no ",
              { code: "mixed" },
              ", ",
              { code: "any" },
              ", ",
              { code: "scalar" },
              ", ",
              { code: "resource" },
              ", or broad ",
              { code: "object" },
              " top type. Use concrete unions, interfaces, structural types, generics, errors, or ",
              { code: "unknown" },
              ".",
            ],
          },
        ],
      },
      {
        title: "Null And Unknown",
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "null" },
              " is a literal value, not a type name. Nullable values use ",
              { code: "?T" },
              ". External data that must be checked before use should enter as ",
              { code: "unknown" },
              ".",
            ],
          },
          {
            kind: "code",
            code: "let $user: ?User = null\nlet $value: unknown = json.decode($body)\n\nif $value is UserPayload {\n    save_user($value)\n}",
          },
        ],
      },
      {
        title: "Conversions",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Safe numeric widening is allowed, but narrowing and lossy conversions are explicit. Integers do not silently become floats, and ",
              { code: "string" },
              " and ",
              { code: "bytes" },
              " never implicitly convert.",
            ],
          },
          {
            kind: "code",
            code: "let $small: int32 = 10\nlet $large: int64 = $small\n\nlet $count: int = get_count()\nlet $ratio = float64.from_int($count)\n\nlet $encoded = encoding.utf8.encode(\"Ada\")\nlet $decoded = encoding.utf8.decode($encoded)",
          },
        ],
      },
    ],
  },
  {
    id: "book-data-and-destructuring",
    path: "/book/data-and-destructuring",
    navGroup: "Book",
    category: "The Echo Book",
    title: "Data And Destructuring",
    summary: "Work with lists, arrays, tuples, structural objects, object literals, and destructuring patterns.",
    tags: ["book", "objects", "lists", "arrays", "tuples", "destructuring"],
    aliases: ["data structures", "object literals", "destructuring"],
    sections: [
      {
        title: "Collections",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo gives collection delimiters distinct meanings. ",
              { code: "{}" },
              " creates a list when untyped, ",
              { code: "{ field: value }" },
              " creates a structural object, ",
              { code: "[]" },
              " is an array, and ",
              { code: "()" },
              " is a tuple.",
            ],
          },
          {
            kind: "code",
            code: "let $ids: array<int> = [1, 2, 3]\nlet $fixed: array<int>[3] = [255, 128, 0]\nlet $names: list<string> = {\"Ada\", \"Grace\"}\nlet $pair = (\"Ada\", 36)",
          },
        ],
      },
      {
        title: "Structural Objects",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Structural objects are plain public data. Fields always exist, can be read and assigned with dot access, and cannot be added dynamically.",
            ],
          },
          {
            kind: "code",
            code: "type UserPayload = {\n    name: string\n    nickname: ?string = null\n}\n\nlet $payload: UserPayload = { name: \"Echo\" }\n$payload.name = \"Ada\"",
          },
          {
            kind: "paragraph",
            text: [
              "There are no undefined fields in strict Echo. If a field can lack a real value, model that with ",
              { code: "?T" },
              " and assign ",
              { code: "null" },
              ".",
            ],
          },
        ],
      },
      {
        title: "Destructuring",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Destructuring patterns are separate from declaration keywords. ",
              { code: "let" },
              " declares reassignable bindings, ",
              { code: "const" },
              " declares non-reassignable bindings, and a bare pattern assigns existing variables.",
            ],
          },
          {
            kind: "code",
            code: "let ($user, $posts) = join $tasks\n($user, $posts) = refresh()\n\nlet { $name, $email } = $user\nlet { name: $display_name } = $user\n{ $name, $email } = refresh_user()",
          },
          {
            kind: "paragraph",
            text: [
              "Tuple destructuring requires exact arity. Object destructuring is partial by default. Patterns bind or assign variables only, not fields, properties, or indexes.",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "book-classes-facets-and-enums",
    path: "/book/classes-facets-and-enums",
    navGroup: "Book",
    category: "The Echo Book",
    title: "Classes, Facets, And Enums",
    summary: "Use classes for identity and encapsulation, facets for receiver methods, and enums for variants.",
    tags: ["book", "classes", "facets", "enums", "factory"],
    aliases: ["classes", "facets", "enums"],
    sections: [
      {
        title: "Classes",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Classes are for identity, private state, and instance behavior. They use ",
              { code: "$this" },
              " and ",
              { code: "->" },
              " for instance access. Construction goes through public factories, not ",
              { code: "new" },
              ".",
            ],
          },
          {
            kind: "code",
            code: "pub class User {\n    pub $name: string\n    $email: string\n\n    factory {\n        pub create($name: string, $email: string) {\n            $this->name = $name\n            $this->email = $email\n        }\n    }\n\n    pub fn rename($name: string): void {\n        $this->name = $name\n    }\n}",
          },
          {
            kind: "paragraph",
            text: [
              { code: "pub" },
              " is the visibility keyword for classes, properties, methods, factories, constants, types, enums, traits, interfaces, and facet methods. Private is the default.",
            ],
          },
        ],
      },
      {
        title: "Facets",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "A facet defines receiver methods for a type or object value surface. It does not add class instance methods and does not use ",
              { code: "->" },
              ". The receiver alias is declared once with ",
              { code: "as $alias" },
              ".",
            ],
          },
          {
            kind: "code",
            code: "facet UserPayload as $payload {\n    pub fn display_name(): string {\n        return $payload.name\n    }\n}\n\nfacet int as $n {\n    pub fn label(): string {\n        return $n.as_str()\n    }\n}",
          },
          {
            kind: "paragraph",
            text: [
              "Public facet methods admitted into the closed compilation graph are globally visible for receiver lookup. Duplicate target type and method names fail compilation.",
            ],
          },
        ],
      },
      {
        title: "Enums",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Enums name a closed set of variants. Cases use PascalCase and dot access. Backed cases are checked against the enum backing type; payload cases carry typed data.",
            ],
          },
          {
            kind: "code",
            code: "pub enum OrderStatus: string {\n    Pending = \"pending\"\n    Paid = \"paid\"\n}\n\npub enum Result<T, E> {\n    Ok(T)\n    Err(E)\n}",
          },
          {
            kind: "paragraph",
            text: [
              "Enum bodies declare cases only. Add behavior with facets and branch with exhaustive ",
              { code: "match" },
              ".",
            ],
          },
        ],
      },
    ],
  },
  {
    id: "book-control-flow-and-effects",
    path: "/book/control-flow-and-effects",
    navGroup: "Book",
    category: "The Echo Book",
    title: "Control Flow And Effects",
    summary: "Write strict Echo conditions, matches, loops, effects, and concurrent work.",
    tags: ["book", "if", "match", "loop", "effect", "concurrency"],
    aliases: ["control flow", "effects", "loop"],
    sections: [
      {
        title: "Conditions And Match",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Strict Echo conditions do not use PHP-style parentheses. Boolean operators are words, and ",
              { code: "match" },
              " is the expression form for multi-branch value selection.",
            ],
          },
          {
            kind: "code",
            code: "if not $user.active or $user.locked {\n    return false\n}\n\nlet $message = match $result {\n    Result.Ok($user) => \"Saved {$user.name}\",\n    Result.Err($error) => $error.message\n}",
          },
        ],
      },
      {
        title: "Loops",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "The strict Echo loop construct is ",
              { code: "loop" },
              ". It can run forever, iterate over values, and produce a value through ",
              { code: "break value" },
              ".",
            ],
          },
          {
            kind: "code",
            code: "let $found = loop $users as $user {\n    if $user.id == $target_id {\n        break $user\n    }\n}: ?User",
          },
        ],
      },
      {
        title: "Effects And Concurrency",
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "effect" },
              " is a direct-style expression for effectful code. Concurrent work uses ",
              { code: "defer" },
              ", ",
              { code: "run" },
              ", ",
              { code: "fork" },
              ", ",
              { code: "spawn" },
              ", and ",
              { code: "join" },
              ".",
            ],
          },
          {
            kind: "code",
            code: "let $tasks = run {\n    fetch_user($id),\n    fetch_posts($id)\n}\n\nlet ($user, $posts) = join $tasks",
          },
        ],
      },
    ],
  },
  {
    id: "book-strings-bytes-and-numbers",
    path: "/book/strings-bytes-and-numbers",
    navGroup: "Book",
    category: "The Echo Book",
    title: "Strings, Bytes, And Numbers",
    summary: "Understand interpolation, raw strings, byte literals, checked arithmetic, and delete.",
    tags: ["book", "strings", "bytes", "numbers", "delete"],
    aliases: ["strings", "bytes", "numbers"],
    sections: [
      {
        title: "Strings And Bytes",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Double-quoted strings interpolate. Single-quoted strings are raw text. Interpolation accepts normal Echo expressions, but each expression must produce ",
              { code: "string" },
              "; Echo does not format values implicitly.",
            ],
          },
          {
            kind: "code",
            code: "let $message = \"Count {$count.as_str()}\"\nlet $template = 'Hello {$name}'\nlet $bytes = b'hello'\nlet $fire = x'f09f94a5'",
          },
          {
            kind: "paragraph",
            text: [
              { code: "b'...'" },
              " creates UTF-8 bytes from raw text. ",
              { code: "x'...'" },
              " creates exact bytes from static hex pairs. Use encoding APIs for dynamic conversion.",
            ],
          },
        ],
      },
      {
        title: "Operators",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Integer arithmetic is checked by default. Assignment, compound assignment, increment, and decrement are statements, not expressions.",
            ],
          },
          {
            kind: "code",
            code: "$count++\n$count += 1\nlet $whole = 5 // 2\nlet $ratio = 5 / 2\nlet $huge = 2n ** 256",
          },
          {
            kind: "paragraph",
            text: [
              "Use word boolean operators for logic and symbolic operators for bitwise work. There is no relaxed equality in strict Echo; ",
              { code: "==" },
              " is strict value equality, and ",
              { code: "is same" },
              " checks identity.",
            ],
          },
        ],
      },
      {
        title: "Delete",
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "delete" },
              " removes entries from primitive deletable containers and returns whether removal happened. It does not delete variables, fields, properties, or memory.",
            ],
          },
          {
            kind: "code",
            code: "let $removed = delete $users[2]\n\n$items.append($item)\n$dict.remove($key)",
          },
        ],
      },
    ],
  },
  {
    id: "book-errors-and-programs",
    path: "/book/errors-and-programs",
    navGroup: "Book",
    category: "The Echo Book",
    title: "Errors And Programs",
    summary: "Declare errors, recover from panics, and define the closed compilation graph.",
    tags: ["book", "errors", "panic", "recover", "compile"],
    aliases: ["errors", "compile graph", "programs"],
    sections: [
      {
        title: "Errors",
        blocks: [
          {
            kind: "paragraph",
            text: [
              { code: "error" },
              " declares a nominal failure type. Construct errors like normal type objects, then use ",
              { code: "panic" },
              " to raise them and ",
              { code: "recover" },
              " to handle them.",
            ],
          },
          {
            kind: "code",
            code: "pub error FileNotFound {\n    path: string\n    message: string = \"file not found\"\n}\n\nlet $result = try {\n    open_file($path)\n} recover {\n    FileNotFound as $err => fallback_file()\n} ensure {\n    close_handles()\n}",
          },
        ],
      },
      {
        title: "Closed Programs",
        blocks: [
          {
            kind: "paragraph",
            text: [
              "Echo compiles a closed graph. Static includes add edges automatically; dynamic includes may execute only files admitted by the graph. Use ",
              { code: "compile { ... }" },
              " for known dynamic targets and packages.",
            ],
          },
          {
            kind: "code",
            code: 'compile {\n    "./routes/*.php"\n    "/srv/app/shared/bootstrap.php"\n    "modoterra/laravel-echo"\n}',
          },
        ],
      },
    ],
  },
];

export const contentPages = [...docsPages, ...bookPages];

export const docsPageByPath = new Map(contentPages.map((page) => [page.path, page]));

export const docsNavigation: DocsNavGroup[] = [
  {
    title: "Getting Started",
    links: [
      { label: "Installation", to: "/docs" },
      { label: "Quickstart", to: "/docs/quickstart" },
      { label: "Single Language Mode", to: "/docs/single-language-mode" },
      { label: "Roadmap", to: "/docs/roadmap" },
    ],
  },
  {
    title: "Language",
    links: [
      {
        label: "Data Structures",
        to: "/docs/data-structures",
        children: [
          { label: "List", to: "/docs/data-structures/list" },
          { label: "Object", to: "/docs/data-structures/object" },
          { label: "Class", to: "/docs/data-structures/class" },
          { label: "Array", to: "/docs/data-structures/array" },
          { label: "Enum", to: "/docs/data-structures/enum" },
        ],
      },
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
        to: "/docs/std",
        children: [
          { label: "net", to: "/docs/std/net" },
          { label: "http", to: "/docs/std/http" },
          { label: "time", to: "/docs/std/time" },
          { label: "reflect", to: "/docs/std/reflect" },
          { label: "assert", to: "/docs/std/assert" },
        ],
      },
      { label: "PHP Compatibility", to: "/docs/php-compatibility" },
      { label: "Examples", to: "/docs/examples" },
      { label: "Semantic Profiles", to: "/docs/semantic-profiles" },
      { label: "Imports", to: "/docs/imports" },
      { label: "Compilation Graph", to: "/docs/compilation-graph" },
    ],
  },
  {
    title: "Tools",
    links: [
      { label: "Command Line", to: "/docs/command-line" },
      { label: "Language Server", to: "/docs/language-server" },
    ],
  },
];

export const bookNavigation: DocsNavGroup[] = [
  {
    title: "The Echo Book",
    links: [
      { label: "The Echo Language", to: "/book" },
      { label: "Files And Modules", to: "/book/files-and-modules" },
      { label: "Bindings And Types", to: "/book/bindings-and-types" },
      { label: "Data And Destructuring", to: "/book/data-and-destructuring" },
      { label: "Classes, Facets, And Enums", to: "/book/classes-facets-and-enums" },
      { label: "Control Flow And Effects", to: "/book/control-flow-and-effects" },
      { label: "Strings, Bytes, And Numbers", to: "/book/strings-bytes-and-numbers" },
      { label: "Errors And Programs", to: "/book/errors-and-programs" },
    ],
  },
];
