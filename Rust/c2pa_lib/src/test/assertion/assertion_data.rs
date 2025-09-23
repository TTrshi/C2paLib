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