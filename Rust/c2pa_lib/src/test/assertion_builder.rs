use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use serde_json::{self, json, Value};

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    id: u32,
    title: String,
    done: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Project {
    pub name: String,
    // ← 基底型を汎用にしたい場合
    #[serde(rename = "com.assertion")]
    pub tasks: Vec<serde_json::Value>,

    #[serde(rename = "com.assertion.value")]
    pub json_value: serde_json::Value,
}

#[derive(Debug, Default)]
pub struct AssertionBuilder {
    input_file_path: String,
    output_file_path: String,
}

impl AssertionBuilder {
    pub fn new(cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self {
            input_file_path: cert_path.into(),
            output_file_path: key_path.into(),
        }
    }

    fn build_response() -> Value {
        json!({
            "status": "ok",
            "data": {
                "id": 42,
                "name": "example"
            }
        })
    }

    pub fn write_json(&self) {
        let project = Project {
            name: "Rust入門".to_string(),
            tasks: vec![
                json!({ "id": 1, "title": "cargo new", "done": true }),
                json!({ "id": 2, "title": "serde", "done": false }),
            ],
            json_value: json!({ "id": 3, 
            "title": "serde を学ぶ", 
            "done": { "id": 2, "title": "serde", "done": Self::build_response() } 
        })
        };

        // struct → JSON 文字列
        let json_str = serde_json::to_string_pretty(&project).unwrap();
        println!("struct を JSON に変換:\n{:#?}", json_str);

        // ファイルに書き込み
        let mut file = File::create("target/config.json").unwrap();
        file.write_all(json_str.as_bytes()).unwrap();

        println!("保存しました:\n {}", json_str);
    }

    pub fn read_json(&self) {
        let json_str = fs::read_to_string("target/config.json").unwrap();

        // JSON → struct に戻す
        let p2: Project = serde_json::from_str(&json_str).unwrap();
        println!("復元した struct:\n {}", p2.json_value);
    }
}
