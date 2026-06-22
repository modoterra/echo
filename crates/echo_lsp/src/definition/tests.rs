use echo_index::{
    DefinitionLocation, DependencyFact, DependencyKind, EchoFileMode, EchoIndex, FileId, FqName,
    IndexFacts, IndexedFile, ReferenceFact, ReferenceKind, Signature, SymbolFact, SymbolKind,
    SymbolName, TextRange,
};
use tower_lsp_server::ls_types::{Position, Range};

use super::*;

#[test]
fn converts_same_document_dependency_definition_location() {
    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    let uri = "file:///project/public/index.php".parse::<Uri>().unwrap();
    index.insert_file(IndexedFile {
        file_id,
        uri: uri.to_string(),
        path: None,
        version: Some(1),
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });

    let location = definition_location_to_lsp(
        &index,
        &Rope::from_str("<?php\nuse Illuminate\\Http\\Request;\n"),
        file_id,
        &uri,
        DefinitionLocation::Dependency {
            file_id,
            range: TextRange::new(6, 34),
            selection_range: TextRange::new(6, 34),
        },
    )
    .expect("location");

    assert_eq!(location.uri, uri);
    assert_eq!(
        location.range,
        Range {
            start: Position::new(1, 0),
            end: Position::new(1, 28),
        }
    );
}

#[test]
fn resolves_phpdoc_receiver_method_to_indexed_class_method() {
    let mut index = EchoIndex::new();
    let source_file_id = FileId(1);
    let vendor_file_id = FileId(2);
    index.insert_file(IndexedFile {
        file_id: source_file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.insert_file(IndexedFile {
        file_id: vendor_file_id,
        uri: "file:///project/vendor/laravel/framework/src/Illuminate/Foundation/Application.php"
            .to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        vendor_file_id,
        IndexFacts::declarations(
            vendor_file_id,
            EchoFileMode::PhpCompat,
            vec![SymbolFact {
                name: SymbolName::new("handleRequest"),
                fq_name: Some(FqName::new(
                    vec!["Illuminate".into(), "Foundation".into()],
                    "Application::handleRequest",
                )),
                kind: SymbolKind::Method,
                range: TextRange::new(60, 110),
                selection_range: TextRange::new(76, 89),
                visibility: None,
                signature: None,
            }],
        ),
    );
    let app = echo_index::Symbol {
        id: echo_index::SymbolId(1),
        file_id: source_file_id,
        name: SymbolName::new("app"),
        fq_name: None,
        kind: SymbolKind::LocalVariable,
        range: TextRange::new(70, 95),
        selection_range: TextRange::new(91, 95),
        visibility: None,
        container: None,
        signature: Some(Signature {
            text: "Application".to_string(),
        }),
    };
    let dependency = DependencyFact {
        kind: DependencyKind::PhpUse,
        target: "Illuminate\\Foundation\\Application".to_string(),
        alias: None,
        range: TextRange::new(6, 43),
        target_range: TextRange::new(6, 43),
    };

    let definition = method_definition_at(
        &index,
        &Rope::from_str("<?php\n$app->handleRequest(Request::capture());\n"),
        TextOffset(13),
        &[app],
        &[dependency],
    )
    .expect("method definition");

    let DefinitionLocation::Symbol(location) = definition else {
        panic!("expected symbol definition");
    };
    assert_eq!(location.file_id, vendor_file_id);
    assert_eq!(location.selection_range, TextRange::new(76, 89));
}

#[test]
fn resolves_require_dependency_to_target_file() {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/php/112_laravel_bootstrap")
        .canonicalize()
        .expect("fixture root");
    let autoload = fixture_root.join("vendor/autoload.php");
    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    index.insert_file(IndexedFile {
        file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        file_id,
        IndexFacts {
            file_id,
            mode: EchoFileMode::PhpCompat,
            declarations: Vec::new(),
            dependencies: vec![DependencyFact {
                kind: DependencyKind::ComposerAutoload,
                target: autoload.to_string_lossy().to_string(),
                alias: None,
                range: TextRange::new(10, 55),
                target_range: TextRange::new(18, 54),
            }],
            references: Vec::new(),
        },
    );

    let location = dependency_target_location_at(&mut index, file_id, TextOffset(25))
        .expect("autoload target location");

    assert_eq!(location.uri, Uri::from_file_path(&autoload).unwrap());
}

#[test]
fn resolves_php_use_dependency_through_composer_psr4() {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/php/112_laravel_bootstrap")
        .canonicalize()
        .expect("fixture root");
    let autoload = fixture_root.join("vendor/autoload.php");
    let request = fixture_root.join("vendor/laravel/framework/src/Illuminate/Http/Request.php");
    let request_source = std::fs::read_to_string(&request).expect("request source");
    let class_start = request_source.find("Request").expect("request class");
    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    index.insert_file(IndexedFile {
        file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        file_id,
        IndexFacts {
            file_id,
            mode: EchoFileMode::PhpCompat,
            declarations: Vec::new(),
            dependencies: vec![
                DependencyFact {
                    kind: DependencyKind::ComposerAutoload,
                    target: autoload.to_string_lossy().to_string(),
                    alias: None,
                    range: TextRange::new(50, 90),
                    target_range: TextRange::new(58, 89),
                },
                DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: "Illuminate\\Http\\Request".to_string(),
                    alias: None,
                    range: TextRange::new(0, 30),
                    target_range: TextRange::new(4, 29),
                },
            ],
            references: Vec::new(),
        },
    );

    let location = dependency_target_location_at(&mut index, file_id, TextOffset(10))
        .expect("Request target location");

    assert_eq!(location.uri, Uri::from_file_path(&request).unwrap());
    assert_eq!(
        location.range.start,
        range_to_lsp_range(
            &Rope::from_str(&request_source),
            TextRange::new(class_start as u32, class_start as u32)
        )
        .start
    );
}

#[test]
fn resolves_class_reference_through_composer_psr4() {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/php/112_laravel_bootstrap")
        .canonicalize()
        .expect("fixture root");
    let autoload = fixture_root.join("vendor/autoload.php");
    let request = fixture_root.join("vendor/laravel/framework/src/Illuminate/Http/Request.php");
    let request_source = std::fs::read_to_string(&request).expect("request source");
    let class_start = request_source.find("Request").expect("request class");
    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    index.insert_file(IndexedFile {
        file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        file_id,
        IndexFacts {
            file_id,
            mode: EchoFileMode::PhpCompat,
            declarations: Vec::new(),
            dependencies: vec![
                DependencyFact {
                    kind: DependencyKind::ComposerAutoload,
                    target: autoload.to_string_lossy().to_string(),
                    alias: None,
                    range: TextRange::new(0, 20),
                    target_range: TextRange::new(0, 20),
                },
                DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: "Illuminate\\Http\\Request".to_string(),
                    alias: None,
                    range: TextRange::new(30, 60),
                    target_range: TextRange::new(34, 58),
                },
            ],
            references: vec![ReferenceFact {
                kind: ReferenceKind::ClassLike,
                name: "Request".to_string(),
                qualifier: None,
                range: TextRange::new(80, 87),
            }],
        },
    );

    let location = reference_target_location_at(&mut index, file_id, TextOffset(82))
        .expect("Request class location");

    assert_eq!(location.uri, Uri::from_file_path(&request).unwrap());
    assert_eq!(
        location.range.start,
        range_to_lsp_range(
            &Rope::from_str(&request_source),
            TextRange::new(class_start as u32, class_start as u32)
        )
        .start
    );
}

#[test]
fn resolves_class_reference_to_source_line_when_target_php_is_not_parseable() {
    let root = std::env::temp_dir().join(format!("echo-lsp-class-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let composer_dir = root.join("vendor/composer");
    let request_dir = root.join("vendor/laravel/framework/src/Illuminate/Http");
    std::fs::create_dir_all(&composer_dir).expect("composer dir");
    std::fs::create_dir_all(&request_dir).expect("request dir");
    let autoload = root.join("vendor/autoload.php");
    let request = request_dir.join("Request.php");
    std::fs::write(&autoload, "<?php\n").expect("autoload");
    std::fs::write(
            composer_dir.join("autoload_psr4.php"),
            "<?php\n$vendorDir = dirname(__DIR__);\n$baseDir = dirname($vendorDir);\nreturn array(\n    'Illuminate\\\\' => array($vendorDir . '/laravel/framework/src/Illuminate'),\n);\n",
        )
        .expect("autoload psr4");
    std::fs::write(
        &request,
        "<?php\nnamespace Illuminate\\Http;\nfinal class Request\n{\n    use Macroable;\n}\n",
    )
    .expect("request source");

    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    index.insert_file(IndexedFile {
        file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        file_id,
        IndexFacts {
            file_id,
            mode: EchoFileMode::PhpCompat,
            declarations: Vec::new(),
            dependencies: vec![
                DependencyFact {
                    kind: DependencyKind::ComposerAutoload,
                    target: autoload.to_string_lossy().to_string(),
                    alias: None,
                    range: TextRange::new(0, 20),
                    target_range: TextRange::new(0, 20),
                },
                DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: "Illuminate\\Http\\Request".to_string(),
                    alias: None,
                    range: TextRange::new(30, 60),
                    target_range: TextRange::new(34, 58),
                },
            ],
            references: vec![ReferenceFact {
                kind: ReferenceKind::ClassLike,
                name: "Request".to_string(),
                qualifier: None,
                range: TextRange::new(80, 87),
            }],
        },
    );

    let location = reference_target_location_at(&mut index, file_id, TextOffset(82))
        .expect("Request class location");

    assert_eq!(location.uri, Uri::from_file_path(&request).unwrap());
    assert_eq!(location.range.start.line, 2);
    assert_eq!(location.range.start.character, 12);

    std::fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn resolves_static_method_reference_to_composer_class_declaration() {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/php/112_laravel_bootstrap")
        .canonicalize()
        .expect("fixture root");
    let autoload = fixture_root.join("vendor/autoload.php");
    let request = fixture_root.join("vendor/laravel/framework/src/Illuminate/Http/Request.php");
    let request_source = std::fs::read_to_string(&request).expect("request source");
    let capture_start = request_source.find("capture").expect("capture method");
    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    index.insert_file(IndexedFile {
        file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        file_id,
        IndexFacts {
            file_id,
            mode: EchoFileMode::PhpCompat,
            declarations: Vec::new(),
            dependencies: vec![
                DependencyFact {
                    kind: DependencyKind::ComposerAutoload,
                    target: autoload.to_string_lossy().to_string(),
                    alias: None,
                    range: TextRange::new(0, 20),
                    target_range: TextRange::new(0, 20),
                },
                DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: "Illuminate\\Http\\Request".to_string(),
                    alias: None,
                    range: TextRange::new(30, 60),
                    target_range: TextRange::new(34, 58),
                },
            ],
            references: vec![ReferenceFact {
                kind: ReferenceKind::StaticMethod,
                name: "capture".to_string(),
                qualifier: Some("Request".to_string()),
                range: TextRange::new(89, 96),
            }],
        },
    );

    let location = reference_target_location_at(&mut index, file_id, TextOffset(90))
        .expect("capture method location");

    assert_eq!(location.uri, Uri::from_file_path(&request).unwrap());
    assert_eq!(
        location.range.start,
        range_to_lsp_range(
            &Rope::from_str(&request_source),
            TextRange::new(capture_start as u32, capture_start as u32)
        )
        .start
    );
}

#[test]
fn resolves_static_method_reference_to_source_line_when_target_php_is_not_parseable() {
    let root = std::env::temp_dir().join(format!("echo-lsp-static-method-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let composer_dir = root.join("vendor/composer");
    let request_dir = root.join("vendor/laravel/framework/src/Illuminate/Http");
    std::fs::create_dir_all(&composer_dir).expect("composer dir");
    std::fs::create_dir_all(&request_dir).expect("request dir");
    let autoload = root.join("vendor/autoload.php");
    let request = request_dir.join("Request.php");
    std::fs::write(&autoload, "<?php\n").expect("autoload");
    std::fs::write(
            composer_dir.join("autoload_psr4.php"),
            "<?php\n$vendorDir = dirname(__DIR__);\n$baseDir = dirname($vendorDir);\nreturn array(\n    'Illuminate\\\\' => array($vendorDir . '/laravel/framework/src/Illuminate'),\n);\n",
        )
        .expect("autoload psr4");
    let request_source = "<?php\nnamespace Illuminate\\Http;\nclass Request\n{\n    use Macroable;\n\n    public static function capture()\n    {\n    }\n}\n";
    std::fs::write(&request, request_source).expect("request source");

    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    index.insert_file(IndexedFile {
        file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        file_id,
        IndexFacts {
            file_id,
            mode: EchoFileMode::PhpCompat,
            declarations: Vec::new(),
            dependencies: vec![
                DependencyFact {
                    kind: DependencyKind::ComposerAutoload,
                    target: autoload.to_string_lossy().to_string(),
                    alias: None,
                    range: TextRange::new(0, 20),
                    target_range: TextRange::new(0, 20),
                },
                DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: "Illuminate\\Http\\Request".to_string(),
                    alias: None,
                    range: TextRange::new(30, 60),
                    target_range: TextRange::new(34, 58),
                },
            ],
            references: vec![ReferenceFact {
                kind: ReferenceKind::StaticMethod,
                name: "capture".to_string(),
                qualifier: Some("Request".to_string()),
                range: TextRange::new(89, 96),
            }],
        },
    );

    let location = reference_target_location_at(&mut index, file_id, TextOffset(90))
        .expect("capture method location");

    assert_eq!(location.uri, Uri::from_file_path(&request).unwrap());
    assert_eq!(location.range.start.line, 6);
    assert_eq!(location.range.start.character, 27);

    std::fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn resolves_phpdoc_receiver_method_through_composer_psr4() {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/php/112_laravel_bootstrap")
        .canonicalize()
        .expect("fixture root");
    let autoload = fixture_root.join("vendor/autoload.php");
    let application =
        fixture_root.join("vendor/laravel/framework/src/Illuminate/Foundation/Application.php");
    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    index.insert_file(IndexedFile {
        file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        file_id,
        IndexFacts {
            file_id,
            mode: EchoFileMode::PhpCompat,
            declarations: Vec::new(),
            dependencies: vec![
                DependencyFact {
                    kind: DependencyKind::ComposerAutoload,
                    target: autoload.to_string_lossy().to_string(),
                    alias: None,
                    range: TextRange::new(0, 20),
                    target_range: TextRange::new(0, 20),
                },
                DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: "Illuminate\\Foundation\\Application".to_string(),
                    alias: None,
                    range: TextRange::new(30, 75),
                    target_range: TextRange::new(34, 70),
                },
            ],
            references: vec![ReferenceFact {
                kind: ReferenceKind::Method,
                name: "handleRequest".to_string(),
                qualifier: Some("app".to_string()),
                range: TextRange::new(90, 103),
            }],
        },
    );
    let app = echo_index::Symbol {
        id: echo_index::SymbolId(1),
        file_id,
        name: SymbolName::new("app"),
        fq_name: None,
        kind: SymbolKind::LocalVariable,
        range: TextRange::new(80, 84),
        selection_range: TextRange::new(80, 84),
        visibility: None,
        container: None,
        signature: Some(Signature {
            text: "Application".to_string(),
        }),
    };
    let dependencies = index
        .dependencies(DependencyQuery::in_file(file_id))
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let definition =
        receiver_method_definition_at(&mut index, file_id, TextOffset(95), &[app], &dependencies)
            .expect("handleRequest definition");

    let DefinitionLocation::Symbol(location) = definition else {
        panic!("expected symbol location");
    };
    let file = index.file(location.file_id).expect("application file");
    assert_eq!(file.path.as_deref(), Some(application.as_path()));
}

#[test]
fn resolves_file_path_reference_to_target_file() {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/php/112_laravel_bootstrap")
        .canonicalize()
        .expect("fixture root");
    let autoload = fixture_root.join("vendor/autoload.php");
    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    index.insert_file(IndexedFile {
        file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        file_id,
        IndexFacts {
            file_id,
            mode: EchoFileMode::PhpCompat,
            declarations: Vec::new(),
            dependencies: Vec::new(),
            references: vec![ReferenceFact {
                kind: ReferenceKind::FilePath,
                name: autoload.to_string_lossy().to_string(),
                qualifier: None,
                range: TextRange::new(10, 45),
            }],
        },
    );

    let location = reference_target_location_at(&mut index, file_id, TextOffset(20))
        .expect("autoload path location");

    assert_eq!(location.uri, Uri::from_file_path(&autoload).unwrap());
}

#[test]
fn file_path_reference_link_uses_full_origin_expression() {
    let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/php/112_laravel_bootstrap")
        .canonicalize()
        .expect("fixture root");
    let autoload = fixture_root.join("vendor/autoload.php");
    let source = "<?php\nrequire __DIR__.'/../vendor/autoload.php';\n";
    let expr = "__DIR__.'/../vendor/autoload.php'";
    let start = source.find(expr).expect("expr");
    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    index.insert_file(IndexedFile {
        file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        file_id,
        IndexFacts {
            file_id,
            mode: EchoFileMode::PhpCompat,
            declarations: Vec::new(),
            dependencies: Vec::new(),
            references: vec![ReferenceFact {
                kind: ReferenceKind::FilePath,
                name: autoload.to_string_lossy().to_string(),
                qualifier: None,
                range: TextRange::new(start as u32, (start + expr.len()) as u32),
            }],
        },
    );

    let link = reference_target_link_at(
        &mut index,
        &Rope::from_str(source),
        file_id,
        TextOffset((start + expr.find("vendor").expect("vendor")) as u32),
    )
    .expect("autoload link");

    assert_eq!(link.target_uri, Uri::from_file_path(&autoload).unwrap());
    assert_eq!(
        link.origin_selection_range,
        Some(range_to_lsp_range(
            &Rope::from_str(source),
            TextRange::new(start as u32, (start + expr.len()) as u32),
        ))
    );
}

#[test]
fn file_path_reference_link_skips_missing_dir_target() {
    let target = "/project/public/../storage/framework/maintenance.php";
    let source = "<?php\nfile_exists(__DIR__.'/../storage/framework/maintenance.php');\n";
    let expr = "__DIR__.'/../storage/framework/maintenance.php'";
    let start = source.find(expr).expect("expr");
    let mut index = EchoIndex::new();
    let file_id = FileId(1);
    index.insert_file(IndexedFile {
        file_id,
        uri: "file:///project/public/index.php".to_string(),
        path: None,
        version: None,
        mode: EchoFileMode::PhpCompat,
        content_hash: None,
    });
    index.update_file(
        file_id,
        IndexFacts {
            file_id,
            mode: EchoFileMode::PhpCompat,
            declarations: Vec::new(),
            dependencies: Vec::new(),
            references: vec![ReferenceFact {
                kind: ReferenceKind::FilePath,
                name: target.to_string(),
                qualifier: None,
                range: TextRange::new(start as u32, (start + expr.len()) as u32),
            }],
        },
    );

    let link = reference_target_link_at(
        &mut index,
        &Rope::from_str(source),
        file_id,
        TextOffset((start + expr.find("storage").expect("storage")) as u32),
    );

    assert!(link.is_none());
}
