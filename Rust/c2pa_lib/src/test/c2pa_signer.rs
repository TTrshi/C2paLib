use std::fs::{self, File};
use std::io::{BufReader, Error, ErrorKind, Read, Write, Result, Seek, SeekFrom};
use std::path::Path;

use c2pa::assertions::Exif;
use chrono::{DateTime, Utc};
use serde::Serialize;

use c2pa::{Builder, Reader, SigningAlg, ValidationState, create_signer};

use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

// カスタムアサーション構造体
#[derive(Serialize)]
struct MyCustomAssertion2 {
    project: String,
    author: String,
    list_tmp: Vec<String>,
}

#[derive(Serialize)]
struct MyCustomAssertion {
    project: String,
    author: String,
    list_tmp: Vec<String>,
    my_custom_assertion2: MyCustomAssertion2,
}

#[derive(Debug, Default)]
pub struct C2paSigner {
    input_file_path: String,
    output_file_path: String,
}

impl C2paSigner {
    pub fn new(cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self {
            input_file_path: cert_path.into(),
            output_file_path: key_path.into(),
        }
    }

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
            // "thumbnail": {
            //     "format": format,
            //     "identifier": "manifest_thumbnail.jpg"
            // },
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

    pub fn sign_media_file(&self, input_file_path: &str, output_file_path: &str) {
        let source = PathBuf::from(input_file_path);
        let dest = PathBuf::from(output_file_path);
        if dest.exists() {
            std::fs::remove_file(&dest).unwrap();
        }

        // let schemas_file_string = "schemas/test.json";
        // let bytes = std::fs::read(&schemas_file_string).unwrap();
        // let schemas_json = String::from_utf8(bytes).unwrap();
        let title = "v2_edited.jpg";
        let format = "image/jpeg";
        let json = C2paSigner::manifest_def(title, format);
        let mut builder = match Builder::from_json(&json) {
            Ok(builder) => builder,
            Err(e) => {
                println!("Error.");
                return;
            }
        };

        // 証明書と秘密鍵
        let cert_path = "src/fixtures/certs/es256.pub";
        let key_path = "src/fixtures/certs/es256.pem";

        // 署名に使う signer を準備
        let signer =
            create_signer::from_files(cert_path, key_path, SigningAlg::Es256, None).unwrap();

        // 作成日時
        let now: DateTime<Utc> = Utc::now();
        builder
            .add_assertion("c2pa.created_time", &now.to_rfc3339())
            .unwrap();

        let exif = Exif::from_json_str(
            r#"{
                "@context" : {
                "exif": "http://ns.adobe.com/exif/1.0/"
                },
                "exif:GPSVersionID": "2.2.0.0",
                "exif:GPSLatitude": "39,21.102N",
                "exif:GPSLongitude": "74,26.5737W",
                "exif:GPSAltitudeRef": 0,
                "exif:GPSAltitude": "100963/29890",
                "exif:GPSTimeStamp": "2019-09-22T18:22:57Z"
            }"#,
        )
        .unwrap();
        builder.add_assertion(Exif::LABEL, &exif).unwrap();

        // カスタムアサーションを追加
        let custom = MyCustomAssertion {
            project: "MyProjectName".to_string(),
            author: "Alice".to_string(),
            list_tmp: vec![
                String::from("item_a"),
                String::from("item_b"),
                String::from("item_c"),
            ],
            my_custom_assertion2: MyCustomAssertion2 {
                project: "MyProjectName".to_string(),
                author: "Alice".to_string(),
                list_tmp: vec![
                    String::from("item_0"),
                    String::from("item_1"),
                    String::from("item_2"),
                ],
            },
        };
        builder
            .add_assertion("com.example.custom_info", &custom)
            .unwrap();

        // 最後に署名してファイルに manifest を埋め込む
        builder.sign_file(&*signer, &source, &dest).unwrap();

        let reader = Reader::from_file(&dest).unwrap();
        // let unsigned_like = "target/unsigned_like.jpg"; // output with embedded manifest
        // Self::extract_jpeg_without_c2pa(output_file_path, unsigned_like).unwrap();

        // // ハッシュを計算
        // let h1 = Self::sha256_all_bytes(output_file_path).unwrap();
        // let h2 = Self::sha256_all_bytes(unsigned_like).unwrap();
        // println!("original: {:x?}", h1);
        // println!("from signed: {:x?}", h2);
        
        // example of how to print out the whole manifest as json
        // println!("{reader}\n");
        // assert_ne!(reader.validation_state(), ValidationState::Invalid);
    }

    pub fn comp_hash_jpeg(&self, signed_file_path: &str, unsigned_like: &str) {
        Self::extract_jpeg_without_c2pa(signed_file_path, unsigned_like).unwrap();

        // ハッシュを計算
        let h1 = Self::sha256_all_bytes(signed_file_path).unwrap();
        let h2 = Self::sha256_all_bytes(unsigned_like).unwrap();
        println!("original: {:x?}", hex::encode(h1));
        println!("from signed: {:x?}", hex::encode(h2));
    }

    pub fn comp_hash_mp4(&self, signed_file_path: &str, origin_file_path: &str) {
        let signed_hash = Self::hash_mp4_boxes(signed_file_path).unwrap();
        let origin_hash = Self::hash_mp4_boxes(origin_file_path).unwrap();

        for (key, value) in origin_hash.iter() {
            println!("original   : {:x?}", hex::encode(value));
        }
        // println!("");
        for (key, value) in signed_hash.iter() {
            println!("from signed: {:x?}", hex::encode(value));
        }
    }

    fn sha256_all_bytes<P: AsRef<Path>>(path: P) -> std::io::Result<[u8; 32]> {
        let data = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let out = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&out);
        Ok(arr)
    }

    /// UUID box が C2PA 署名かどうかを判定
    fn is_c2pa_uuid(uuid: [u8; 16]) -> bool {
        // 署名の box type は 'uuid' で、値は "C2PA" の GUID になる
        // （実際の UUID は C2PA 仕様に記載。ここでは先頭 4byte だけ比較）
        uuid[0..4] == [0x63, 0x32, 0x70, 0x61] // "c2pa"
    }

    /// MP4 をパースして、各 box のハッシュを計算
    fn hash_mp4_boxes(path: &str) -> anyhow::Result<Vec<(String, [u8; 32])>> {
        let mut f = File::open(path)?;
        let mut result = Vec::new();

        loop {
            // --- box header 読み込み ---
            let mut header = [0u8; 8];
            if f.read(&mut header)? != 8 {
                break; // EOF
            }

            let size = u32::from_be_bytes(header[0..4].try_into().unwrap()) as u64;
            let typ = [header[4], header[5], header[6], header[7]];

            let mut largesize = 0;
            let mut box_size = size;
            if size == 1 {
                // largesize
                let mut ls = [0u8; 8];
                f.read_exact(&mut ls)?;
                largesize = u64::from_be_bytes(ls);
                box_size = largesize;
            } else if size == 0 {
                // ファイル終端まで
                let pos = f.stream_position()?;
                let len = f.metadata()?.len();
                box_size = len - pos + 8;
            }

            let payload_size = box_size.saturating_sub(8 + if size == 1 { 8 } else { 0 });

            // --- C2PA uuid box か判定 ---
            if &typ == b"uuid" {
                let mut uuid = [0u8; 16];
                f.read_exact(&mut uuid)?;
                if Self::is_c2pa_uuid(uuid) {
                    // スキップ
                    f.seek(SeekFrom::Current(payload_size as i64 - 16))?;
                    continue;
                } else {
                    // uuid だが C2PA 以外 → ハッシュ
                    let mut hasher = Sha256::new();
                    hasher.update(&header);
                    if size == 1 {
                        hasher.update(&largesize.to_be_bytes());
                    }
                    hasher.update(&uuid);
                    let mut buf = vec![0u8; payload_size as usize - 16];
                    f.read_exact(&mut buf)?;
                    hasher.update(&buf);
                    result.push((
                        String::from_utf8_lossy(&typ).to_string(),
                        hasher.finalize().into(),
                    ));
                    continue;
                }
            }

            // --- 通常 box のハッシュ ---
            let mut hasher = Sha256::new();
            hasher.update(&header);
            if size == 1 {
                hasher.update(&largesize.to_be_bytes());
            }
            let mut buf = vec![0u8; payload_size as usize];
            f.read_exact(&mut buf)?;
            hasher.update(&buf);

            result.push((
                String::from_utf8_lossy(&typ).to_string(),
                hasher.finalize().into(),
            ));
        }

        Ok(result)
    }

    pub fn extract_jpeg_without_c2pa<P: AsRef<Path>>(signed: P, out: P) -> std::io::Result<()> {
        let mut buf = Vec::new();
        File::open(&signed)?.read_to_end(&mut buf)?;

        let mut cursor = 0;
        let mut result = Vec::with_capacity(buf.len());

        // SOI (0xFFD8)
        if buf.len() < 2 || buf[0] != 0xFF || buf[1] != 0xD8 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "not a JPEG",
            ));
        }
        result.extend_from_slice(&buf[..2]);
        cursor = 2;

        // 以降セグメントを順に読む
        while cursor + 4 <= buf.len() {
            if buf[cursor] != 0xFF {
                // スキャンデータに入った
                result.extend_from_slice(&buf[cursor..]);
                break;
            }
            let marker = buf[cursor + 1];
            cursor += 2;

            // スタンドアロンマーカー (RSTn, EOI 等)
            if marker == 0xD9 || (marker >= 0xD0 && marker <= 0xD7) {
                result.extend_from_slice(&[0xFF, marker]);
                continue;
            }

            if cursor + 2 > buf.len() {
                break;
            }
            let len = u16::from_be_bytes([buf[cursor], buf[cursor + 1]]) as usize;
            if cursor + len > buf.len() {
                break;
            }

            // APP11 (0xFFEB) のうち「C2PA」で始まるものをスキップ
            if marker == 0xEB && len > 6 && &buf[cursor + 2..cursor + 6] == b"C2PA" {
                // skip this segment (marker+length+payload)
                cursor += len;
                continue;
            }

            // それ以外のセグメントはそのままコピー
            result.extend_from_slice(&[0xFF, marker]);
            result.extend_from_slice(&buf[cursor..cursor + len]);
            cursor += len;
        }

        File::create(out)?.write_all(&result)?;
        Ok(())
    }
}
