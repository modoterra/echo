//! Echo language toolchain CLI.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, ExitCode, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

mod repl;

use clap::{Parser, Subcommand};
use echo_ast::format_ast_kinds;
use echo_build::project_root_for;
use echo_cache::{ArtifactStore, CacheLayout};
use echo_diagnostics::{Diagnostics, Severity};
use echo_fingerprint::{ArtifactPhase, CACHE_FORMAT_VERSION, phase_fingerprint};
use echo_lexer::{format_diag_codes, format_token_kinds, format_tokens, lex};
use echo_parser::parse;
use echo_pipeline::{AnalyzeOptions, OptLevel, analyze, compile_to_llvm_with};
use echo_resolver::{
    format_check_diag_codes, install_git, install_local_dir, packages_root, xo_home,
    CheckCacheOutcome, PackageSpec, XoToml, XO_HOME_ENV,
};
use echo_source::SourceMap;

#[derive(Debug, Parser)]
#[command(name = "xo")]
#[command(about = "Echo language toolchain")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Dump lexer tokens for a source file.
    Lex {
        /// One token kind per line (fixture-stable; used by `e26`).
        #[arg(long)]
        kinds: bool,
        /// Print diagnostic codes only on stderr (one per line; used by `e26`).
        #[arg(long)]
        diag_codes: bool,
        file: PathBuf,
    },
    /// Dump the AST for a source file (lex → parse → echo_ast).
    Ast {
        /// Stable kind tree (fixture-stable; used by `e26`).
        #[arg(long)]
        kinds: bool,
        /// Print diagnostic codes only on stderr (one per line; used by `e26`).
        #[arg(long)]
        diag_codes: bool,
        file: PathBuf,
    },
    /// Format Echo source via shared parse + AST pretty-print (canonical layout).
    Fmt {
        /// Write formatted text in place (default: print to stdout).
        #[arg(long = "write", short = 'w')]
        write: bool,
        /// Exit 1 if the file is not already formatted (no rewrite).
        #[arg(long = "check", short = 'c')]
        check: bool,
        /// Echo source file.
        file: PathBuf,
    },
    /// Emit LLVM IR for a source file (check → HIR → MIR → LLVM).
    Ir {
        /// Print diagnostic codes only on stderr (one per line).
        #[arg(long)]
        diag_codes: bool,
        /// Skip parse/check/codegen caches.
        #[arg(long)]
        no_cache: bool,
        /// Print parse/check/codegen cache outcomes on stderr.
        #[arg(long)]
        cache_status: bool,
        /// LLVM optimization level: 0, 1, 2, 3, or z (Oz). Default 0.
        #[arg(
            short = 'O',
            long = "opt-level",
            default_value = "0",
            value_name = "LEVEL"
        )]
        opt_level: String,
        file: PathBuf,
    },
    /// Compile and run a source file (AOT: LLVM IR + clang + libecho_runtime).
    Run {
        /// Execute with the in-process LLVM JIT instead of a temporary native binary.
        #[arg(long)]
        jit: bool,
        /// Print diagnostic codes only on stderr (one per line; used by `e26`).
        #[arg(long)]
        diag_codes: bool,
        /// Skip parse/check/codegen caches.
        #[arg(long)]
        no_cache: bool,
        /// Print parse/check/codegen cache outcomes on stderr.
        #[arg(long)]
        cache_status: bool,
        /// LLVM optimization level: 0, 1, 2, 3, or z (Oz). Default 0.
        #[arg(
            short = 'O',
            long = "opt-level",
            default_value = "0",
            value_name = "LEVEL"
        )]
        opt_level: String,
        file: PathBuf,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Build a native binary.
    Build {
        file: PathBuf,
        #[arg(short = 'o', long)]
        output: Option<PathBuf>,
        /// Skip parse/check/codegen caches.
        #[arg(long)]
        no_cache: bool,
        /// Print parse/check/codegen cache outcomes on stderr.
        #[arg(long)]
        cache_status: bool,
        /// LLVM optimization level: 0, 1, 2, 3, or z (Oz). Default 0.
        #[arg(
            short = 'O',
            long = "opt-level",
            default_value = "0",
            value_name = "LEVEL"
        )]
        opt_level: String,
    },
    /// Run Echo language tests (Model A: `std/test` registration via `XO_TEST`).
    ///
    /// Paths may be `.echo` files, directories, or globs (`*`, `**`). With no
    /// paths, searches `.` for `*_test.echo` and `tests/**/*.echo`.
    ///
    /// With `--bench`, only `test.bench` cases run (auto-N, ns/op); `test.it`
    /// cases are skipped. Without `--bench`, only `test.it` cases run.
    Test {
        /// Run benchmarks only (`test.bench`); skip ordinary test cases.
        #[arg(long)]
        bench: bool,
        /// Files, directories, or glob patterns.
        paths: Vec<String>,
    },
    /// Check from an entry file (closed graph: imports, %/@ merge, local semantics).
    Check {
        /// Print `sem-*` / `res-*` codes on stderr (one per line; used by `e26`).
        #[arg(long)]
        diag_codes: bool,
        /// Print resolved module paths on stdout.
        #[arg(long)]
        graph: bool,
        /// Skip the check-phase artifact cache (always re-run semantics).
        #[arg(long)]
        no_cache: bool,
        /// Print whether the semantic check phase was a cache hit/miss.
        #[arg(long)]
        cache_status: bool,
        file: PathBuf,
    },
    /// Install a package into the user package cache (`$XO_HOME/packages/…`).
    ///
    /// Spec: `github.com/owner/repo[@version]` (git clone), or a local directory
    /// with `--path` / when the argument is an existing directory.
    Get {
        /// Package id and optional `@version` (default version: `default`).
        spec: String,
        /// Copy from this local directory instead of git clone.
        #[arg(long)]
        path: Option<PathBuf>,
        /// Also install dependencies listed in the package's `xo.toml`.
        #[arg(long)]
        deps: bool,
    },
    /// Print the user package cache root (`$XO_HOME`) and packages path.
    Home,
    /// Cache introspection and maintenance.
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
    /// Project index operations.
    Index {
        #[command(subcommand)]
        command: IndexCommand,
    },
    /// Developer tools.
    Tools {
        #[command(subcommand)]
        command: ToolsCommand,
    },
    /// Start the language server.
    Lsp,
    /// Start an interactive REPL.
    Repl,
}

#[derive(Debug, Subcommand)]
enum CacheCommand {
    /// Show cache root and per-phase artifact counts.
    Status {
        /// Project root (default: inferred from cwd).
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Remove the project `.xo` cache tree.
    Clean {
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Placeholder: reclaim unused blobs (not yet implemented).
    Gc {
        #[arg(long)]
        path: Option<PathBuf>,
    },
    /// Print layout health and phase fingerprint samples.
    Doctor {
        #[arg(long)]
        path: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum IndexCommand {
    /// Scan source roots and update the project index.
    Scan { roots: Vec<PathBuf> },
}

#[derive(Debug, Subcommand)]
enum ToolsCommand {
    Grammar {
        #[command(subcommand)]
        command: GrammarCommand,
    },
}

#[derive(Debug, Subcommand)]
enum GrammarCommand {
    /// Generate a tree-sitter grammar package from echo_syntax facts.
    TreeSitter {
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Lex {
            kinds,
            diag_codes,
            file,
        } => cmd_lex(&file, kinds, diag_codes),
        Command::Ast {
            kinds,
            diag_codes,
            file,
        } => cmd_ast(&file, kinds, diag_codes),
        Command::Fmt {
            write,
            check,
            file,
        } => cmd_fmt(&file, write, check),
        Command::Ir {
            diag_codes,
            no_cache,
            cache_status,
            opt_level,
            file,
        } => match parse_opt_level(&opt_level) {
            Ok(opt) => cmd_ir(&file, diag_codes, no_cache, cache_status, opt),
            Err(e) => {
                eprintln!("xo ir: {e}");
                ExitCode::from(2)
            }
        },
        Command::Run {
            jit,
            diag_codes,
            no_cache,
            cache_status,
            opt_level,
            file,
            args,
        } => match parse_opt_level(&opt_level) {
            Ok(opt) => cmd_run(&file, jit, diag_codes, no_cache, cache_status, opt, &args),
            Err(e) => {
                eprintln!("xo run: {e}");
                ExitCode::from(2)
            }
        },
        Command::Build {
            file,
            output,
            no_cache,
            cache_status,
            opt_level,
        } => match parse_opt_level(&opt_level) {
            Ok(opt) => cmd_build(&file, output.as_deref(), no_cache, cache_status, opt),
            Err(e) => {
                eprintln!("xo build: {e}");
                ExitCode::from(2)
            }
        },
        Command::Test { bench, paths } => cmd_test(bench, &paths),
        Command::Check {
            diag_codes,
            graph,
            no_cache,
            cache_status,
            file,
        } => cmd_check(&file, diag_codes, graph, no_cache, cache_status),
        Command::Get { spec, path, deps } => {
            let mut seen = std::collections::HashSet::new();
            cmd_get(&spec, path.as_deref(), deps, &mut seen)
        }
        Command::Home => {
            println!("XO_HOME={}", xo_home().display());
            println!("packages={}", packages_root().display());
            if let Ok(v) = std::env::var(XO_HOME_ENV) {
                println!("{XO_HOME_ENV}={v}");
            }
            ExitCode::SUCCESS
        }
        Command::Cache { command } => match command {
            CacheCommand::Status { path } => cmd_cache_status(path.as_deref()),
            CacheCommand::Clean { path } => cmd_cache_clean(path.as_deref()),
            CacheCommand::Gc { path: _ } => {
                eprintln!("xo cache gc: not implemented yet (use `xo cache clean`)");
                ExitCode::from(1)
            }
            CacheCommand::Doctor { path } => cmd_cache_doctor(path.as_deref()),
        },
        Command::Index { command } => match command {
            IndexCommand::Scan { roots: _ } => not_implemented("index scan", None),
        },
        Command::Tools { command } => match command {
            ToolsCommand::Grammar { command } => match command {
                GrammarCommand::TreeSitter { output } => cmd_tools_grammar_tree_sitter(&output),
            },
        },
        Command::Lsp => match echo_lsp::run_stdio() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("xo lsp: {e}");
                ExitCode::from(1)
            }
        },
        Command::Repl => repl::run_repl(),
    }
}

fn cmd_lex(path: &Path, kinds: bool, diag_codes: bool) -> ExitCode {
    let mut map = SourceMap::new();
    let id = match map.load(path) {
        Ok(id) => id,
        Err(err) => {
            eprintln!("xo lex: cannot read {}: {err}", path.display());
            return ExitCode::from(1);
        }
    };
    let file = map.get(id).expect("just inserted");
    let lexed = lex(file);

    emit_diags(path, &lexed.diagnostics, diag_codes);

    if kinds {
        print!("{}", format_token_kinds(&lexed.tokens));
    } else {
        print!("{}", format_tokens(file, &lexed.tokens));
    }

    if lexed.diagnostics.error_count() > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn cmd_get(
    spec: &str,
    local_path: Option<&Path>,
    with_deps: bool,
    seen: &mut std::collections::HashSet<String>,
) -> ExitCode {
    let Some(parsed) = PackageSpec::parse(spec) else {
        eprintln!("xo get: invalid package spec `{spec}` (want host/path[@tag|@hash])");
        return ExitCode::from(2);
    };
    let PackageSpec {
        package_id,
        version,
    } = parsed;

    // Cycle / re-entry guard for recursive --deps.
    let guard_key = match &version {
        Some(v) => format!("{package_id}@{v}"),
        None => package_id.clone(),
    };
    if !seen.insert(guard_key.clone()) {
        eprintln!("xo get: skip already visited {guard_key}");
        return ExitCode::SUCCESS;
    }

    // Local directory: explicit --path, or spec that is an existing dir.
    let local = local_path
        .map(Path::to_path_buf)
        .or_else(|| {
            let p = PathBuf::from(spec.split('@').next().unwrap_or(spec));
            if p.is_dir() {
                Some(p)
            } else {
                None
            }
        });

    let dest = match local {
        Some(src) => {
            let id = if local_path.is_some() {
                package_id.clone()
            } else if package_id.contains('/') {
                package_id.clone()
            } else {
                src.canonicalize()
                    .map(|p| p.display().to_string())
                    .unwrap_or(package_id.clone())
            };
            // Local path installs: omit @version → `local` dir (no git `default` alias).
            let ver = version
                .clone()
                .unwrap_or_else(|| echo_resolver::LOCAL_VERSION.to_string());
            match install_local_dir(&id, &ver, &src) {
                Ok(d) => {
                    eprintln!("xo get: installed local {} @ {ver} → {}", id, d.display());
                    d
                }
                Err(e) => {
                    eprintln!("xo get: {e}");
                    return ExitCode::from(1);
                }
            }
        }
        None => {
            // Git: omit @version → pin to full commit hash; @tag / @hash allowed.
            match install_git(&package_id, version.as_deref()) {
                Ok(d) => {
                    let ver_label = d
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "?".into());
                    eprintln!(
                        "xo get: cloned {} @ {ver_label} → {}",
                        package_id,
                        d.display()
                    );
                    d
                }
                Err(e) => {
                    eprintln!("xo get: {e}");
                    return ExitCode::from(1);
                }
            }
        }
    };

    if with_deps {
        let toml_path = dest.join("xo.toml");
        if toml_path.is_file() {
            match XoToml::load(&toml_path) {
                Ok(manifest) => {
                    for (dep_id, dep_ver) in &manifest.dependencies {
                        let dep_spec = format!("{dep_id}@{dep_ver}");
                        eprintln!("xo get: dependency {dep_spec}");
                        // Recurse: also pull transitive deps from each dep's xo.toml.
                        let code = cmd_get(&dep_spec, None, true, seen);
                        if code != ExitCode::SUCCESS {
                            return code;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("xo get: xo.toml: {e}");
                    return ExitCode::from(1);
                }
            }
        }
    }

    ExitCode::SUCCESS
}

fn cmd_fmt(path: &Path, write: bool, check: bool) -> ExitCode {
    use echo_parser::format_source;

    if write && check {
        eprintln!("xo fmt: --write and --check are mutually exclusive");
        return ExitCode::from(2);
    }

    let mut map = SourceMap::new();
    let id = match map.load(path) {
        Ok(id) => id,
        Err(err) => {
            eprintln!("xo fmt: cannot read {}: {err}", path.display());
            return ExitCode::from(1);
        }
    };
    let source = map.get(id).expect("just inserted");
    let original = source.text().to_string();
    match format_source(source) {
        Ok(text) => {
            if check {
                if text == original {
                    return ExitCode::SUCCESS;
                }
                eprintln!("xo fmt: {} is not formatted", path.display());
                return ExitCode::from(1);
            }
            if write {
                if text != original {
                    if let Err(err) = std::fs::write(path, &text) {
                        eprintln!("xo fmt: cannot write {}: {err}", path.display());
                        return ExitCode::from(1);
                    }
                }
            } else {
                print!("{text}");
            }
            ExitCode::SUCCESS
        }
        Err(diags) => {
            for d in diags.items() {
                let loc = d
                    .span
                    .map(|sp| {
                        let sf = map.get(sp.source).unwrap_or(source);
                        sf.line_map().format_span_location(path, sp)
                    })
                    .unwrap_or_else(|| path.display().to_string());
                let code = d.code.as_deref().unwrap_or("fmt-error");
                eprintln!("error[{code}] {loc}: {}", d.message);
            }
            eprintln!("xo fmt: failed (parse errors)");
            ExitCode::from(1)
        }
    }
}

fn cmd_ast(path: &Path, kinds: bool, diag_codes: bool) -> ExitCode {
    let mut map = SourceMap::new();
    let id = match map.load(path) {
        Ok(id) => id,
        Err(err) => {
            eprintln!("xo ast: cannot read {}: {err}", path.display());
            return ExitCode::from(1);
        }
    };
    let source = map.get(id).expect("just inserted");
    let parsed = parse(source);

    emit_diags(path, &parsed.diagnostics, diag_codes);

    match &parsed.file {
        Some(ast) => {
            if kinds {
                print!("{}", format_ast_kinds(ast));
            } else {
                print!("{ast:#?}\n");
            }
        }
        None => {
            if !diag_codes {
                eprintln!("xo ast: no AST produced");
            }
        }
    }

    if parsed.diagnostics.error_count() > 0 || parsed.file.is_none() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn cmd_check(
    path: &Path,
    diag_codes: bool,
    show_graph: bool,
    no_cache: bool,
    show_cache_status: bool,
) -> ExitCode {
    let product = analyze(
        path,
        &AnalyzeOptions {
            use_cache: !no_cache,
            ..Default::default()
        },
    );

    if show_cache_status {
        let label = match product.check_cache {
            CheckCacheOutcome::Bypass => "bypass",
            CheckCacheOutcome::Hit => "hit",
            CheckCacheOutcome::Miss => "miss",
            CheckCacheOutcome::StoreError => "store-error",
        };
        eprintln!("check cache: {label}");
        eprintln!(
            "parse cache: hits={} misses={} bypasses={}",
            product.parse_cache.hits, product.parse_cache.misses, product.parse_cache.bypasses
        );
    }

    if show_graph {
        for m in &product.modules {
            println!("{}", m.path.display());
        }
    }

    if diag_codes {
        eprint!("{}", format_check_diag_codes(&product.diagnostics));
    } else {
        emit_diags(path, &product.diagnostics, false);
    }

    if product.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// Shared front-end + lower to LLVM IR via `echo_pipeline`.
struct CompiledIr {
    ir: String,
    diagnostics: Diagnostics,
    check_cache: CheckCacheOutcome,
    parse_cache: echo_resolver::ResolveParseCacheStats,
    ir_cache: IrCacheOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IrCacheOutcome {
    Bypass,
    Hit,
    Miss,
    /// Check failed; IR not attempted.
    Skipped,
}

fn parse_opt_level(s: &str) -> Result<OptLevel, String> {
    OptLevel::parse(s)
}

fn compile_to_ir(path: &Path, no_cache: bool, opt: OptLevel) -> Result<CompiledIr, String> {
    let result = compile_to_llvm_with(
        path,
        &AnalyzeOptions {
            use_cache: !no_cache,
            ..Default::default()
        },
        opt,
    );

    let ir_cache = if !result.analysis.is_ok() {
        IrCacheOutcome::Skipped
    } else if result.ir_cache_hit {
        IrCacheOutcome::Hit
    } else if no_cache {
        IrCacheOutcome::Bypass
    } else if result.ir.is_empty() {
        IrCacheOutcome::Miss
    } else {
        IrCacheOutcome::Miss
    };

    Ok(CompiledIr {
        ir: result.ir,
        diagnostics: result.diagnostics,
        check_cache: result.analysis.check_cache,
        parse_cache: result.analysis.parse_cache,
        ir_cache,
    })
}

fn emit_compile_cache_status(c: &CompiledIr) {
    let check = match c.check_cache {
        CheckCacheOutcome::Bypass => "bypass",
        CheckCacheOutcome::Hit => "hit",
        CheckCacheOutcome::Miss => "miss",
        CheckCacheOutcome::StoreError => "store-error",
    };
    let ir = match c.ir_cache {
        IrCacheOutcome::Bypass => "bypass",
        IrCacheOutcome::Hit => "hit",
        IrCacheOutcome::Miss => "miss",
        IrCacheOutcome::Skipped => "skipped",
    };
    eprintln!("check cache: {check}");
    eprintln!(
        "parse cache: hits={} misses={} bypasses={}",
        c.parse_cache.hits, c.parse_cache.misses, c.parse_cache.bypasses
    );
    eprintln!("codegen cache: {ir}");
}

fn cmd_ir(
    path: &Path,
    diag_codes: bool,
    no_cache: bool,
    cache_status: bool,
    opt: OptLevel,
) -> ExitCode {
    let compiled = match compile_to_ir(path, no_cache, opt) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("xo ir: {err}");
            return ExitCode::from(2);
        }
    };

    if cache_status {
        emit_compile_cache_status(&compiled);
    }
    emit_diags(path, &compiled.diagnostics, diag_codes);
    if compiled.diagnostics.error_count() > 0 {
        return ExitCode::from(1);
    }

    print!("{}", compiled.ir);
    ExitCode::SUCCESS
}

fn cmd_run(
    path: &Path,
    jit: bool,
    diag_codes: bool,
    no_cache: bool,
    cache_status: bool,
    opt: OptLevel,
    args: &[String],
) -> ExitCode {
    cmd_run_inner(path, jit, diag_codes, no_cache, cache_status, opt, args, false, false)
}

fn cmd_run_inner(
    path: &Path,
    jit: bool,
    diag_codes: bool,
    no_cache: bool,
    cache_status: bool,
    opt: OptLevel,
    args: &[String],
    suite: bool,
    bench: bool,
) -> ExitCode {
    let compiled = match compile_to_ir(path, no_cache, opt) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("xo run: {err}");
            return ExitCode::from(2);
        }
    };

    if cache_status {
        emit_compile_cache_status(&compiled);
    }
    emit_diags(path, &compiled.diagnostics, diag_codes);
    if compiled.diagnostics.error_count() > 0 {
        return ExitCode::from(1);
    }

    if jit {
        if !args.is_empty() {
            eprintln!("xo run --jit: program args not supported yet (ignored)");
        }
        if suite {
            if bench {
                echo_runtime::echo_runtime_test_enable_bench();
            } else {
                echo_runtime::echo_runtime_test_enable();
            }
        }
        return match echo_codegen::run_jit_ir(&compiled.ir) {
            Ok(status) => {
                // Match AOT: process status is low 8 bits of echo_entry.
                ExitCode::from(status as u8)
            }
            Err(err) => {
                eprintln!("xo run --jit: {err}");
                ExitCode::from(2)
            }
        };
    }

    let work = match temp_work_dir() {
        Ok(p) => p,
        Err(err) => {
            eprintln!("xo run: {err}");
            return ExitCode::from(2);
        }
    };

    let binary_path = work.join("echo_prog");
    let aot_cache_label = match materialize_aot_binary(
        &compiled.ir,
        opt,
        no_cache,
        path,
        &work,
        "echo_prog",
        &binary_path,
    ) {
        Ok(label) => label,
        Err(err) => {
            eprintln!("xo run: {err}");
            let _ = std::fs::remove_dir_all(&work);
            return ExitCode::from(2);
        }
    };

    if cache_status {
        eprintln!("aot cache: {aot_cache_label}");
    }

    let mut cmd = ProcessCommand::new(&binary_path);
    cmd.args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if suite {
        cmd.env("XO_TEST", "1");
        if bench {
            cmd.env("XO_BENCH", "1");
        }
    }
    let status = cmd.status();

    let _ = std::fs::remove_dir_all(&work);

    match status {
        Ok(st) => {
            if let Some(code) = st.code() {
                // Propagate child exit status (0–255).
                ExitCode::from(code as u8)
            } else {
                eprintln!("xo run: process terminated by signal");
                ExitCode::from(1)
            }
        }
        Err(err) => {
            eprintln!("xo run: exec failed: {err}");
            ExitCode::from(2)
        }
    }
}

fn cmd_build(
    path: &Path,
    output: Option<&Path>,
    no_cache: bool,
    cache_status: bool,
    opt: OptLevel,
) -> ExitCode {
    let compiled = match compile_to_ir(path, no_cache, opt) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("xo build: {err}");
            return ExitCode::from(2);
        }
    };

    if cache_status {
        emit_compile_cache_status(&compiled);
    }
    emit_diags(path, &compiled.diagnostics, false);
    if compiled.diagnostics.error_count() > 0 {
        return ExitCode::from(1);
    }

    let out = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("a.out"));
    let parent = match out.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(p) => p.to_path_buf(),
        None => PathBuf::from("."),
    };
    let name = out.file_name().and_then(|s| s.to_str()).unwrap_or("a.out");
    // Link in a sibling temp dir, then place the binary at `-o` (cache may skip clang).
    let build_dir = parent.join(format!(".echo-build-{}", process_stamp()));

    let aot_cache_label = match materialize_aot_binary(
        &compiled.ir,
        opt,
        no_cache,
        path,
        &build_dir,
        name,
        &out,
    ) {
        Ok(label) => label,
        Err(err) => {
            eprintln!("xo build: {err}");
            let _ = std::fs::remove_dir_all(&build_dir);
            return ExitCode::from(2);
        }
    };
    let _ = std::fs::remove_dir_all(&build_dir);

    if cache_status {
        eprintln!("aot cache: {aot_cache_label}");
    }
    ExitCode::SUCCESS
}

/// Link or restore a native binary for the given post-opt IR.
///
/// Uses the same [`echo_codegen::aot_binary_cache_key_with_opt`] as `xo run`.
/// On a cache hit, writes cached bytes to `dest_path`. On a miss, links into
/// `link_dir` / `link_name`, stores the binary, then places it at `dest_path`.
///
/// Returns the cache outcome label: `hit`, `miss`, or `bypass`.
fn materialize_aot_binary(
    ir: &str,
    opt: OptLevel,
    no_cache: bool,
    entry: &Path,
    link_dir: &Path,
    link_name: &str,
    dest_path: &Path,
) -> Result<&'static str, String> {
    let aot_key = echo_codegen::aot_binary_cache_key_with_opt(ir, opt);
    let store = if no_cache {
        None
    } else {
        Some(ArtifactStore::new(CacheLayout::for_project(
            project_root_for(entry),
        )))
    };

    if let Some(ref s) = store {
        if let Ok(Some(bytes)) = s.get(&aot_key) {
            write_executable(dest_path, &bytes)
                .map_err(|e| format!("write {}: {e}", dest_path.display()))?;
            return Ok("hit");
        }
    }

    let artifact = echo_codegen::link_aot(ir, link_dir, link_name)
        .map_err(|e| format!("link failed: {e}"))?;

    if let Some(ref s) = store {
        if let Ok(bytes) = std::fs::read(&artifact.binary) {
            let _ = s.put(&aot_key, &bytes);
        }
    }

    if artifact.binary != dest_path {
        if let Err(err) = std::fs::rename(&artifact.binary, dest_path) {
            std::fs::copy(&artifact.binary, dest_path).map_err(|err2| {
                format!("write {}: {err} / {err2}", dest_path.display())
            })?;
            #[cfg(unix)]
            set_executable(dest_path);
        }
    }

    Ok(if store.is_some() { "miss" } else { "bypass" })
}

fn write_executable(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, bytes)?;
    #[cfg(unix)]
    set_executable(path);
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755));
}

fn temp_work_dir() -> Result<PathBuf, String> {
    let base = std::env::temp_dir();
    let dir = base.join(format!("echo-run-{}", process_stamp()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("temp dir: {e}"))?;
    Ok(dir)
}

fn process_stamp() -> String {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{t}-{}", std::process::id())
}

fn emit_diags(path: &Path, diagnostics: &Diagnostics, codes_only: bool) {
    if codes_only {
        eprint!("{}", format_diag_codes(diagnostics));
        return;
    }
    // Prefer line:col via LineMap when the file is readable.
    let line_map = std::fs::read_to_string(path)
        .ok()
        .map(|t| echo_source::LineMap::from_text(&t));
    for d in diagnostics.items() {
        let sev = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        };
        let code = d.code.as_deref().unwrap_or("-");
        if let Some(span) = d.span {
            if let Some(ref lm) = line_map {
                eprintln!(
                    "{sev}[{code}] {}: {}",
                    lm.format_span_location(path, span),
                    d.message
                );
            } else {
                eprintln!(
                    "{sev}[{code}] {}:{}-{}: {}",
                    path.display(),
                    span.start.0,
                    span.end.0,
                    d.message
                );
            }
        } else {
            eprintln!("{sev}[{code}] {}: {}", path.display(), d.message);
        }
    }
}

fn cache_layout_for(path: Option<&Path>) -> CacheLayout {
    let root = match path {
        Some(p) => project_root_for(p),
        None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };
    CacheLayout::for_project(root)
}

fn cmd_cache_status(path: Option<&Path>) -> ExitCode {
    let layout = cache_layout_for(path);
    println!("cache root: {}", layout.root().display());
    println!("exists: {}", layout.exists());
    println!("cache_format: {CACHE_FORMAT_VERSION}");
    if !layout.exists() {
        println!("(no cache yet; pipeline does not write artifacts in this milestone)");
        return ExitCode::SUCCESS;
    }
    let store = ArtifactStore::new(layout);
    match store.phase_counts() {
        Ok(counts) => {
            println!("artifacts by phase:");
            for (phase, n) in counts {
                println!("  {:12} {n}", phase.name());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("xo cache status: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_cache_clean(path: Option<&Path>) -> ExitCode {
    let layout = cache_layout_for(path);
    match layout.clean() {
        Ok(()) => {
            println!("cleaned {}", layout.root().display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("xo cache clean: {e}");
            ExitCode::from(1)
        }
    }
}

fn cmd_cache_doctor(path: Option<&Path>) -> ExitCode {
    let layout = cache_layout_for(path);
    println!("cache root: {}", layout.root().display());
    println!("exists: {}", layout.exists());
    println!("cache_format: {CACHE_FORMAT_VERSION}");
    println!("phase fingerprints (no extra inputs):");
    for phase in ArtifactPhase::ALL {
        let pf = phase_fingerprint(phase, &[]);
        println!("  {:12} {}", phase.name(), pf.fingerprint.as_str());
    }
    if layout.exists() {
        let store = ArtifactStore::new(layout);
        match store.phase_counts() {
            Ok(counts) => {
                let total: usize = counts.iter().map(|(_, n)| n).sum();
                println!("total artifacts: {total}");
            }
            Err(e) => {
                eprintln!("xo cache doctor: read error: {e}");
                return ExitCode::from(1);
            }
        }
    } else {
        println!("note: layout not created until a phase writes or `ensure` is called");
    }
    println!("ok");
    ExitCode::SUCCESS
}

fn cmd_tools_grammar_tree_sitter(output: &Path) -> ExitCode {
    match echo_syntax::write_tree_sitter_grammar(output) {
        Ok(()) => {
            let n = echo_syntax::tree_sitter_package_files().len();
            let leaders = echo_syntax::LEADERS.len();
            println!(
                "wrote tree-sitter-echo package to {} ({n} files, {leaders} leaders from echo_syntax)",
                output.display()
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!(
                "xo tools grammar tree-sitter: cannot write {}: {err}",
                output.display()
            );
            ExitCode::from(1)
        }
    }
}

/// Discover and run Echo suite entries (`XO_TEST=1`, Model A registration).
///
/// When `bench` is true, also sets `XO_BENCH=1` so only `test.bench` cases run.
fn cmd_test(bench: bool, paths: &[String]) -> ExitCode {
    use std::time::{Duration, Instant};

    fn fmt_dur(d: Duration) -> String {
        let ms = d.as_secs_f64() * 1000.0;
        if ms < 1000.0 {
            format!("{ms:.1}ms")
        } else {
            format!("{:.2}s", d.as_secs_f64())
        }
    }

    let label = if bench { "xo test --bench" } else { "xo test" };

    let files = match collect_test_files(paths) {
        Ok(f) if f.is_empty() => {
            eprintln!("{label}: no test files matched");
            return ExitCode::from(1);
        }
        Ok(f) => f,
        Err(e) => {
            eprintln!("{label}: {e}");
            return ExitCode::from(2);
        }
    };

    let mut failed_files = 0usize;
    let total_files = files.len();
    let all_start = Instant::now();
    for (i, file) in files.iter().enumerate() {
        eprintln!("{label} [{}/{}] {}", i + 1, total_files, file.display());
        let file_start = Instant::now();
        let code = run_suite_file(file, bench);
        let file_elapsed = fmt_dur(file_start.elapsed());
        if code != 0 {
            failed_files += 1;
            eprintln!("{label}: FAILED {} ({file_elapsed})", file.display());
        } else {
            eprintln!("{label}: ok {} ({file_elapsed})", file.display());
        }
    }

    let total_elapsed = fmt_dur(all_start.elapsed());
    if failed_files > 0 {
        eprintln!(
            "{label}: {failed_files} of {total_files} file(s) failed ({total_elapsed})"
        );
        ExitCode::from(1)
    } else {
        eprintln!("{label}: {total_files} file(s) passed ({total_elapsed})");
        ExitCode::SUCCESS
    }
}

fn run_suite_file(path: &Path, bench: bool) -> u8 {
    // Suite mode: AOT child gets XO_TEST=1 (+ XO_BENCH when benchmarking).
    let code = cmd_run_inner(
        path,
        false,
        false,
        false,
        false,
        OptLevel::O0,
        &[],
        true,
        bench,
    );
    if code == ExitCode::SUCCESS {
        0
    } else {
        1
    }
}

fn collect_test_files(paths: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    if paths.is_empty() {
        collect_from_dir(Path::new("."), &mut out)?;
    } else {
        for raw in paths {
            collect_one_pattern(raw, &mut out)?;
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn collect_one_pattern(raw: &str, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let p = Path::new(raw);
    if looks_like_glob(raw) {
        return expand_glob(raw, out);
    }
    if p.is_file() {
        if is_echo_file(p) {
            out.push(p.to_path_buf());
        } else {
            return Err(format!("not an .echo file: {}", p.display()));
        }
        return Ok(());
    }
    if p.is_dir() {
        return collect_from_dir(p, out);
    }
    Err(format!("path not found: {raw}"))
}

fn looks_like_glob(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

fn expand_glob(pattern: &str, out: &mut Vec<PathBuf>) -> Result<(), String> {
    // Minimal glob: `**` = any path segment sequence; `*` = within one segment.
    let root = Path::new(".");
    walk_glob(root, pattern, out)
}

fn walk_glob(dir: &Path, pattern: &str, out: &mut Vec<PathBuf>) -> Result<(), String> {
    // Normalize pattern to path components.
    let parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    walk_glob_parts(dir, &parts, out)
}

fn walk_glob_parts(dir: &Path, parts: &[&str], out: &mut Vec<PathBuf>) -> Result<(), String> {
    if parts.is_empty() {
        return Ok(());
    }
    let head = parts[0];
    let rest = &parts[1..];
    if head == "**" {
        // Match zero or more directories.
        walk_glob_parts(dir, rest, out)?;
        if let Ok(rd) = std::fs::read_dir(dir) {
            for ent in rd.flatten() {
                let p = ent.path();
                if p.is_dir() {
                    walk_glob_parts(&p, parts, out)?;
                }
            }
        }
        return Ok(());
    }
    if rest.is_empty() {
        // Final component: match files in dir.
        if let Ok(rd) = std::fs::read_dir(dir) {
            for ent in rd.flatten() {
                let p = ent.path();
                let name = ent.file_name().to_string_lossy().into_owned();
                if p.is_file() && is_echo_file(&p) && glob_match(head, &name) {
                    out.push(p);
                }
            }
        }
        return Ok(());
    }
    // Intermediate directory segment.
    if let Ok(rd) = std::fs::read_dir(dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            let name = ent.file_name().to_string_lossy().into_owned();
            if p.is_dir() && glob_match(head, &name) {
                walk_glob_parts(&p, rest, out)?;
            }
        }
    }
    Ok(())
}

fn glob_match(pat: &str, name: &str) -> bool {
    if pat == "*" {
        return true;
    }
    // Single `*` wildcards only (no `?` classes for v0 beyond literal).
    let mut pi = 0;
    let mut ni = 0;
    let pb = pat.as_bytes();
    let nb = name.as_bytes();
    let mut star = None::<(usize, usize)>;
    while ni < nb.len() {
        if pi < pb.len() && (pb[pi] == nb[ni] || pb[pi] == b'?') {
            pi += 1;
            ni += 1;
        } else if pi < pb.len() && pb[pi] == b'*' {
            star = Some((pi, ni));
            pi += 1;
        } else if let Some((sp, sn)) = star {
            pi = sp + 1;
            ni = sn + 1;
            star = Some((sp, ni));
        } else {
            return false;
        }
    }
    while pi < pb.len() && pb[pi] == b'*' {
        pi += 1;
    }
    pi == pb.len()
}

fn collect_from_dir(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    // Convention: `*_test.echo` anywhere under dir, and everything under `tests/`.
    fn walk(dir: &Path, under_tests: bool, out: &mut Vec<PathBuf>) -> Result<(), String> {
        let rd = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for ent in rd.flatten() {
            let p = ent.path();
            let name = ent.file_name().to_string_lossy().into_owned();
            if name == ".xo" || name == "target" || name == "node_modules" {
                continue;
            }
            if p.is_dir() {
                let next_tests = under_tests || name == "tests";
                walk(&p, next_tests, out)?;
            } else if p.is_file() && is_echo_file(&p) {
                if under_tests || name.ends_with("_test.echo") {
                    out.push(p);
                }
            }
        }
        Ok(())
    }
    walk(dir, dir.ends_with("tests"), out)
}

fn is_echo_file(p: &Path) -> bool {
    p.extension().and_then(|e| e.to_str()) == Some("echo")
}

fn not_implemented(command: &str, path: Option<&Path>) -> ExitCode {
    eprint!("xo {command}: not implemented yet");
    if let Some(path) = path {
        eprint!(" ({})", path.display());
    }
    eprintln!();
    let _ = (
        echo_source::crate_name(),
        echo_diagnostics::crate_name(),
        echo_syntax::crate_name(),
        echo_lexer::crate_name(),
        echo_ast::crate_name(),
        echo_parser::crate_name(),
        echo_semantics::crate_name(),
        echo_hir::crate_name(),
        echo_mir::crate_name(),
        echo_codegen::crate_name(),
        echo_codegen_abi::crate_name(),
        echo_runtime::crate_name(),
        echo_std::crate_name(),
        echo_index::crate_name(),
        echo_resolver::crate_name(),
        echo_fingerprint::crate_name(),
        echo_cache::crate_name(),
        echo_build::crate_name(),
        echo_reflection::crate_name(),
    );
    ExitCode::from(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn ir_opt(args: &[&str]) -> Result<OptLevel, String> {
        let mut full = vec!["xo", "ir"];
        full.extend_from_slice(args);
        full.push("t.echo");
        let cli = Cli::try_parse_from(full).map_err(|e| e.to_string())?;
        match cli.command {
            Command::Ir { opt_level, .. } => parse_opt_level(&opt_level),
            other => Err(format!("expected Ir, got {other:?}")),
        }
    }

    fn run_opt(args: &[&str]) -> Result<OptLevel, String> {
        let mut full = vec!["xo", "run"];
        full.extend_from_slice(args);
        full.push("t.echo");
        let cli = Cli::try_parse_from(full).map_err(|e| e.to_string())?;
        match cli.command {
            Command::Run { opt_level, .. } => parse_opt_level(&opt_level),
            other => Err(format!("expected Run, got {other:?}")),
        }
    }

    fn build_opt(args: &[&str]) -> Result<OptLevel, String> {
        let mut full = vec!["xo", "build"];
        full.extend_from_slice(args);
        full.push("t.echo");
        let cli = Cli::try_parse_from(full).map_err(|e| e.to_string())?;
        match cli.command {
            Command::Build { opt_level, .. } => parse_opt_level(&opt_level),
            other => Err(format!("expected Build, got {other:?}")),
        }
    }

    #[test]
    fn default_opt_is_o0_for_ir_run_build() {
        assert_eq!(ir_opt(&[]).unwrap(), OptLevel::O0);
        assert_eq!(run_opt(&[]).unwrap(), OptLevel::O0);
        assert_eq!(build_opt(&[]).unwrap(), OptLevel::O0);
    }

    #[test]
    fn short_and_long_opt_flags_parse_all_levels() {
        for (flag, expected) in [
            ("-O0", OptLevel::O0),
            ("-O1", OptLevel::O1),
            ("-O2", OptLevel::O2),
            ("-O3", OptLevel::O3),
            ("-Oz", OptLevel::Oz),
        ] {
            assert_eq!(ir_opt(&[flag]).unwrap(), expected, "ir {flag}");
            assert_eq!(run_opt(&[flag]).unwrap(), expected, "run {flag}");
            assert_eq!(build_opt(&[flag]).unwrap(), expected, "build {flag}");
        }
        assert_eq!(ir_opt(&["--opt-level", "O2"]).unwrap(), OptLevel::O2);
        assert_eq!(run_opt(&["--opt-level", "z"]).unwrap(), OptLevel::Oz);
    }

    #[test]
    fn invalid_opt_level_fails_clearly() {
        let err = parse_opt_level("9").unwrap_err();
        assert!(err.contains("unknown opt level"), "{err}");
        assert!(err.contains("O0"), "{err}");
        assert!(parse_opt_level("fast").is_err());
    }

    #[test]
    fn ir_run_build_share_opt_semantics() {
        // Same parse path → same OptLevel for every host command.
        for level in ["0", "1", "2", "3", "z", "O0", "O2", "Oz"] {
            let expected = OptLevel::parse(level).unwrap();
            assert_eq!(parse_opt_level(level).unwrap(), expected);
        }
    }

    #[test]
    fn aot_cache_key_shared_by_run_and_build() {
        // Same IR + opt → same key (build/run must not diverge).
        let ir = "; ModuleID = 'echo'\ndefine i64 @echo_entry() { ret i64 0 }\n";
        let k_run = echo_codegen::aot_binary_cache_key_with_opt(ir, OptLevel::O2);
        let k_build = echo_codegen::aot_binary_cache_key_with_opt(ir, OptLevel::O2);
        assert_eq!(k_run.blob_name(), k_build.blob_name());
        assert_ne!(
            echo_codegen::aot_binary_cache_key_with_opt(ir, OptLevel::O0).blob_name(),
            k_run.blob_name()
        );
    }
}
