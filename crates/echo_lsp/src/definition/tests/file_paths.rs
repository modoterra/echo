use super::*;

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
