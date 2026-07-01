use crate::CoreRuntimeSymbol;
use echo_ast::{ImportSource, Stmt};
use echo_diagnostics::Diagnostic;
use echo_source::Span;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub(crate) enum RuntimeValue {
    StaticString(String),
    EchoValue(String),
    Object {
        value: String,
        class_name: Option<String>,
    },
    Closure {
        params: Vec<echo_ast::TypedParam>,
        body: Vec<echo_mir::MirStmt>,
    },
}

pub(crate) fn stmt_span(statement: &Stmt) -> Span {
    match statement {
        Stmt::Echo(statement) => statement.span,
        Stmt::Compile(statement) => statement.span,
        Stmt::FunctionCall(statement) => statement.span,
        Stmt::DynamicFunctionCall(statement) => statement.span,
        Stmt::FunctionDecl(statement) => statement.span,
        Stmt::Assign(statement) => statement.span,
        Stmt::CoalesceAssign(statement) => statement.span,
        Stmt::ListAssign(statement) => statement.span,
        Stmt::Let(statement) => statement.span,
        Stmt::AssignRef(statement) => statement.span,
        Stmt::Return(statement) => statement.span,
        Stmt::Throw(statement) => statement.span,
        Stmt::Yield(statement) => statement.span,
        Stmt::Global(statement) => statement.span,
        Stmt::StaticVar(statement) => statement.span,
        Stmt::Expr(statement) => statement.span,
        Stmt::Namespace(statement) => statement.span,
        Stmt::Use(statement) => statement.span,
        Stmt::Import(statement) => statement.span,
        Stmt::UnnamedExport(statement) => statement.span,
        Stmt::ClassDecl(statement) => statement.span,
        Stmt::InterfaceDecl(statement) => statement.span,
        Stmt::TraitDecl(statement) => statement.span,
        Stmt::EnumDecl(statement) => statement.span,
        Stmt::FacetDecl(statement) => statement.span,
        Stmt::TypeDecl(statement) => statement.span,
        Stmt::Loop(statement) => statement.span,
        Stmt::While(statement) => statement.span,
        Stmt::For(statement) => statement.span,
        Stmt::Foreach(statement) => statement.span,
        Stmt::Switch(statement) => statement.span,
        Stmt::If(statement) => statement.span,
        Stmt::Try(statement) => statement.span,
        Stmt::Break(statement) => statement.span,
        Stmt::Continue(statement) => statement.span,
        Stmt::Append(statement) => statement.span,
    }
}

pub(crate) struct IrModule {
    pub(crate) globals: String,
    pub(crate) functions_ir: String,
    pub(crate) aliases: HashMap<String, String>,
    pub(crate) std_imports: HashMap<String, String>,
    pub(crate) locals: HashMap<String, RuntimeValue>,
    pub(crate) static_property_globals: HashMap<String, String>,
    pub(crate) functions: HashMap<String, echo_mir::MirFunction>,
    pub(crate) class_parents: HashMap<String, String>,
    pub(crate) class_traits: HashMap<String, Vec<String>>,
    pub(crate) property_types: HashMap<String, String>,
    pub(crate) rendered_functions: HashSet<String>,
    pub(crate) include_units: HashMap<String, String>,
    pub(crate) dynamic_include_units: HashMap<String, String>,
    pub(crate) class_include_units: HashMap<String, String>,
    pub(crate) include_once_globals: HashMap<String, String>,
    pub(crate) source_dir: Option<String>,
    pub(crate) current_class: Option<String>,
    pub(crate) returned: bool,
    pub(crate) terminated: bool,
    pub(crate) break_labels: Vec<String>,
    pub(crate) continue_labels: Vec<String>,
    pub(crate) break_value_slots: Vec<Option<String>>,
    pub(crate) next_string_id: usize,
    pub(crate) next_call_id: usize,
    pub(crate) next_defer_id: usize,
    pub(crate) next_loop_id: usize,
    pub(crate) next_if_id: usize,
    pub(crate) next_static_property_id: usize,
}

impl IrModule {
    pub(crate) fn new() -> Self {
        Self {
            globals: String::new(),
            functions_ir: String::new(),
            aliases: HashMap::new(),
            std_imports: HashMap::new(),
            locals: HashMap::new(),
            static_property_globals: HashMap::new(),
            functions: HashMap::new(),
            class_parents: HashMap::new(),
            class_traits: HashMap::new(),
            property_types: HashMap::new(),
            rendered_functions: HashSet::new(),
            include_units: HashMap::new(),
            dynamic_include_units: HashMap::new(),
            class_include_units: HashMap::new(),
            include_once_globals: HashMap::new(),
            source_dir: None,
            current_class: None,
            returned: false,
            terminated: false,
            break_labels: Vec::new(),
            continue_labels: Vec::new(),
            break_value_slots: Vec::new(),
            next_string_id: 0,
            next_call_id: 0,
            next_defer_id: 0,
            next_loop_id: 0,
            next_if_id: 0,
            next_static_property_id: 0,
        }
    }

    pub(crate) fn runtime_value_as_echo_value(
        &mut self,
        body: &mut String,
        value: RuntimeValue,
    ) -> String {
        match value {
            RuntimeValue::EchoValue(name) => format!("%EchoValue {name}"),
            RuntimeValue::Object { value, .. } => format!("%EchoValue {value}"),
            RuntimeValue::Closure { .. } => {
                let RuntimeValue::EchoValue(name) = self.render_runtime_object_new(body) else {
                    unreachable!("runtime object constructor returns an EchoValue")
                };
                format!("%EchoValue {name}")
            }
            RuntimeValue::StaticString(value) => {
                let global = self.string_global(&value);
                let name = self.push_echo_value_call(
                    body,
                    CoreRuntimeSymbol::ValueString.symbol(),
                    &format!("ptr @{global}, i64 {}", value.len()),
                );

                format!("%EchoValue {name}")
            }
        }
    }

    pub(crate) fn next_runtime_call_name(&mut self) -> String {
        let call_id = self.next_call_id;
        self.next_call_id += 1;
        format!("%runtime_call_{call_id}")
    }

    pub(crate) fn push_echo_value_call(
        &mut self,
        body: &mut String,
        symbol: &str,
        args: &str,
    ) -> String {
        let name = self.next_runtime_call_name();
        body.push_str(&format!("  {name} = call %EchoValue @{symbol}({args})\n"));
        name
    }

    pub(crate) fn write_call(&mut self, body: &mut String, value: &str) {
        let global = self.string_global(value);
        body.push_str(&format!(
            "  call void @{}(ptr @{global}, i64 {})\n",
            CoreRuntimeSymbol::Write.symbol(),
            value.len()
        ));
    }

    pub(crate) fn write_value(&mut self, body: &mut String, value: RuntimeValue) {
        match value {
            RuntimeValue::StaticString(value) => self.write_call(body, &value),
            RuntimeValue::Object { value, .. } => body.push_str(&format!(
                "  call void @{}(%EchoValue {value})\n",
                CoreRuntimeSymbol::WriteValue.symbol()
            )),
            RuntimeValue::Closure { .. } => {
                let value = self.runtime_value_as_echo_value(body, value);
                body.push_str(&format!(
                    "  call void @{}({value})\n",
                    CoreRuntimeSymbol::WriteValue.symbol()
                ));
            }
            RuntimeValue::EchoValue(name) => body.push_str(&format!(
                "  call void @{}(%EchoValue {name})\n",
                CoreRuntimeSymbol::WriteValue.symbol()
            )),
        }
    }

    pub(crate) fn resolve_alias(&self, name: &str) -> String {
        let mut current = name;

        while let Some(next) = self.aliases.get(current) {
            current = next;
        }

        current.to_string()
    }

    pub(crate) fn register_std_import(
        &mut self,
        statement: &echo_ast::ImportStmt,
    ) -> Result<(), Diagnostic> {
        if statement.source != ImportSource::Std {
            return Ok(());
        }

        let Some(module) = statement.name.parts.first() else {
            return Err(Diagnostic::new(
                "empty std import in LLVM codegen",
                statement.span,
            ));
        };

        if !is_known_std_module(module) {
            return Err(Diagnostic::new(
                format!("unknown std import `{}`", statement.name.as_string()),
                statement.span,
            ));
        }

        if statement.name.parts.len() == 1 {
            let local = statement.alias.as_deref().unwrap_or(module).to_string();
            self.std_imports.insert(local, module.clone());
        }

        Ok(())
    }

    pub(crate) fn resolve_std_call_name(
        &self,
        name: &str,
        span: Span,
    ) -> Result<String, Diagnostic> {
        let Some((module, rest)) = name.split_once('.') else {
            return Ok(name.to_string());
        };

        if let Some(imported) = self.std_imports.get(module) {
            return Ok(format!("{imported}.{rest}"));
        }

        if is_known_std_module(module) {
            return Err(Diagnostic::new(
                format!("std module `{module}` must be imported before use"),
                span,
            ));
        }

        Ok(name.to_string())
    }

    pub(crate) fn string_global(&mut self, value: &str) -> String {
        let name = format!("echo_str_{}", self.next_string_id);
        self.next_string_id += 1;

        self.globals.push_str(&format!(
            "@{name} = private unnamed_addr constant [{} x i8] c\"{}\", align 1\n",
            value.len(),
            llvm_string_literal(value)
        ));

        name
    }
}

fn is_known_std_module(name: &str) -> bool {
    let module_name = format!("std.{name}");
    echo_std::modules()
        .iter()
        .any(|module| module.name == module_name)
}

fn llvm_string_literal(value: &str) -> String {
    let mut output = String::new();

    for byte in value.bytes() {
        match byte {
            b'\\' => output.push_str(r#"\5C"#),
            b'"' => output.push_str(r#"\22"#),
            0x20..=0x7e => output.push(byte as char),
            _ => output.push_str(&format!(r#"\{byte:02X}"#)),
        }
    }

    output
}
