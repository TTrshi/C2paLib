use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Serialize;

use c2pa::{Builder, Error, SigningAlg, create_signer};

use anyhow::Result;
use std::io::{Cursor, Seek, Write};


use c2pa::{
    settings::Settings, validation_results::ValidationState,
    CallbackSigner, Reader,
};
use serde_json::json;

// （オプション）Exif 読み取り用
//use exif::{Reader as ExifReader, Tag, In};

// カスタムアサーション構造体
#[derive(Serialize)]
struct MyCustomAssertion {
    project: String,
    author: String,
}

fn clean_tmp_file(tmp_file_path: &str) {
    let file_path = Path::new(tmp_file_path);

    // ファイルが存在するか確認
    if file_path.exists() {
        // ファイルを削除
        match fs::remove_file(&file_path) {
            Ok(_) => println!("ファイル '{}' を削除しました。", file_path.display()),
            Err(e) => eprintln!("ファイルの削除中にエラーが発生しました: {}", e),
        }
    }
}

const TEST_IMAGE: &[u8] = include_bytes!("../fixtures/earth_apollo17.jpg");
const CERTS: &[u8] = include_bytes!("../fixtures/certs/ed25519.pub");
const PRIVATE_KEY: &[u8] = include_bytes!("../fixtures/certs/ed25519.pem");

fn manifest_def(title: &str, format: &str) -> String {
    json!({
        "title": title,
        "format": format,
        "claim_generator_info": [
            {
                "name": "c2pa test",
                "version": env!("CARGO_PKG_VERSION")
            }
        ],
        "thumbnail": {
            "format": format,
            "identifier": "manifest_thumbnail.jpg"
        },
        "ingredients": [
            {
                "title": "Test",
                "format": "image/jpeg",
                "instance_id": "12345",
                "relationship": "inputTo"
            }
        ],
        "assertions": [
            {
                "label": "c2pa.actions",
                "data": {
                    "actions": [
                        {
                            "action": "c2pa.edited",
                            "digitalSourceType": "http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia",
                            "softwareAgent": {
                                "name": "My AI Tool",
                                "version": "0.1.0"
                            }
                        }
                    ]
                }
            }
        ]
    }).to_string()
}

pub fn main() {
    println!("start sign_content.");

    // // 入力／出力ファイル
    // let input_file_string = "src/fixtures/earth_apollo17.jpg";
    // let output_file_string = "target/output.jpg";
    // let input_path = Path::new(input_file_string);
    // let output_path = Path::new(output_file_string);

    // clean_tmp_file(output_file_string);

    // // 証明書と秘密鍵
    // let cert_path = "src/fixtures/certs/es256.pub";
    // let key_path = "src/fixtures/certs/es256.pem";

    let title = "v2_edited.jpg";
    let format = "image/jpeg";
    let parent_name = "CA.jpg";
    let mut source = Cursor::new(TEST_IMAGE);

    let modified_core = toml::toml! {
        [core]
        debug = true
        hash_alg = "sha512"
        max_memory_usage = 123456
    }
    .to_string();

    Settings::from_toml(&modified_core).unwrap();

    let json = manifest_def(title, format);

    let mut builder = Builder::from_json(&json).unwrap();
    builder.add_ingredient_from_stream(
        json!({
            "title": parent_name,
            "relationship": "parentOf"
        })
        .to_string(),
        format,
        &mut source,
    ).unwrap();

    let thumb_uri = builder
        .definition
        .thumbnail
        .as_ref()
        .map(|t| t.identifier.clone());

    // add a manifest thumbnail ( just reuse the image for now )
    if let Some(uri) = thumb_uri {
        if !uri.starts_with("self#jumbf") {
            source.rewind().unwrap();
            builder.add_resource(&uri, &mut source).unwrap();
        }
    }

    // write the manifest builder to a zipped stream
    let mut zipped = Cursor::new(Vec::new());
    builder.to_archive(&mut zipped).unwrap();

    // write the zipped stream to a file for debugging
    let debug_path = format!("{}/target/test.zip", env!("CARGO_MANIFEST_DIR"));
    std::fs::write(debug_path, zipped.get_ref()).unwrap();

    // unzip the manifest builder from the zipped stream
    zipped.rewind().unwrap();

    let ed_signer =
        |_context: *const (), data: &[u8]| CallbackSigner::ed25519_sign(data, PRIVATE_KEY);
    let signer = CallbackSigner::new(ed_signer, SigningAlg::Ed25519, CERTS);

    let mut builder = Builder::from_archive(&mut zipped).unwrap();
    builder.definition.claim_version = Some(2);
    // sign the ManifestStoreBuilder and write it to the output stream
    let mut dest = Cursor::new(Vec::new());
    builder.sign(&signer, format, &mut source, &mut dest).unwrap();

    // read and validate the signed manifest store
    dest.rewind().unwrap();

    let reader = Reader::from_stream(format, &mut dest).unwrap();

    // extract a thumbnail image from the ManifestStore
    let mut thumbnail = Cursor::new(Vec::new());
    if let Some(manifest) = reader.active_manifest() {
        if let Some(thumbnail_ref) = manifest.thumbnail_ref() {
            reader.resource_to_stream(&thumbnail_ref.identifier, &mut thumbnail).unwrap();
            println!(
                "wrote thumbnail {} of size {}",
                thumbnail_ref.format,
                thumbnail.get_ref().len()
            );
        }
    }

    println!("{}", reader.json());
    assert_ne!(reader.validation_state(), ValidationState::Invalid);
    assert_eq!(reader.active_manifest().unwrap().title().unwrap(), title);

    // Cursorから内部のVec<u8>を取得
    let data = dest.into_inner();

    // ファイルに書き込む
    fs::write("target/output.jpg", data).expect("ファイルへの書き込みに失敗しました");

    //println!("Signed output written to {:?}", output_path);
}
