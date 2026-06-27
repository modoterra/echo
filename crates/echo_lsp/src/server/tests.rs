use echo_index::{DependencyKind, EchoFileMode, FileId, ReferenceKind, SymbolKind, TextRange};
use ropey::Rope;

use crate::position::range_to_lsp_range;

use super::*;

#[test]
fn parsed_document_produces_index_facts() {
    let facts = parse_index_facts(
        r#"<?php
namespace App\Http;
use Psr\Log\LoggerInterface as Logger;
fn handler(): string {
    echo "ok";
}
class UserController {
    function show(): string;
}
"#,
        FileId(3),
        EchoFileMode::PhpCompat,
        None,
    )
    .expect("source parses");

    assert_eq!(
        facts
            .declarations
            .iter()
            .map(|symbol| (symbol.name.text.as_str(), symbol.kind))
            .collect::<Vec<_>>(),
        vec![
            ("App\\Http", SymbolKind::Namespace),
            ("handler", SymbolKind::Function),
            ("UserController", SymbolKind::Class),
            ("show", SymbolKind::Method),
        ]
    );
    assert_eq!(facts.dependencies.len(), 1);
    assert_eq!(facts.dependencies[0].kind, DependencyKind::PhpUse);
    assert_eq!(facts.dependencies[0].target, "Psr\\Log\\LoggerInterface");
    assert_eq!(facts.dependencies[0].alias.as_deref(), Some("Logger"));
}

#[test]
fn echo_module_server_surface_produces_index_facts() {
    let facts = parse_index_facts(
        r#"module acme.http_server.runtime

use std.http
use std.net
use acme.http_server.ServerKernel

let $app = require_once config("server.paths.bootstrap")
let $kernel = ServerKernel.from($app)
let $address = config("echo.server.host", "127.0.0.1") . ":" . config("echo.server.port", 8080)
let $server = net.listen($address)

loop {
    let $connection = net.accept($server)
    let $request = http.readRequest($connection)
    let $response = $kernel.handle($request)

    net.write($connection, http.toBytes($response))
    net.close($connection)
}
"#,
        FileId(9),
        EchoFileMode::Echo,
        None,
    )
    .expect("Echo module server source parses for LSP indexing");

    assert!(facts.declarations.iter().any(|symbol| {
        symbol.kind == SymbolKind::Namespace
            && symbol.name.text.as_str() == "acme\\http_server\\runtime"
    }));
    assert!(facts.dependencies.iter().any(|dependency| {
        dependency.kind == DependencyKind::EchoStdImport && dependency.target == "http"
    }));
    assert!(facts.dependencies.iter().any(|dependency| {
        dependency.kind == DependencyKind::EchoStdImport && dependency.target == "net"
    }));
}

#[test]
fn echo_package_provider_surface_produces_index_facts() {
    let facts = parse_index_facts(
        r#"class ReportFormatter {
    fn slug($name): string {
        return $name
    }

    pub fn title($name): string {
        return $this.slug($name)
    }
}
"#,
        FileId(10),
        EchoFileMode::Echo,
        None,
    )
    .expect("Echo package provider source parses for LSP indexing");

    assert!(
        facts
            .declarations
            .iter()
            .any(|symbol| symbol.kind == SymbolKind::Class
                && symbol.name.text.as_str() == "ReportFormatter")
    );
}

#[test]
fn laravel_entrypoint_produces_import_and_static_reference_facts() {
    let source = r#"<?php

use Illuminate\Foundation\Application;
use Illuminate\Http\Request;

define('LARAVEL_START', microtime(true));

if (file_exists($maintenance = __DIR__.'/../storage/framework/maintenance.php')) {
    require $maintenance;
}

require __DIR__.'/../vendor/autoload.php';

/** @var Application $app */
$app = require_once __DIR__.'/../bootstrap/app.php';

$app->handleRequest(Request::capture());
"#;
    let facts = parse_index_facts(
        source,
        FileId(4),
        EchoFileMode::PhpCompat,
        Some(Path::new("/project/public")),
    )
    .expect("source parses");

    assert!(facts.dependencies.iter().any(|dependency| {
        dependency.kind == DependencyKind::PhpUse
            && dependency.target == "Illuminate\\Http\\Request"
    }));
    let use_request_start = source
        .find("Illuminate\\Http\\Request")
        .expect("use request");
    assert!(facts.dependencies.iter().any(|dependency| {
        dependency.kind == DependencyKind::PhpUse
            && dependency.target == "Illuminate\\Http\\Request"
            && dependency.target_range
                == TextRange::new(
                    use_request_start as u32,
                    (use_request_start + "Illuminate\\Http\\Request".len()) as u32,
                )
    }));
    let phpdoc_application_start = source.find("@var Application").expect("phpdoc app") + 5;
    assert!(facts.references.iter().any(|reference| {
        reference.kind == ReferenceKind::ClassLike
            && reference.name == "Application"
            && reference.range
                == TextRange::new(
                    phpdoc_application_start as u32,
                    (phpdoc_application_start + "Application".len()) as u32,
                )
    }));
    let autoload_start = source
        .find("__DIR__.'/../vendor/autoload.php'")
        .expect("autoload");
    assert!(facts.references.iter().any(|reference| {
        reference.kind == ReferenceKind::FilePath
            && reference.name == "/project/public/../vendor/autoload.php"
            && reference.range
                == TextRange::new(
                    autoload_start as u32,
                    (autoload_start + "__DIR__.'/../vendor/autoload.php'".len()) as u32,
                )
    }));
    let request_call_start = source.find("Request::capture").expect("static call");
    assert!(facts.references.iter().any(|reference| {
        reference.kind == ReferenceKind::ClassLike
            && reference.name == "Request"
            && reference.range
                == TextRange::new(
                    request_call_start as u32,
                    (request_call_start + "Request".len()) as u32,
                )
    }));
    assert!(facts.references.iter().any(|reference| {
        reference.kind == ReferenceKind::StaticMethod
            && reference.name == "capture"
            && reference.qualifier.as_deref() == Some("Request")
    }));
    assert!(facts.references.iter().any(|reference| {
        reference.kind == ReferenceKind::Method
            && reference.name == "handleRequest"
            && reference.qualifier.as_deref() == Some("app")
    }));
    assert!(facts.declarations.iter().any(|symbol| {
        symbol.kind == SymbolKind::LocalVariable
            && symbol.name.text == "app"
            && symbol
                .signature
                .as_ref()
                .is_some_and(|signature| signature.text == "Application")
    }));
}

#[test]
fn indexes_required_php_files_from_laravel_fixture() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/php/112_laravel_bootstrap")
        .canonicalize()
        .expect("fixture root");
    let public_index = fixture_root.join("public/index.php");
    let public_source = std::fs::read_to_string(&public_index).expect("public source");

    let mut index = EchoIndex::new();
    let file_id = index.alloc_file_id();
    index.insert_file(IndexedFile {
        file_id,
        uri: Uri::from_file_path(&public_index).unwrap().to_string(),
        path: Some(public_index.clone()),
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });

    let facts = parse_index_facts(
        &public_source,
        file_id,
        EchoFileMode::PhpCompat,
        public_index.parent(),
    )
    .expect("public source parses");
    index.update_file(file_id, facts);
    index_required_files(&mut index, file_id);

    let method = index
        .method_definition("Illuminate\\Foundation\\Application", "handleRequest")
        .expect("vendored handleRequest method");
    let method_file = index.file(method.file_id).expect("method file");

    assert_eq!(
        method_file.path.as_deref(),
        Some(
            fixture_root
                .join("vendor/laravel/framework/src/Illuminate/Foundation/Application.php")
                .as_path()
        )
    );
}

#[test]
fn document_links_cover_whole_dir_path_expression() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/php/112_laravel_bootstrap")
        .canonicalize()
        .expect("fixture root");
    let public_index = fixture_root.join("public/index.php");
    let public_source = std::fs::read_to_string(&public_index).expect("public source");
    let autoload_expr = "__DIR__.'/../vendor/autoload.php'";
    let autoload_start = public_source.find(autoload_expr).expect("autoload expr");
    let maintenance_expr = "__DIR__.'/../storage/framework/maintenance.php'";
    let maintenance_start = public_source
        .find(maintenance_expr)
        .expect("maintenance expr");
    let mut index = EchoIndex::new();
    let file_id = index.alloc_file_id();
    index.insert_file(IndexedFile {
        file_id,
        uri: Uri::from_file_path(&public_index).unwrap().to_string(),
        path: Some(public_index.clone()),
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    let facts = parse_index_facts(
        &public_source,
        file_id,
        EchoFileMode::PhpCompat,
        public_index.parent(),
    )
    .expect("public source parses");
    index.update_file(file_id, facts);

    let dependencies = index.dependencies(DependencyQuery::in_file(file_id));
    let references = index.references(ReferenceQuery::in_file(file_id));
    let links =
        document_links_for_paths(&Rope::from_str(&public_source), &dependencies, &references);
    let link = links
        .iter()
        .find(|link| {
            link.target.as_ref()
                == Some(&Uri::from_file_path(fixture_root.join("vendor/autoload.php")).unwrap())
        })
        .expect("autoload document link");
    let autoload_target_links = links
        .iter()
        .filter(|link| {
            link.target.as_ref()
                == Some(&Uri::from_file_path(fixture_root.join("vendor/autoload.php")).unwrap())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        link.range,
        range_to_lsp_range(
            &Rope::from_str(&public_source),
            TextRange::new(
                autoload_start as u32,
                (autoload_start + autoload_expr.len()) as u32,
            ),
        )
    );
    assert_eq!(autoload_target_links.len(), 1);

    let maintenance_target =
        Uri::from_file_path(fixture_root.join("storage/framework/maintenance.php")).unwrap();
    assert!(
        links
            .iter()
            .all(|link| link.target.as_ref() != Some(&maintenance_target)),
        "missing maintenance file should not produce a document link"
    );

    assert!(references.iter().any(|reference| {
        reference.kind == ReferenceKind::FilePath
            && reference.range
                == TextRange::new(
                    maintenance_start as u32,
                    (maintenance_start + maintenance_expr.len()) as u32,
                )
    }));
}
