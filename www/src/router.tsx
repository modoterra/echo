import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
  useLocation,
} from "@tanstack/react-router";
import { RiCheckLine, RiFileCopyLine } from "@remixicon/react";
import { layout, prepare } from "@chenglou/pretext";
import { AnimatePresence, motion } from "motion/react";
import ShikiHighlighter, {
  createHighlighterCore,
  createJavaScriptRegexEngine,
} from "react-shiki/core";
import {
  createContext,
  useContext,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { HomePage } from "./app";

type DocsNavGroup = {
  title: string;
  links: DocsNavLink[];
};

type DocsNavLink = {
  label: string;
  to: string;
  disabled?: boolean;
  children?: DocsNavLink[];
};

type DocsShellProps = {
  category: string;
  title: string;
  headings: string[];
  children: ReactNode;
};

type DocsPageMeta = Omit<DocsShellProps, "children">;

type DocsLayoutContextValue = {
  setMeta: (meta: DocsPageMeta) => void;
};

const defaultDocsPageMeta: DocsPageMeta = {
  category: "Getting Started",
  headings: [
    "Meet Echo",
    "Installation",
    "Run a Program",
    "Compile a Program",
    "Project Status",
  ],
  title: "Installation",
};

const DocsLayoutContext = createContext<DocsLayoutContextValue | null>(null);

type BuiltinDoc = {
  name: string;
  signature: string;
  description: string;
};

type BuiltinFamily = {
  slug: string;
  title: string;
  description: string;
  builtins: BuiltinDoc[];
};

const builtinFamilies: BuiltinFamily[] = [
  {
    slug: "strings",
    title: "Strings",
    description:
      "String functions inspect, transform, encode, split, search, and compare text.",
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
        description:
          "Returns the string with alphabetic characters converted to uppercase.",
      },
      {
        name: "strtolower",
        signature: "strtolower(string $string): string",
        description:
          "Returns the string with alphabetic characters converted to lowercase.",
      },
      {
        name: "ucwords",
        signature: "ucwords(string $string): string",
        description:
          "Uppercases the first character of each word in the string.",
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
        description:
          "Returns the byte value of the first character in the string.",
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
        description:
          "Escapes characters that need backslashes in quoted PHP strings.",
      },
      {
        name: "stripslashes",
        signature: "stripslashes(string $string): string",
        description: "Unquotes a string quoted with backslashes.",
      },
      {
        name: "quotemeta",
        signature: "quotemeta(string $string): string",
        description:
          "Quotes regular expression metacharacters with backslashes.",
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
        description:
          "Finds the first occurrence of a string case-insensitively.",
      },
      {
        name: "strrpos",
        signature: "strrpos(string $haystack, string $needle): int|false",
        description: "Finds the last occurrence of a string.",
      },
      {
        name: "strripos",
        signature: "strripos(string $haystack, string $needle): int|false",
        description:
          "Finds the last occurrence of a string case-insensitively.",
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
        description:
          "Finds the last occurrence of a character and returns the matching tail.",
      },
      {
        name: "strpbrk",
        signature: "strpbrk(string $string, string $characters): string|false",
        description:
          "Searches a string for any character from a character set.",
      },
      {
        name: "strspn",
        signature: "strspn(string $string, string $characters): int",
        description:
          "Counts the initial span containing only characters from a character set.",
      },
      {
        name: "strcspn",
        signature: "strcspn(string $string, string $characters): int",
        description:
          "Counts the initial span containing no characters from a character set.",
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
        signature:
          "strncmp(string $string1, string $string2, int $length): int",
        description: "Binary-safe string comparison up to a fixed length.",
      },
      {
        name: "strncasecmp",
        signature:
          "strncasecmp(string $string1, string $string2, int $length): int",
        description:
          "Binary-safe case-insensitive string comparison up to a fixed length.",
      },
      {
        name: "explode",
        signature:
          "explode(string $separator, string $string, int $limit): array",
        description: "Splits a string into an array using a separator.",
      },
    ],
  },
  {
    slug: "arrays",
    title: "Arrays",
    description:
      "Array functions count values and inspect whether arrays behave like lists.",
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
        description:
          "Counts the number of elements in an array or countable value.",
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
    description:
      "Type functions inspect current values and convert them to scalar forms.",
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
        description:
          "Returns true when a value is numeric or a numeric string.",
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
    description:
      "Numeric functions calculate simple values and convert integers to base strings.",
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
    slug: "filesystem",
    title: "Filesystem",
    description:
      "Filesystem functions inspect local paths and derive path components.",
    builtins: [
      {
        name: "file_exists",
        signature: "file_exists(string $filename): bool",
        description: "Returns true when a local path exists.",
      },
      {
        name: "is_dir",
        signature: "is_dir(string $filename): bool",
        description:
          "Returns true when a local path exists and is a directory.",
      },
      {
        name: "is_file",
        signature: "is_file(string $filename): bool",
        description:
          "Returns true when a local path exists and is a regular file.",
      },
      {
        name: "is_link",
        signature: "is_link(string $filename): bool",
        description:
          "Returns true when a local path exists and is a symbolic link.",
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
    ],
  },
  {
    slug: "reflection",
    title: "Reflection",
    description:
      "Reflection functions ask questions about functions and callable values.",
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
    description:
      "Shell functions quote or escape strings for command-line usage.",
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
    description:
      "Output buffering functions control PHP-style buffered output.",
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
        description:
          "Gets the active output buffer contents and closes the buffer.",
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
        description:
          "Enables or disables implicit flushing after output calls.",
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

const builtinExamples = new Map<string, string>([
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

const builtinExampleNotes = new Map<string, string>([
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
]);

const builtinFamilyBySlug = new Map(
  builtinFamilies.map((family) => [family.slug, family]),
);

function builtinExample(name: string) {
  const example = builtinExamples.get(name);

  if (!example) {
    throw new Error(`Missing documentation example for PHP builtin: ${name}`);
  }

  return example;
}

function builtinExampleNote(builtin: BuiltinDoc) {
  return (
    builtinExampleNotes.get(builtin.name) ??
    `This example puts ${builtin.name} in the middle of a small workflow so the return value is immediately used instead of being printed as an isolated probe.`
  );
}

function headingId(heading: string) {
  return heading.toLowerCase().replaceAll(" ", "-");
}

let phpHighlighterPromise: Promise<PhpHighlighter> | null = null;

type PhpHighlighter = Awaited<ReturnType<typeof createHighlighterCore>>;

const codeSnippetFont = '14px "Geist Mono"';
const codeSnippetLineHeight = 28;
const codeSnippetBlockPadding = 48;
const codeSnippetSkeletonMinDelay = 320;
const codeSnippetSkeletonMaxDelay = 680;
const codeSnippetLoadRootMargin = "0px";

function loadPhpHighlighter() {
  phpHighlighterPromise ??= Promise.all([
    import("@shikijs/langs/php"),
    import("@shikijs/themes/github-dark"),
  ]).then(([php, githubDark]) =>
    createHighlighterCore({
      engine: createJavaScriptRegexEngine({ forgiving: true }),
      langs: [php.default],
      themes: [githubDark.default],
    }),
  );

  return phpHighlighterPromise;
}

function randomCodeSnippetSkeletonDelay() {
  const range = codeSnippetSkeletonMaxDelay - codeSnippetSkeletonMinDelay;

  return codeSnippetSkeletonMinDelay + Math.round(Math.random() * range);
}

function CodeSnippet({
  children,
  className = "mt-8",
}: {
  children: string;
  className?: string;
}) {
  const snippetRef = useRef<HTMLDivElement | null>(null);
  const [copied, setCopied] = useState(false);
  const [highlighter, setHighlighter] = useState<PhpHighlighter | null>(null);
  const [shouldLoadHighlighter, setShouldLoadHighlighter] = useState(false);
  const code = children.trim();
  const minHeight = useMemo(() => {
    const prepared = prepare(code, codeSnippetFont, { whiteSpace: "pre-wrap" });
    const measured = layout(prepared, 100_000, codeSnippetLineHeight);

    return measured.height + codeSnippetBlockPadding;
  }, [code]);

  useEffect(() => {
    const snippet = snippetRef.current;

    if (!snippet) {
      return;
    }

    if (!("IntersectionObserver" in window)) {
      setShouldLoadHighlighter(true);
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry?.isIntersecting) {
          setShouldLoadHighlighter(true);
          observer.disconnect();
        }
      },
      { rootMargin: codeSnippetLoadRootMargin },
    );

    observer.observe(snippet);

    return () => {
      observer.disconnect();
    };
  }, []);

  useEffect(() => {
    if (!shouldLoadHighlighter) {
      return;
    }

    let active = true;
    let delayTimeout: number | undefined;
    const delay = new Promise((resolve) => {
      delayTimeout = window.setTimeout(resolve, randomCodeSnippetSkeletonDelay());
    });

    void Promise.all([loadPhpHighlighter(), delay]).then(([loadedHighlighter]) => {
      if (active) {
        setHighlighter(loadedHighlighter);
      }
    });

    return () => {
      active = false;
      window.clearTimeout(delayTimeout);
    };
  }, [shouldLoadHighlighter]);

  async function copyCode() {
    await navigator.clipboard.writeText(code);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1400);
  }

  return (
    <div
      ref={snippetRef}
      className={`${className} group relative rounded-lg bg-[#101218] shadow-sm`}
      style={{ minHeight }}
    >
      <button
        aria-label={copied ? "Copied code" : "Copy code"}
        className="absolute right-3 top-3 z-10 inline-flex size-8 select-none items-center justify-center rounded-md text-slate-400 opacity-70 transition hover:bg-white/10 hover:text-slate-100 hover:opacity-100 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-slate-400"
        onClick={copyCode}
        title={copied ? "Copied" : "Copy"}
        type="button"
      >
        {copied ? <RiCheckLine size={18} /> : <RiFileCopyLine size={18} />}
      </button>
      <AnimatePresence initial={false} mode="wait">
        {highlighter ? (
          <motion.div
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -4 }}
            initial={{ opacity: 0, y: 4 }}
            key="highlighted"
            transition={{ duration: 0.22, ease: "easeOut" }}
          >
            <ShikiHighlighter
              addDefaultStyles={false}
              className="docs-code-snippet overflow-x-auto rounded-lg p-6 pr-14 font-mono text-sm leading-7 scrollbar-thin scrollbar-nice-dark"
              highlighter={highlighter}
              language="php"
              showLanguage={false}
              showLineNumbers
              theme="github-dark"
            >
              {code}
            </ShikiHighlighter>
          </motion.div>
        ) : (
          <motion.div
            animate={{ opacity: 1, y: 0 }}
            aria-label="Loading highlighted code"
            className="docs-code-skeleton p-6 pr-14"
            exit={{ opacity: 0, y: -4 }}
            initial={{ opacity: 0, y: 4 }}
            key="skeleton"
            transition={{ duration: 0.18, ease: "easeOut" }}
          >
            {code.split("\n").map((line, index) => (
              <div className="flex h-7 items-center gap-5" key={index}>
                <span className="h-3 w-5 shrink-0 rounded-full bg-slate-700/45" />
                <span
                  className="h-3 rounded-full bg-slate-700/55"
                  style={{
                    width: `${Math.min(92, Math.max(24, line.length * 1.8))}%`,
                  }}
                />
              </div>
            ))}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

function Topbar() {
  return (
    <header className="fixed inset-x-0 top-0 z-30 border-b border-slate-200/70 bg-white/85 px-6 backdrop-blur shadow-2xs">
      <div className="mx-auto grid h-20 w-full max-w-7xl grid-cols-[auto_1fr] items-center gap-10 lg:grid-cols-[220px_minmax(0,720px)] lg:gap-12 xl:grid-cols-[220px_minmax(0,720px)_220px]">
        <Link
          aria-label="Echo home"
          className="block w-16 transition opacity-90 hover:opacity-100 lg:w-20"
          to="/"
        >
          <img alt="Echo" className="h-8 w-full" src="/logo.svg" />
        </Link>
        <nav
          aria-label="Primary navigation"
          className="flex translate-x-0.5 items-center justify-start gap-8 text-sm font-semibold text-slate-500 lg:translate-x-0.5"
        >
          <Link className="transition hover:text-slate-950" to="/">
            Home
          </Link>
          <Link className="transition hover:text-slate-950" to="/docs">
            Docs
          </Link>
        </nav>
      </div>
    </header>
  );
}

function RootLayout() {
  return (
    <>
      <Topbar />
      <Outlet />
    </>
  );
}

function DocsNavLinkItem({
  link,
  pathname,
}: {
  link: DocsNavLink;
  pathname: string;
}) {
  const isActive = pathname === link.to;
  const hasActiveChild = link.children?.some((child) => pathname === child.to);
  const activeChildIndex = link.children?.findIndex((child) => pathname === child.to) ?? -1;
  const shouldShowChildren = Boolean(link.children && (isActive || hasActiveChild));
  const textClass = link.disabled
    ? "text-sm leading-6 text-slate-300"
    : isActive
      ? "text-sm font-semibold leading-6 text-slate-950"
      : "text-sm leading-6 text-slate-500 transition hover:text-slate-950";

  return (
    <li>
      {link.disabled ? (
        <span className={textClass}>{link.label}</span>
      ) : (
        <Link className={textClass} to={link.to}>
          {link.label}
        </Link>
      )}
      <AnimatePresence initial={false}>
        {shouldShowChildren ? (
          <motion.div
            animate={{ height: "auto", opacity: 1 }}
            className="overflow-hidden"
            exit={{ height: 0, opacity: 0 }}
            initial={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2, ease: "easeOut" }}
          >
            <div className="relative mt-3 pl-3">
              <span
                aria-hidden="true"
                className="absolute bottom-0 left-0 top-0 w-[3px] bg-slate-200"
              />
              {activeChildIndex >= 0 ? (
                <span
                  aria-hidden="true"
                  className="docs-primary-nav-train absolute left-0 top-[3px] h-[18px] w-[3px] rounded-full bg-orange-400 transition-transform duration-200 ease-out"
                  style={{
                    transform: `translateY(${activeChildIndex * 36}px)`,
                  }}
                />
              ) : null}
              <ul className="space-y-3">
                {link.children?.map((child) => (
                  <DocsNavLinkItem
                    key={child.label}
                    link={child}
                    pathname={pathname}
                  />
                ))}
              </ul>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </li>
  );
}

function DocsShell({ category, title, headings, children }: DocsShellProps) {
  const docsLayout = useContext(DocsLayoutContext);

  useLayoutEffect(() => {
    docsLayout?.setMeta({ category, headings, title });
  }, [category, docsLayout, headings, title]);

  return <>{children}</>;
}

function DocsLayout() {
  const location = useLocation();
  const [meta, setMeta] = useState<DocsPageMeta>(defaultDocsPageMeta);
  const docsLayoutContext = useMemo(() => ({ setMeta }), []);
  const { category, headings, title } = meta;
  const [activeHeading, setActiveHeading] = useState(headings[0] ?? "");
  const onThisPageRailRef = useRef<HTMLDivElement | null>(null);
  const onThisPageItemRefs = useRef<Record<string, HTMLLIElement | null>>({});
  const [onThisPageTrainY, setOnThisPageTrainY] = useState(0);
  const navigation: DocsNavGroup[] = [
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
          children: [
            { label: "Strings", to: "/docs/php-built-ins/strings" },
            { label: "Arrays", to: "/docs/php-built-ins/arrays" },
            { label: "Types", to: "/docs/php-built-ins/types" },
            { label: "Math and Bases", to: "/docs/php-built-ins/math" },
            { label: "Filesystem", to: "/docs/php-built-ins/filesystem" },
            { label: "Reflection", to: "/docs/php-built-ins/reflection" },
            { label: "Shell", to: "/docs/php-built-ins/shell" },
            {
              label: "Output Buffering",
              to: "/docs/php-built-ins/output-buffering",
            },
            { label: "Core", to: "/docs/php-built-ins/core" },
          ],
        },
        {
          label: "PHP Compatibility",
          to: "/docs/php-compatibility",
          disabled: true,
        },
        { label: "Strict Mode", to: "/docs/strict-mode", disabled: true },
        { label: "Imports", to: "/docs/imports", disabled: true },
      ],
    },
    {
      title: "Tooling",
      links: [
        { label: "Command Line", to: "/docs/command-line", disabled: true },
        {
          label: "Language Server",
          to: "/docs/language-server",
          disabled: true,
        },
        { label: "Testing", to: "/docs/testing", disabled: true },
        { label: "Source Builds", to: "/docs/source-builds" },
      ],
    },
  ];

  useEffect(() => {
    let animationFrame = 0;

    function updateActiveHeading() {
      const nextActiveHeading =
        headings.findLast((heading) => {
          const element = document.getElementById(headingId(heading));

          return element ? element.getBoundingClientRect().top <= 160 : false;
        }) ??
        headings.find((heading) => document.getElementById(headingId(heading))) ??
        headings[0] ??
        "";

      setActiveHeading(nextActiveHeading);
    }

    function scheduleUpdate() {
      window.cancelAnimationFrame(animationFrame);
      animationFrame = window.requestAnimationFrame(updateActiveHeading);
    }

    setActiveHeading(headings[0] ?? "");
    scheduleUpdate();
    window.addEventListener("scroll", scheduleUpdate, { passive: true });
    window.addEventListener("resize", scheduleUpdate);

    return () => {
      window.cancelAnimationFrame(animationFrame);
      window.removeEventListener("scroll", scheduleUpdate);
      window.removeEventListener("resize", scheduleUpdate);
    };
  }, [headings]);

  useLayoutEffect(() => {
    let animationFrame = 0;

    function updateTrainPosition() {
      const rail = onThisPageRailRef.current;
      const item = onThisPageItemRefs.current[activeHeading];

      if (!rail || !item) {
        setOnThisPageTrainY(0);
        return;
      }

      const railRect = rail.getBoundingClientRect();
      const itemRect = item.getBoundingClientRect();
      setOnThisPageTrainY(itemRect.top - railRect.top + itemRect.height / 2 - 9);
    }

    function scheduleUpdate() {
      window.cancelAnimationFrame(animationFrame);
      animationFrame = window.requestAnimationFrame(updateTrainPosition);
    }

    scheduleUpdate();
    window.addEventListener("resize", scheduleUpdate);

    return () => {
      window.cancelAnimationFrame(animationFrame);
      window.removeEventListener("resize", scheduleUpdate);
    };
  }, [activeHeading, headings]);

  function scrollToHeading(heading: string) {
    const id = headingId(heading);
    const element = document.getElementById(id);

    if (!element) {
      return;
    }

    setActiveHeading(heading);
    window.history.pushState(null, "", `#${id}`);
    element.scrollIntoView({ behavior: "smooth", block: "start" });
  }

  return (
    <main className="min-h-screen bg-white px-6 pb-24 pt-32 text-slate-950">
      <div className="mx-auto grid w-full max-w-7xl grid-cols-1 gap-12 lg:grid-cols-[220px_minmax(0,720px)] xl:grid-cols-[220px_minmax(0,720px)_220px]">
        <aside className="hidden lg:block">
          <nav
            aria-label="Documentation sections"
            className="sticky top-32 space-y-10"
          >
            {navigation.map((group) => (
              <section key={group.title}>
                <h2 className="text-sm font-semibold text-slate-950">
                  {group.title}
                </h2>
                <ul className="mt-5 space-y-3">
                  {group.links.map((link) => (
                    <DocsNavLinkItem
                      key={link.label}
                      link={link}
                      pathname={location.pathname}
                    />
                  ))}
                </ul>
              </section>
            ))}
          </nav>
        </aside>

        <DocsLayoutContext.Provider value={docsLayoutContext}>
          <article className="max-w-none">
            <p className="text-sm font-semibold text-slate-500">{category}</p>
            <h1 className="mt-6 text-5xl font-semibold tracking-normal text-slate-950">
              {title}
            </h1>
            <Outlet />
          </article>
        </DocsLayoutContext.Provider>

        <aside className="hidden xl:block">
          <nav
            aria-label="On this page"
            className="sticky top-32"
          >
            <h2 className="text-xs font-semibold uppercase tracking-wide text-slate-400">
              On this page
            </h2>
            <div className="relative mt-5 pl-6" ref={onThisPageRailRef}>
              <span
                aria-hidden="true"
                className="absolute bottom-0 left-0 top-0 w-px bg-slate-200"
              />
              <motion.span
                aria-hidden="true"
                animate={{ y: onThisPageTrainY }}
                className="absolute left-[-1px] top-0 h-[18px] w-[3px] rounded-full bg-orange-400"
                transition={{ duration: 0.22, ease: "easeOut" }}
              />
              <ul className="docs-on-this-page-links space-y-3">
                {headings.map((heading) => (
                  <li
                    key={heading}
                    ref={(element) => {
                      onThisPageItemRefs.current[heading] = element;
                    }}
                  >
                    <a
                      className={
                        activeHeading === heading
                          ? "text-sm font-semibold leading-6 text-slate-950 transition"
                          : "text-sm leading-6 text-slate-500 transition hover:text-slate-950"
                      }
                      href={`#${headingId(heading)}`}
                      onClick={(event) => {
                        if (
                          event.altKey ||
                          event.ctrlKey ||
                          event.metaKey ||
                          event.shiftKey
                        ) {
                          return;
                        }

                        event.preventDefault();
                        scrollToHeading(heading);
                      }}
                    >
                      {heading}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          </nav>
        </aside>
      </div>
    </main>
  );
}

function DocsPage() {
  return (
    <DocsShell
      category="Getting Started"
      headings={[
        "Meet Echo",
        "Installation",
        "Run a Program",
        "Compile a Program",
        "Project Status",
      ]}
      title="Installation"
    >
      <section id="meet-echo" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">
          Meet Echo
        </h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Echo is a Rust implementation of a PHP superset. Existing PHP should
          stay familiar, while Echo adds compiler tooling, native concurrency,
          parallel execution, and a path toward compiled binaries with
          predictable performance gains.
        </p>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          The command line entrypoint is{" "}
          <code className="font-mono text-slate-950">xo</code>. Echo is
          early-stage software, so unsupported PHP behavior should fail
          explicitly rather than silently approximate semantics.
        </p>
      </section>

      <section id="installation" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">
          Installation
        </h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Install the <code className="font-mono text-slate-950">xo</code>{" "}
          command and keep it on your path. The public installer flow is still
          being designed, so current releases are source-built by contributors.
        </p>
        <CodeSnippet>{`xo --help
xo run app.php
xo build app.php -o app`}</CodeSnippet>
      </section>

      <section id="run-a-program" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">
          Run a Program
        </h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Use <code className="font-mono text-slate-950">xo run</code> to
          execute an Echo-compatible PHP file directly from the command line.
        </p>
        <CodeSnippet>{`xo run examples/hello.php`}</CodeSnippet>
      </section>

      <section id="compile-a-program" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">
          Compile a Program
        </h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Use <code className="font-mono text-slate-950">xo build</code> to
          compile a supported program into a native binary. The current backend
          lowers through LLVM IR and links through the project build path while
          Echo's native toolchain matures.
        </p>
        <CodeSnippet>{`xo build examples/hello.php -o /tmp/hello
/tmp/hello`}</CodeSnippet>
      </section>

      <section id="project-status" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">
          Project Status
        </h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Echo currently supports a small but growing PHP-compatible slice
          across parsing, AST generation, LLVM IR codegen, runtime behavior, and
          CLI execution. The docs should make that boundary visible as the
          language grows.
        </p>
      </section>
    </DocsShell>
  );
}

function SourceModesPage() {
  return (
    <DocsShell
      category="Getting Started"
      headings={["Source Modes"]}
      title="Source Modes"
    >
      <section id="source-modes" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">
          Source Modes
        </h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Files ending in <code className="font-mono text-slate-950">.php</code>{" "}
          use Echo superset mode by default. Files ending in{" "}
          <code className="font-mono text-slate-950">.echo</code> or{" "}
          <code className="font-mono text-slate-950">.xo</code> use strict mode
          by default.
        </p>
      </section>
    </DocsShell>
  );
}

function PhpBuiltinsPage() {
  return (
    <DocsShell
      category="Language"
      headings={builtinFamilies.map((family) => family.title)}
      title="PHP Built-ins"
    >
      <p className="mt-6 text-lg leading-8 text-slate-600">
        PHP built-ins keep familiar names and signatures. They are grouped by
        family so each page can stay focused: strings, arrays, types, math,
        filesystem, reflection, shell integration, output buffering, and core
        runtime helpers.
      </p>

      <div className="mt-10 grid gap-6 sm:grid-cols-2">
        {builtinFamilies.map((family) => (
          <section
            key={family.slug}
            className="border-t border-slate-200 pt-6"
            id={headingId(family.title)}
          >
            <h2 className="text-2xl font-semibold tracking-normal text-slate-950">
              {family.title}
            </h2>
            <p className="mt-4 text-base leading-7 text-slate-600">
              {family.description}
            </p>
            <a
              className="mt-5 inline-flex text-sm font-semibold text-slate-500 transition hover:text-slate-950"
              href={`/docs/php-built-ins/${family.slug}`}
            >
              {family.builtins.length} functions
            </a>
          </section>
        ))}
      </div>
    </DocsShell>
  );
}

function PhpBuiltinFamilyPage({ family }: { family: BuiltinFamily }) {
  return (
    <DocsShell
      category="PHP Built-ins"
      headings={family.builtins.map((builtin) => builtin.name)}
      title={family.title}
    >
      <p className="mt-6 text-lg leading-8 text-slate-600">
        {family.description}
      </p>

      <div className="mt-10 divide-y divide-slate-200 border-y border-slate-200">
        {family.builtins.map((builtin) => (
          <BuiltinReference key={builtin.name} builtin={builtin} />
        ))}
      </div>
    </DocsShell>
  );
}

function BuiltinReference({ builtin }: { builtin: BuiltinDoc }) {
  const example = builtinExample(builtin.name);
  const exampleNote = builtinExampleNote(builtin);

  return (
    <section className="py-8">
      <h2
        className="font-mono text-2xl font-semibold text-slate-950"
        id={headingId(builtin.name)}
      >
        {builtin.name}
      </h2>
      <p className="mt-3 font-mono text-sm text-slate-500">
        {builtin.signature}
      </p>
      <p className="mt-7 text-lg leading-8 text-slate-600">
        {builtin.description}
      </p>

      <CodeSnippet className="mt-7">{example}</CodeSnippet>
      <p className="mt-5 text-base leading-7 text-slate-600">{exampleNote}</p>
    </section>
  );
}

function SourceBuildsPage() {
  return (
    <DocsShell
      category="Tooling"
      headings={["Source Builds"]}
      title="Source Builds"
    >
      <section id="source-builds" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">
          Source Builds
        </h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Contributors can build the current command line from source. Full
          workspace builds require Rust, LLVM 22, clang, and PHP for
          compatibility fixture generation.
        </p>
        <CodeSnippet>{`git clone https://github.com/modoterra/echo.git
cd echo
cargo build -p xo
cargo test --workspace
cargo run -p xo -- run examples/hello.php`}</CodeSnippet>
      </section>
    </DocsShell>
  );
}

const rootRoute = createRootRoute({
  component: RootLayout,
});

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: HomePage,
});

const docsLayoutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs",
  component: DocsLayout,
});

const docsRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "/",
  component: DocsPage,
});

const sourceModesRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "source-modes",
  component: SourceModesPage,
});

const phpBuiltinsRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins",
  component: PhpBuiltinsPage,
});

const phpBuiltinStringsRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/strings",
  component: () => (
    <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("strings")!} />
  ),
});

const phpBuiltinArraysRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/arrays",
  component: () => (
    <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("arrays")!} />
  ),
});

const phpBuiltinTypesRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/types",
  component: () => (
    <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("types")!} />
  ),
});

const phpBuiltinMathRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/math",
  component: () => (
    <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("math")!} />
  ),
});

const phpBuiltinFilesystemRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/filesystem",
  component: () => (
    <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("filesystem")!} />
  ),
});

const phpBuiltinReflectionRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/reflection",
  component: () => (
    <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("reflection")!} />
  ),
});

const phpBuiltinShellRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/shell",
  component: () => (
    <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("shell")!} />
  ),
});

const phpBuiltinOutputBufferingRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/output-buffering",
  component: () => (
    <PhpBuiltinFamilyPage
      family={builtinFamilyBySlug.get("output-buffering")!}
    />
  ),
});

const phpBuiltinCoreRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "php-built-ins/core",
  component: () => (
    <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("core")!} />
  ),
});

const sourceBuildsRoute = createRoute({
  getParentRoute: () => docsLayoutRoute,
  path: "source-builds",
  component: SourceBuildsPage,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  docsLayoutRoute.addChildren([
    docsRoute,
    sourceModesRoute,
    phpBuiltinsRoute,
    phpBuiltinStringsRoute,
    phpBuiltinArraysRoute,
    phpBuiltinTypesRoute,
    phpBuiltinMathRoute,
    phpBuiltinFilesystemRoute,
    phpBuiltinReflectionRoute,
    phpBuiltinShellRoute,
    phpBuiltinOutputBufferingRoute,
    phpBuiltinCoreRoute,
    sourceBuildsRoute,
  ]),
]);

export const router = createRouter({
  defaultViewTransition: {
    types: ({ pathChanged }) => {
      if (!pathChanged) {
        return false;
      }

      return ["route-transition"];
    },
  },
  routeTree,
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
