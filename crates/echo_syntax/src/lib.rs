#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxMode {
    Php,
    Echo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordRole {
    Declaration,
    Import,
    Statement,
    Expression,
    Modifier,
    Type,
    Literal,
    Namespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Keyword {
    pub text: &'static str,
    pub role: KeywordRole,
    pub mode: SyntaxMode,
}

impl Keyword {
    pub const fn new(text: &'static str, role: KeywordRole, mode: SyntaxMode) -> Self {
        Self { text, role, mode }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    Left,
    Right,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operator {
    pub text: &'static str,
    pub name: &'static str,
    pub precedence: u8,
    pub associativity: Associativity,
    pub mode: SyntaxMode,
}

impl Operator {
    pub const fn new(
        text: &'static str,
        name: &'static str,
        precedence: u8,
        associativity: Associativity,
        mode: SyntaxMode,
    ) -> Self {
        Self {
            text,
            name,
            precedence,
            associativity,
            mode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxRule {
    pub name: &'static str,
    pub kind: SyntaxRuleKind,
    pub mode: SyntaxMode,
}

impl SyntaxRule {
    pub const fn new(name: &'static str, kind: SyntaxRuleKind, mode: SyntaxMode) -> Self {
        Self { name, kind, mode }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxRuleKind {
    Declaration,
    Statement,
    Expression,
    Literal,
    Name,
}

pub mod keywords {
    use super::{Keyword, KeywordRole, SyntaxMode};

    pub const ABSTRACT: Keyword = Keyword::new("abstract", KeywordRole::Modifier, SyntaxMode::Php);
    pub const AS: Keyword = Keyword::new("as", KeywordRole::Import, SyntaxMode::Echo);
    pub const BREAK: Keyword = Keyword::new("break", KeywordRole::Statement, SyntaxMode::Php);
    pub const CLASS: Keyword = Keyword::new("class", KeywordRole::Declaration, SyntaxMode::Php);
    pub const COMPILE: Keyword =
        Keyword::new("compile", KeywordRole::Declaration, SyntaxMode::Echo);
    pub const CONST: Keyword = Keyword::new("const", KeywordRole::Declaration, SyntaxMode::Php);
    pub const DEFINE: Keyword = Keyword::new("define", KeywordRole::Expression, SyntaxMode::Php);
    pub const DEFER: Keyword = Keyword::new("defer", KeywordRole::Expression, SyntaxMode::Echo);
    pub const ECHO: Keyword = Keyword::new("echo", KeywordRole::Statement, SyntaxMode::Php);
    pub const ELSE: Keyword = Keyword::new("else", KeywordRole::Statement, SyntaxMode::Php);
    pub const ELSEIF: Keyword = Keyword::new("elseif", KeywordRole::Statement, SyntaxMode::Php);
    pub const ENUM: Keyword = Keyword::new("enum", KeywordRole::Declaration, SyntaxMode::Php);
    pub const FALSE: Keyword = Keyword::new("false", KeywordRole::Literal, SyntaxMode::Php);
    pub const FACET: Keyword = Keyword::new("facet", KeywordRole::Declaration, SyntaxMode::Echo);
    pub const FINAL: Keyword = Keyword::new("final", KeywordRole::Modifier, SyntaxMode::Php);
    pub const FN: Keyword = Keyword::new("fn", KeywordRole::Declaration, SyntaxMode::Echo);
    pub const FORK: Keyword = Keyword::new("fork", KeywordRole::Expression, SyntaxMode::Echo);
    pub const FROM: Keyword = Keyword::new("from", KeywordRole::Import, SyntaxMode::Echo);
    pub const FUNCTION: Keyword =
        Keyword::new("function", KeywordRole::Declaration, SyntaxMode::Php);
    pub const GEN: Keyword = Keyword::new("gen", KeywordRole::Declaration, SyntaxMode::Echo);
    pub const IF: Keyword = Keyword::new("if", KeywordRole::Statement, SyntaxMode::Php);
    pub const INCLUDE: Keyword = Keyword::new("include", KeywordRole::Expression, SyntaxMode::Php);
    pub const INCLUDE_ONCE: Keyword =
        Keyword::new("include_once", KeywordRole::Expression, SyntaxMode::Php);
    pub const INTERFACE: Keyword =
        Keyword::new("interface", KeywordRole::Declaration, SyntaxMode::Php);
    pub const INTRINSIC: Keyword =
        Keyword::new("intrinsic", KeywordRole::Modifier, SyntaxMode::Echo);
    pub const IS: Keyword = Keyword::new("is", KeywordRole::Expression, SyntaxMode::Echo);
    pub const JOIN: Keyword = Keyword::new("join", KeywordRole::Expression, SyntaxMode::Echo);
    pub const LET: Keyword = Keyword::new("let", KeywordRole::Statement, SyntaxMode::Echo);
    pub const LOOP: Keyword = Keyword::new("loop", KeywordRole::Statement, SyntaxMode::Echo);
    pub const MODULE: Keyword = Keyword::new("module", KeywordRole::Namespace, SyntaxMode::Echo);
    pub const NAMESPACE: Keyword =
        Keyword::new("namespace", KeywordRole::Namespace, SyntaxMode::Php);
    pub const NEW: Keyword = Keyword::new("new", KeywordRole::Expression, SyntaxMode::Php);
    pub const NOT: Keyword = Keyword::new("not", KeywordRole::Expression, SyntaxMode::Echo);
    pub const NULL: Keyword = Keyword::new("null", KeywordRole::Literal, SyntaxMode::Php);
    pub const PRIVATE: Keyword = Keyword::new("private", KeywordRole::Modifier, SyntaxMode::Php);
    pub const PROTECTED: Keyword =
        Keyword::new("protected", KeywordRole::Modifier, SyntaxMode::Php);
    pub const PUB: Keyword = Keyword::new("pub", KeywordRole::Modifier, SyntaxMode::Echo);
    pub const PUBLIC: Keyword = Keyword::new("public", KeywordRole::Modifier, SyntaxMode::Php);
    pub const READONLY: Keyword = Keyword::new("readonly", KeywordRole::Modifier, SyntaxMode::Php);
    pub const REQUIRE: Keyword = Keyword::new("require", KeywordRole::Expression, SyntaxMode::Php);
    pub const REQUIRE_ONCE: Keyword =
        Keyword::new("require_once", KeywordRole::Expression, SyntaxMode::Php);
    pub const RETURN: Keyword = Keyword::new("return", KeywordRole::Statement, SyntaxMode::Php);
    pub const RUN: Keyword = Keyword::new("run", KeywordRole::Expression, SyntaxMode::Echo);
    pub const SPAWN: Keyword = Keyword::new("spawn", KeywordRole::Expression, SyntaxMode::Echo);
    pub const STATIC: Keyword = Keyword::new("static", KeywordRole::Modifier, SyntaxMode::Php);
    pub const STD: Keyword = Keyword::new("std", KeywordRole::Namespace, SyntaxMode::Echo);
    pub const THROW: Keyword = Keyword::new("throw", KeywordRole::Statement, SyntaxMode::Php);
    pub const TRUE: Keyword = Keyword::new("true", KeywordRole::Literal, SyntaxMode::Php);
    pub const TYPE: Keyword = Keyword::new("type", KeywordRole::Declaration, SyntaxMode::Echo);
    pub const USE: Keyword = Keyword::new("use", KeywordRole::Import, SyntaxMode::Php);
    pub const YIELD: Keyword = Keyword::new("yield", KeywordRole::Statement, SyntaxMode::Php);

    pub const ALL: &[Keyword] = &[
        ABSTRACT,
        AS,
        BREAK,
        CLASS,
        COMPILE,
        CONST,
        DEFINE,
        DEFER,
        ECHO,
        ELSE,
        ELSEIF,
        ENUM,
        FALSE,
        FACET,
        FINAL,
        FN,
        FORK,
        FROM,
        FUNCTION,
        GEN,
        IF,
        INCLUDE,
        INCLUDE_ONCE,
        INTERFACE,
        INTRINSIC,
        IS,
        JOIN,
        LET,
        LOOP,
        MODULE,
        NAMESPACE,
        NEW,
        NOT,
        NULL,
        PRIVATE,
        PROTECTED,
        PUB,
        PUBLIC,
        READONLY,
        REQUIRE,
        REQUIRE_ONCE,
        RETURN,
        RUN,
        SPAWN,
        STATIC,
        STD,
        THROW,
        TRUE,
        TYPE,
        USE,
        YIELD,
    ];

    pub fn is_keyword(text: &str) -> bool {
        ALL.iter()
            .any(|keyword| keyword.text.eq_ignore_ascii_case(text))
            || matches!(text, "__DIR__")
    }
}

pub mod operators {
    use super::{Associativity, Operator, SyntaxMode};

    pub const CONCAT: Operator =
        Operator::new(".", "concat", 6, Associativity::Left, SyntaxMode::Php);
    pub const ADD: Operator = Operator::new("+", "add", 7, Associativity::Left, SyntaxMode::Php);
    pub const SUBTRACT: Operator =
        Operator::new("-", "subtract", 7, Associativity::Left, SyntaxMode::Php);
    pub const MULTIPLY: Operator =
        Operator::new("*", "multiply", 8, Associativity::Left, SyntaxMode::Php);
    pub const DIVIDE: Operator =
        Operator::new("/", "divide", 8, Associativity::Left, SyntaxMode::Php);
    pub const MODULO: Operator =
        Operator::new("%", "modulo", 8, Associativity::Left, SyntaxMode::Php);

    pub const ALL: &[Operator] = &[CONCAT, ADD, SUBTRACT, MULTIPLY, DIVIDE, MODULO];
}

pub mod rules {
    use super::{SyntaxMode, SyntaxRule, SyntaxRuleKind};

    pub const PROGRAM: SyntaxRule =
        SyntaxRule::new("program", SyntaxRuleKind::Declaration, SyntaxMode::Php);
    pub const MODULE_DECLARATION: SyntaxRule = SyntaxRule::new(
        "module_declaration",
        SyntaxRuleKind::Declaration,
        SyntaxMode::Echo,
    );
    pub const NAMESPACE_DECLARATION: SyntaxRule = SyntaxRule::new(
        "namespace_declaration",
        SyntaxRuleKind::Declaration,
        SyntaxMode::Php,
    );
    pub const USE_DECLARATION: SyntaxRule = SyntaxRule::new(
        "use_declaration",
        SyntaxRuleKind::Declaration,
        SyntaxMode::Php,
    );
    pub const FROM_USE_DECLARATION: SyntaxRule = SyntaxRule::new(
        "from_use_declaration",
        SyntaxRuleKind::Declaration,
        SyntaxMode::Echo,
    );
    pub const FUNCTION_DECLARATION: SyntaxRule = SyntaxRule::new(
        "function_declaration",
        SyntaxRuleKind::Declaration,
        SyntaxMode::Php,
    );
    pub const CLASS_DECLARATION: SyntaxRule = SyntaxRule::new(
        "class_declaration",
        SyntaxRuleKind::Declaration,
        SyntaxMode::Php,
    );
    pub const TYPE_DECLARATION: SyntaxRule = SyntaxRule::new(
        "type_declaration",
        SyntaxRuleKind::Declaration,
        SyntaxMode::Echo,
    );
    pub const LET_STATEMENT: SyntaxRule =
        SyntaxRule::new("let_statement", SyntaxRuleKind::Statement, SyntaxMode::Echo);
    pub const ECHO_STATEMENT: SyntaxRule =
        SyntaxRule::new("echo_statement", SyntaxRuleKind::Statement, SyntaxMode::Php);
    pub const IF_STATEMENT: SyntaxRule =
        SyntaxRule::new("if_statement", SyntaxRuleKind::Statement, SyntaxMode::Php);
    pub const LOOP_STATEMENT: SyntaxRule = SyntaxRule::new(
        "loop_statement",
        SyntaxRuleKind::Statement,
        SyntaxMode::Echo,
    );
    pub const BREAK_STATEMENT: SyntaxRule = SyntaxRule::new(
        "break_statement",
        SyntaxRuleKind::Statement,
        SyntaxMode::Php,
    );
    pub const RETURN_STATEMENT: SyntaxRule = SyntaxRule::new(
        "return_statement",
        SyntaxRuleKind::Statement,
        SyntaxMode::Php,
    );
    pub const YIELD_STATEMENT: SyntaxRule = SyntaxRule::new(
        "yield_statement",
        SyntaxRuleKind::Statement,
        SyntaxMode::Php,
    );
    pub const EXPRESSION_STATEMENT: SyntaxRule = SyntaxRule::new(
        "expression_statement",
        SyntaxRuleKind::Statement,
        SyntaxMode::Php,
    );
    pub const BINARY_EXPRESSION: SyntaxRule = SyntaxRule::new(
        "binary_expression",
        SyntaxRuleKind::Expression,
        SyntaxMode::Php,
    );
    pub const CALL_EXPRESSION: SyntaxRule = SyntaxRule::new(
        "call_expression",
        SyntaxRuleKind::Expression,
        SyntaxMode::Php,
    );
    pub const VARIABLE: SyntaxRule =
        SyntaxRule::new("variable", SyntaxRuleKind::Name, SyntaxMode::Php);
    pub const IDENTIFIER: SyntaxRule =
        SyntaxRule::new("identifier", SyntaxRuleKind::Name, SyntaxMode::Php);
    pub const STRING: SyntaxRule =
        SyntaxRule::new("string", SyntaxRuleKind::Literal, SyntaxMode::Php);
    pub const NUMBER: SyntaxRule =
        SyntaxRule::new("number", SyntaxRuleKind::Literal, SyntaxMode::Php);

    pub const ALL: &[SyntaxRule] = &[
        PROGRAM,
        MODULE_DECLARATION,
        NAMESPACE_DECLARATION,
        USE_DECLARATION,
        FROM_USE_DECLARATION,
        FUNCTION_DECLARATION,
        CLASS_DECLARATION,
        TYPE_DECLARATION,
        LET_STATEMENT,
        ECHO_STATEMENT,
        IF_STATEMENT,
        LOOP_STATEMENT,
        BREAK_STATEMENT,
        RETURN_STATEMENT,
        YIELD_STATEMENT,
        EXPRESSION_STATEMENT,
        BINARY_EXPRESSION,
        CALL_EXPRESSION,
        VARIABLE,
        IDENTIFIER,
        STRING,
        NUMBER,
    ];
}

pub mod tree_sitter {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct GrammarRule {
        pub name: &'static str,
        pub body: &'static str,
    }

    impl GrammarRule {
        pub const fn new(name: &'static str, body: &'static str) -> Self {
            Self { name, body }
        }
    }

    pub const RULES: &[GrammarRule] = &[
        GrammarRule::new(
            "source_file",
            "seq(optional($.php_open_tag), repeat($._statement))",
        ),
        GrammarRule::new("php_open_tag", "choice(\"<?php\", \"<?PHP\")"),
        GrammarRule::new(
            "_statement",
            r#"choice(
      $.module_declaration,
      $.namespace_declaration,
      $.use_declaration,
      $.from_use_declaration,
      $.function_declaration,
      $.class_declaration,
      $.facet_declaration,
      $.type_declaration,
      $.let_statement,
      $.echo_statement,
      $.if_statement,
      $.loop_statement,
      $.break_statement,
      $.return_statement,
      $.yield_statement,
      $.expression_statement,
    )"#,
        ),
        GrammarRule::new(
            "module_declaration",
            r#"seq("module", field("path", $.module_path), optional($.terminator))"#,
        ),
        GrammarRule::new(
            "namespace_declaration",
            r#"seq("namespace", field("path", $.qualified_name), optional($.terminator))"#,
        ),
        GrammarRule::new(
            "use_declaration",
            r#"seq(
      "use",
      field("path", choice($.module_path, $.qualified_name)),
      optional(seq("as", field("alias", $.identifier))),
      optional($.terminator),
    )"#,
        ),
        GrammarRule::new(
            "from_use_declaration",
            r#"prec.right(PREC.statement, seq(
      "from",
      field("source", $.module_path),
      "use",
      commaSep1($.import_item),
      optional($.terminator),
    ))"#,
        ),
        GrammarRule::new(
            "import_item",
            r#"seq(
      field("name", $.identifier),
      optional(seq("as", field("alias", $.identifier))),
    )"#,
        ),
        GrammarRule::new(
            "function_declaration",
            r#"seq(
      optional(choice("intrinsic", "gen")),
      choice("fn", "function"),
      field("name", $.identifier),
      field("parameters", $.parameter_list),
      optional(seq(":", field("return_type", $.type_name))),
      field("body", $.block),
    )"#,
        ),
        GrammarRule::new(
            "class_declaration",
            r#"seq(
      "class",
      field("name", $.identifier),
      optional(seq("extends", field("superclass", choice($.dotted_name, $.qualified_name, $.identifier)))),
      field("body", $.class_body),
    )"#,
        ),
        GrammarRule::new(
            "type_declaration",
            r#"seq("type", field("name", $.identifier), field("body", $.class_body))"#,
        ),
        GrammarRule::new(
            "facet_declaration",
            r#"seq(
      "facet",
      field("target", choice($.module_path, $.qualified_name, $.identifier)),
      "as",
      field("receiver", seq("$", $.identifier)),
      field("body", $.class_body),
    )"#,
        ),
        GrammarRule::new(
            "class_body",
            r#"seq("{", repeat(choice($.method_declaration, $.const_declaration)), "}")"#,
        ),
        GrammarRule::new(
            "method_declaration",
            r#"seq(
      optional(choice("pub", "public", "protected", "private")),
      optional("static"),
      choice("fn", "function"),
      field("name", $.identifier),
      field("parameters", $.parameter_list),
      optional(seq(":", field("return_type", $.type_name))),
      field("body", $.block),
    )"#,
        ),
        GrammarRule::new(
            "const_declaration",
            r#"seq("const", field("name", $.identifier), optional(seq("=", $._expression)), optional($.terminator))"#,
        ),
        GrammarRule::new(
            "parameter_list",
            r#"seq("(", optional(commaSep($.parameter)), ")")"#,
        ),
        GrammarRule::new(
            "parameter",
            r#"choice(
      seq(optional(field("type", $.type_name)), field("name", $.variable)),
      field("name", "self"),
    )"#,
        ),
        GrammarRule::new(
            "let_statement",
            r#"seq("let", field("name", choice($.variable, $.identifier)), optional(seq("=", $._expression)), optional($.terminator))"#,
        ),
        GrammarRule::new(
            "echo_statement",
            r#"prec.right(PREC.statement, seq("echo", commaSep1($._expression), optional($.terminator)))"#,
        ),
        GrammarRule::new(
            "if_statement",
            r#"seq("if", field("condition", $._expression), field("consequence", $.block), optional(seq("else", field("alternative", choice($.block, $.if_statement)))))"#,
        ),
        GrammarRule::new("loop_statement", r#"seq("loop", field("body", $.block))"#),
        GrammarRule::new("break_statement", r#"seq("break", optional($.terminator))"#),
        GrammarRule::new(
            "return_statement",
            r#"prec.right(PREC.statement, seq("return", optional($._expression), optional($.terminator)))"#,
        ),
        GrammarRule::new(
            "yield_statement",
            r#"prec.right(PREC.statement, seq("yield", optional($._expression), optional($.terminator)))"#,
        ),
        GrammarRule::new(
            "expression_statement",
            "seq($._expression, optional($.terminator))",
        ),
        GrammarRule::new("block", r#"seq("{", repeat($._statement), "}")"#),
        GrammarRule::new(
            "_expression",
            r#"choice(
      $.binary_expression,
      $.call_expression,
      $.member_expression,
      $.static_member_expression,
      $.new_expression,
      $.require_expression,
      $.collection_literal,
      $.literal,
      $.magic_constant,
      $.variable,
      $.qualified_name,
      $.dotted_name,
      $.identifier,
      $.parenthesized_expression,
    )"#,
        ),
        GrammarRule::new(
            "binary_expression",
            r#"choice(
      ...[
        [".", PREC.concat],
        ["+", PREC.add],
        ["-", PREC.subtract],
        ["*", PREC.multiply],
        ["/", PREC.divide],
        ["%", PREC.modulo],
      ].map(([operator, precedence]) =>
        prec.left(precedence, seq(field("left", $._expression), field("operator", operator), field("right", $._expression)))
      )
    )"#,
        ),
        GrammarRule::new(
            "call_expression",
            r#"prec(10, seq(field("function", choice($.dotted_name, $.qualified_name, $.identifier)), field("arguments", $.argument_list)))"#,
        ),
        GrammarRule::new(
            "member_expression",
            r#"prec(11, seq(field("object", $._expression), "->", field("member", $.identifier)))"#,
        ),
        GrammarRule::new(
            "static_member_expression",
            r#"prec(11, seq(field("class", choice($.qualified_name, $.identifier)), "::", field("member", $.identifier)))"#,
        ),
        GrammarRule::new(
            "new_expression",
            r#"prec.right(PREC.statement, seq("new", field("class", choice($.dotted_name, $.qualified_name, $.identifier)), optional($.argument_list)))"#,
        ),
        GrammarRule::new(
            "require_expression",
            r#"prec.right(PREC.statement, seq(choice("require", "require_once"), field("path", $._expression)))"#,
        ),
        GrammarRule::new(
            "parenthesized_expression",
            r#"seq("(", $._expression, ")")"#,
        ),
        GrammarRule::new(
            "argument_list",
            r#"seq("(", optional(commaSep($._expression)), ")")"#,
        ),
        GrammarRule::new(
            "collection_literal",
            r#"seq("{", optional(commaSep(choice($.collection_entry, $._expression))), optional(","), "}")"#,
        ),
        GrammarRule::new(
            "collection_entry",
            r#"seq(field("key", $._expression), ":", field("value", $._expression))"#,
        ),
        GrammarRule::new("magic_constant", r#""__DIR__""#),
        GrammarRule::new("literal", "choice($.string, $.number, $.boolean, $.null)"),
        GrammarRule::new("boolean", r#"choice("true", "TRUE", "false", "FALSE")"#),
        GrammarRule::new("null", r#"choice("null", "NULL")"#),
        GrammarRule::new(
            "module_path",
            r#"seq($.identifier, repeat(seq(".", $.identifier)))"#,
        ),
        GrammarRule::new(
            "dotted_name",
            r#"prec(12, seq($.identifier, repeat1(seq(".", $.identifier))))"#,
        ),
        GrammarRule::new(
            "qualified_name",
            r#"seq($.identifier, repeat1(seq("\\", $.identifier)))"#,
        ),
        GrammarRule::new("type_name", "choice($.qualified_name, $.identifier)"),
        GrammarRule::new("variable", r#"/\$[A-Za-z_][A-Za-z0-9_]*/"#),
        GrammarRule::new("identifier", "/[A-Za-z_][A-Za-z0-9_]*/"),
        GrammarRule::new(
            "string",
            r#"choice(
      /"([^"\\]|\\.)*"/,
      /'([^'\\]|\\.)*'/,
    )"#,
        ),
        GrammarRule::new("number", r#"/[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?/"#),
        GrammarRule::new(
            "comment",
            r##"token(choice(
      seq("//", /.*/),
      seq("#", /.*/),
      seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/"),
    ))"##,
        ),
        GrammarRule::new("terminator", r#"";""#),
        GrammarRule::new("keyword", "__ECHO_KEYWORD_CHOICE__"),
    ];
}
