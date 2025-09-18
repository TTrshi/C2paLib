/*
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
*/

use crate::client::{assertion_builder::AssertionBuilder, c2pa_signer::C2paSigner};

use anyhow::Result;
use c2pa::{Manifest, Signer};
use futures::future::{AbortHandle, Abortable};
use std::time::Instant;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::sleep,
};
use tokio::{fs, sync::mpsc, task};

pub type SignResult = Result<PathBuf>;
pub type SignCallback = Arc<dyn Fn(SignResult) + Send + Sync>;

const MPSC_BUFFER_SIZE: usize = 100;

struct SignTask {
    input: PathBuf,
    output: PathBuf,
    assertion_builder: AssertionBuilder,
}

/// 非同期署名キュー
pub struct SignerExecuter {
    sender: Option<mpsc::Sender<SignTask>>,
    cancel_flag: Arc<AtomicBool>,
    current_abort_handle: Arc<tokio::sync::Mutex<Option<AbortHandle>>>,
    callback: SignCallback,
}

impl SignerExecuter {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<SignTask>(MPSC_BUFFER_SIZE);
        let arc_cancel_flag = Arc::new(AtomicBool::new(false));
        let arc_current_abort_handle = Arc::new(tokio::sync::Mutex::new(None));

        // ★ 関数をコールバックに設定
        let arc_callback: SignCallback = Arc::new(Self::signed_callback);

        let callback_spawn = arc_callback.clone();
        let cancel_flag_spawn = arc_cancel_flag.clone();
        let current_abort_handle_spawn = arc_current_abort_handle.clone();

        tokio::spawn(async move {
            while let Some(request) = rx.recv().await {
                if cancel_flag_spawn.load(Ordering::Relaxed) {
                    (callback_spawn)(Err(anyhow::anyhow!("キャンセル済み")));
                    continue;
                }

                // 処理を行うデータ
                let SignTask {
                    input,
                    output,
                    assertion_builder,
                } = request;

                let future = Self::process_sign(input.clone(), output.clone(), assertion_builder);

                let (handle, abort_registration) = AbortHandle::new_pair();
                let abortable = Abortable::new(future, abort_registration);

                // AbortHandle を current_abort に登録してロックを解放
                let mut guard = current_abort_handle_spawn.lock().await;
                *guard = Some(handle.clone());
                drop(guard); // ロックを明示的に解放

                let result = match task::spawn(abortable).await {
                    Ok(Ok(path)) => Ok(path),
                    Ok(Err(_)) => Err(anyhow::anyhow!("タスク中断")),
                    Err(e) => Err(anyhow::anyhow!("join エラー: {e}")),
                };

                // コールバックで通知
                (callback_spawn)(result.flatten());

                // AbortHandleのロックを開放
                let mut guard = current_abort_handle_spawn.lock().await;
                *guard = None;
            }
        });

        Self {
            sender: Some(tx),
            cancel_flag: arc_cancel_flag,
            current_abort_handle: arc_current_abort_handle,
            callback: arc_callback,
        }
    }

    /// コールバックを差し替える
    pub fn set_callback<F>(&mut self, f: F)
    where
        F: Fn(SignResult) + Send + Sync + 'static,
    {
        self.callback = Arc::new(f);
    }

    /// 署名ジョブを追加
    pub async fn enqueue(&self, input: PathBuf, output: PathBuf) {
        if let Some(tx) = &self.sender {
            let assertion_builder = AssertionBuilder::default();
            let _ = tx
                .send(SignTask {
                    input,
                    output,
                    assertion_builder,
                })
                .await;
        }
    }

    /// 実行中タスクを含めて全削除
    pub async fn shutdown_now(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        self.sender = None;

        if let Some(handle) = self.current_abort_handle.lock().await.take() {
            handle.abort();
        }
    }

    /// --- ★ 非同期で実行したい処理を関数化 ---
    ///
    /// 入力ファイルに署名し、署名済みファイルのパスを返す
    pub async fn process_sign(
        input: PathBuf,
        output: PathBuf,
        assertion_builder: AssertionBuilder,
    ) -> Result<PathBuf> {
        // 計測開始
        let start = Instant::now();

        assertion_builder.write_json();
        assertion_builder.read_json();

        println!("start sign_content.");
        let input_file_jpeg_string = "src/fixtures/earth_apollo17.jpg";
        let output_file_jpeg_string = "target/output.jpg";
        let signer = C2paSigner::default();
        signer.sign_media_file(input_file_jpeg_string, output_file_jpeg_string);

        //sleep(std::time::Duration::from_secs(1));

        assertion_builder.write_json();
        assertion_builder.read_json();

        println!("start sign_content.");
        let input_file_jpeg_string = "src/fixtures/earth_apollo17.jpg";
        let output_file_jpeg_string = "target/output.jpg";
        let signer = C2paSigner::default();
        signer.sign_media_file(input_file_jpeg_string, output_file_jpeg_string);

        // 経過時間を取得
        let duration = start.elapsed();
        println!("[JPEG] 署名 処理時間: {:?}\n", duration);

        Ok(output)
    }

    /// ===== コールバックを関数として定義 =====
    pub fn signed_callback(res: SignResult) {
        match res {
            Ok(p) => println!("[default] 署名成功: {:?}", p),
            Err(e) => eprintln!("[default] 署名失敗: {e}"),
        }
    }
}
