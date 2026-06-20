import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Outlet,
  useLocation,
} from "@tanstack/react-router";
import type { ReactNode } from "react";
import { HomePage } from "./app";

type DocsNavGroup = {
  title: string;
  links: Array<{
    label: string;
    to: string;
    disabled?: boolean;
  }>;
};

type DocsShellProps = {
  category: string;
  title: string;
  headings: string[];
  children: ReactNode;
};

type BuiltinDoc = {
  name: string;
  signature: string;
  description: string;
  example?: string;
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
    description: "String functions inspect, transform, encode, split, search, and compare text.",
    builtins: [
      {
        name: "strlen",
        signature: "strlen(string $string): int",
        description:
          "Returns the number of bytes in a string. This is byte length, not character length, so multibyte text can be longer than the number of visible characters.",
        example: `let $word = "hello"
let $accented = "é"

echo strlen($word) . "\\n"
echo strlen($accented) . "\\n"`,
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
        example: `let $empty = []
let $numbers = [1, 2, 3]

echo array_is_list($empty) . "\\n"
echo array_is_list($numbers) . "\\n"`,
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
    description: "Reflection functions ask questions about functions and callable values.",
    builtins: [
      {
        name: "function_exists",
        signature: "function_exists(string $function): bool",
        description:
          "Returns true when a name resolves to a function. Language constructs are not functions, so names such as echo and include_once return false.",
        example: `let $callable = "strlen"
let $construct = "echo"

echo function_exists($callable) . "\\n"
echo function_exists($construct) . "\\n"`,
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
      { name: "flush", signature: "flush(): void", description: "Flushes system output buffers." },
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

const builtinFamilyBySlug = new Map(builtinFamilies.map((family) => [family.slug, family]));

function headingId(heading: string) {
  return heading.toLowerCase().replaceAll(" ", "-");
}

function RootLayout() {
  const location = useLocation();
  const isDocs = location.pathname.startsWith("/docs");

  return (
    <>
      <header
        className={
          isDocs
            ? "fixed inset-x-0 top-0 z-30 border-b border-slate-200/70 bg-white/85 px-6 backdrop-blur"
            : "absolute inset-x-0 top-8 z-20 px-6 sm:top-12"
        }
      >
        <div
          className={
            isDocs
              ? "mx-auto flex h-20 w-full max-w-7xl items-center"
              : "mx-auto flex w-full max-w-[624px] items-center"
          }
        >
          {isDocs ? (
            <Link
              aria-label="Echo home"
              className="mr-10 block w-16 transition opacity-90 hover:opacity-100 lg:mr-[214px] lg:w-20"
              to="/"
            >
              <img alt="Echo" className="h-auto w-full" src="/logo.svg" />
            </Link>
          ) : null}
          <nav
            aria-label="Primary navigation"
            className="flex items-center justify-start gap-8 text-sm font-semibold text-slate-500"
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
      <Outlet />
    </>
  );
}

function DocsShell({ category, title, headings, children }: DocsShellProps) {
  const location = useLocation();
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
        { label: "PHP Built-ins", to: "/docs/php-built-ins" },
        { label: "Strings", to: "/docs/php-built-ins/strings" },
        { label: "Arrays", to: "/docs/php-built-ins/arrays" },
        { label: "Types", to: "/docs/php-built-ins/types" },
        { label: "Math and Bases", to: "/docs/php-built-ins/math" },
        { label: "Filesystem", to: "/docs/php-built-ins/filesystem" },
        { label: "Reflection", to: "/docs/php-built-ins/reflection" },
        { label: "Shell", to: "/docs/php-built-ins/shell" },
        { label: "Output Buffering", to: "/docs/php-built-ins/output-buffering" },
        { label: "Core", to: "/docs/php-built-ins/core" },
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

  return (
    <main className="min-h-screen bg-white px-6 pb-24 pt-32 text-slate-950">
      <div className="mx-auto grid w-full max-w-7xl grid-cols-1 gap-12 lg:grid-cols-[220px_minmax(0,720px)] xl:grid-cols-[220px_minmax(0,720px)_220px]">
        <aside className="hidden lg:block">
          <nav aria-label="Documentation sections" className="sticky top-32 space-y-10">
            {navigation.map((group) => (
              <section key={group.title}>
                <h2 className="text-sm font-semibold text-slate-950">{group.title}</h2>
                <ul className="mt-5 space-y-3">
                  {group.links.map((link) => (
                    <li key={link.label}>
                      {link.disabled ? (
                        <span className="text-sm leading-6 text-slate-300">{link.label}</span>
                      ) : (
                        <Link
                          className={
                            location.pathname === link.to
                              ? "text-sm font-semibold leading-6 text-slate-950"
                              : "text-sm leading-6 text-slate-500 transition hover:text-slate-950"
                          }
                          to={link.to}
                        >
                          {link.label}
                        </Link>
                      )}
                    </li>
                  ))}
                </ul>
              </section>
            ))}
          </nav>
        </aside>

        <article className="max-w-none">
          <p className="text-sm font-semibold text-slate-500">{category}</p>
          <h1 className="mt-6 text-5xl font-semibold tracking-normal text-slate-950">{title}</h1>
          {children}
        </article>

        <aside className="hidden xl:block">
          <nav aria-label="On this page" className="sticky top-32 border-l border-slate-200 pl-6">
            <h2 className="text-sm font-semibold text-slate-950">On this page</h2>
            <ul className="mt-5 space-y-3">
              {headings.map((heading) => (
                <li key={heading}>
                  <a
                    className="text-sm leading-6 text-slate-500 transition hover:text-slate-950"
                    href={`#${headingId(heading)}`}
                  >
                    {heading}
                  </a>
                </li>
              ))}
            </ul>
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
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Meet Echo</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Echo is a Rust implementation of a PHP superset. Existing PHP should stay familiar, while
          Echo adds compiler tooling, native concurrency, parallel execution, and a path toward
          compiled binaries with predictable performance gains.
        </p>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          The command line entrypoint is <code className="font-mono text-slate-950">xo</code>. Echo
          is early-stage software, so unsupported PHP behavior should fail explicitly rather than
          silently approximate semantics.
        </p>
      </section>

      <section id="installation" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Installation</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Install the <code className="font-mono text-slate-950">xo</code> command and keep it on
          your path. The public installer flow is still being designed, so current releases are
          source-built by contributors.
        </p>
        <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
          <code>{`xo --help
xo run app.php
xo build app.php -o app`}</code>
        </pre>
      </section>

      <section id="run-a-program" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Run a Program</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Use <code className="font-mono text-slate-950">xo run</code> to execute an Echo-compatible
          PHP file directly from the command line.
        </p>
        <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
          <code>{`xo run examples/hello.php`}</code>
        </pre>
      </section>

      <section id="compile-a-program" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Compile a Program</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Use <code className="font-mono text-slate-950">xo build</code> to compile a supported
          program into a native binary. The current backend lowers through LLVM IR and links through
          the project build path while Echo's native toolchain matures.
        </p>
        <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
          <code>{`xo build examples/hello.php -o /tmp/hello
/tmp/hello`}</code>
        </pre>
      </section>

      <section id="project-status" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Project Status</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Echo currently supports a small but growing PHP-compatible slice across parsing, AST
          generation, LLVM IR codegen, runtime behavior, and CLI execution. The docs should make
          that boundary visible as the language grows.
        </p>
      </section>
    </DocsShell>
  );
}

function SourceModesPage() {
  return (
    <DocsShell category="Getting Started" headings={["Source Modes"]} title="Source Modes">
      <section id="source-modes" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Source Modes</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Files ending in <code className="font-mono text-slate-950">.php</code> use Echo superset
          mode by default. Files ending in <code className="font-mono text-slate-950">.echo</code>{" "}
          or <code className="font-mono text-slate-950">.xo</code> use strict mode by default.
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
        PHP built-ins keep familiar names and signatures. They are grouped by family so each page
        can stay focused: strings, arrays, types, math, filesystem, reflection, shell integration,
        output buffering, and core runtime helpers.
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
            <p className="mt-4 text-base leading-7 text-slate-600">{family.description}</p>
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
      <p className="mt-6 text-lg leading-8 text-slate-600">{family.description}</p>

      <div className="mt-10 divide-y divide-slate-200 border-y border-slate-200">
        {family.builtins.map((builtin) => (
          <section key={builtin.name} className="py-8">
            <h2
              className="font-mono text-2xl font-semibold text-slate-950"
              id={headingId(builtin.name)}
            >
              {builtin.name}
            </h2>
            <p className="mt-3 font-mono text-sm text-slate-500">{builtin.signature}</p>
            <p className="mt-7 text-lg leading-8 text-slate-600">{builtin.description}</p>

            {builtin.example ? (
              <pre className="mt-7 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
                <code>{builtin.example}</code>
              </pre>
            ) : null}
          </section>
        ))}
      </div>
    </DocsShell>
  );
}

function SourceBuildsPage() {
  return (
    <DocsShell category="Tooling" headings={["Source Builds"]} title="Source Builds">
      <section id="source-builds" className="mt-16 scroll-mt-28">
        <h2 className="text-3xl font-semibold tracking-normal text-slate-950">Source Builds</h2>
        <p className="mt-6 text-lg leading-8 text-slate-600">
          Contributors can build the current command line from source. Full workspace builds require
          Rust, LLVM 22, clang, and PHP for compatibility fixture generation.
        </p>
        <pre className="mt-8 overflow-x-auto rounded-lg bg-[#101218] p-6 text-sm leading-7 text-slate-100 shadow-sm">
          <code>{`git clone https://github.com/modoterra/echo.git
cd echo
cargo build -p xo
cargo test --workspace
cargo run -p xo -- run examples/hello.php`}</code>
        </pre>
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

const docsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs",
  component: DocsPage,
});

const sourceModesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/source-modes",
  component: SourceModesPage,
});

const phpBuiltinsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins",
  component: PhpBuiltinsPage,
});

const phpBuiltinStringsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins/strings",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("strings")!} />,
});

const phpBuiltinArraysRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins/arrays",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("arrays")!} />,
});

const phpBuiltinTypesRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins/types",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("types")!} />,
});

const phpBuiltinMathRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins/math",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("math")!} />,
});

const phpBuiltinFilesystemRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins/filesystem",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("filesystem")!} />,
});

const phpBuiltinReflectionRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins/reflection",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("reflection")!} />,
});

const phpBuiltinShellRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins/shell",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("shell")!} />,
});

const phpBuiltinOutputBufferingRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins/output-buffering",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("output-buffering")!} />,
});

const phpBuiltinCoreRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/php-built-ins/core",
  component: () => <PhpBuiltinFamilyPage family={builtinFamilyBySlug.get("core")!} />,
});

const sourceBuildsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs/source-builds",
  component: SourceBuildsPage,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
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
]);

export const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
