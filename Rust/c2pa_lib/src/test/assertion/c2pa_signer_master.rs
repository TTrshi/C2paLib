use std::path::Path;
use std::fs;
use c2pa::{Reader, Builder, create_signer, SigningAlg};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use tokio::fs::read;
use std::path::PathBuf;
use serde_json::json;


/// 撮影時のカメラパラメータ
#[derive(Debug, Serialize, Deserialize)]
pub struct CameraParams {
    pub make: String,          // メーカー名 (例: "Canon")
    pub model: String,         // モデル (例: "EOS R5")
    pub lens: Option<String>,  // レンズ情報
    pub iso: Option<u32>,
    pub exposure_time: Option<f32>, // 秒
    pub aperture: Option<f32>,      // f値
    pub focal_length: Option<f32>,  // mm
}

/// Exif 情報の一部
#[derive(Debug, Serialize, Deserialize)]
pub struct ExifInfo {
    pub datetime_original: Option<DateTime<Utc>>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<u8>,
}

/// カスタムメタデータ
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomInfo {
    pub author: String,
    pub location: Option<String>,
    pub comment: Option<String>,
}

/// 署名に含める総合的なアサーション構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct CaptureAssertion {
    pub exif: ExifInfo,
    pub camera: CameraParams,
    pub custom: CustomInfo,
}

#[derive(Debug, Default)]
pub struct C2paSignerMaster {
    input_file_path: String,
    output_file_path: String,
}

impl C2paSignerMaster {

    pub fn sign_media_file(&self, input_file_path: &str, output_file_path: &str) {
        let source = PathBuf::from(input_file_path);
        let dest = PathBuf::from(output_file_path);
        if dest.exists() {
            std::fs::remove_file(&dest).unwrap();
        }

        // アサーションデータを構築
        let assertion = CaptureAssertion {
            exif: ExifInfo {
                datetime_original: Some(Utc::now()),
                width: Some(4000),
                height: Some(3000),
                orientation: Some(1),
            },
            camera: CameraParams {
                make: "Canon".into(),
                model: "EOS R5".into(),
                lens: Some("RF24-70mm F2.8".into()),
                iso: Some(100),
                exposure_time: Some(0.005),
                aperture: Some(2.8),
                focal_length: Some(35.0),
            },
            custom: CustomInfo {
                author: "T. Takahashi".into(),
                location: Some("Tokyo, Japan".into()),
                comment: Some("テスト撮影".into()),
            },
        };

        // マニフェスト生成
        let manifest_def = serde_json::json!({
            "claim_generator": "rust-c2pa/0.1",
            "title": "Photo with EXIF + Camera Params",
            "assertions": []
        });

        let mut builder = Builder::from_json(&manifest_def.to_string()).unwrap();
        //let mut builder = Builder::edit();

        // 構造体を JSON として埋め込む
        builder.add_assertion("org.example.capture", &assertion).unwrap();

        // 署名用の signer を準備
        let signer = create_signer::from_files(
            "src/fixtures/certs/es256.pub",
            "src/fixtures/certs/es256.pem",
            SigningAlg::Es256,
            None,
        ).unwrap();

        // JPEG や MP4 でも同様に可能
        builder.sign_file(&*signer, input_file_path, output_file_path).unwrap();
    }

    

    pub fn read_media_file(&self, input_file_path: &str) {
        let reader = Reader::from_file(input_file_path).unwrap();
        println!("manifest store json:\n{}", reader.json());
        let manifest = reader.active_manifest().unwrap();

        // アサーションを検索（org.example.capture）
        //if let Some(assertion) = manifest.find_assertion("org.example.capture") {
        let assertion: Result<CaptureAssertion, c2pa::Error> = manifest.find_assertion("org.example.capture");
        let capture_assertion = match assertion {
            Ok(capture_assertion) => capture_assertion,
            Err(error) => {
                // ファイルを開く際に問題がありました
                panic!("There was a problem opening the file: {:?}", error)
            },
        };
        // let f = f.unwrap_or_else(|error| {
        //     panic!("There was a problem opening the file: {:?}", error);
        // });
        // if let assertion: Action = manifest.find_assertion("org.example.capture") {
        //     // JSON を構造体に変換
        //     let value: Value = serde_json::from_slice(assertion.data()).unwrap();
        //     let parsed: CaptureAssertion = serde_json::from_value(value).unwrap();

        //     println!("復元したアサーション: {:#?}", parsed);
        // } else {
        //     println!("アサーションが見つかりませんでした");
        // }
        println!("{:#?}", capture_assertion);
    }

}
