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
        name: "str_repeat",
        signature: "str_repeat(string $string, int $times): string",
        description: "Repeats a string a fixed number of times.",
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
    ],
  },
  {
    slug: "arrays",
    title: "Arrays",
    description: "Array functions count values and inspect whether arrays behave like lists.",
    builtins: [
      {
        name: "array_is_list",
        signature: "array_is_list(array $array): bool",
        description:
          "Returns true when an array has consecutive integer keys starting at zero. Empty arrays are lists. Associative keys or gaps in numeric keys make an array stop being a list.",
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
    "rmdir",
    "Use `rmdir()` when cleanup should remove only an empty directory, leaving non-empty directories intact so accidental recursive deletion is avoided.",
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
  return (
    builtinExampleNotes.get(builtin.name) ??
    `This example puts ${builtin.name} in the middle of a small workflow so the return value is immediately used instead of being printed as an isolated probe.`
  );
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
