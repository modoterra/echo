use super::*;

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
