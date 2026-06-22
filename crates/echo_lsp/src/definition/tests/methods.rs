use super::*;

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
