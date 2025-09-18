use std::fs::{self, File};

//use crate::client::c2pa_signer::C2paSigner;
use anyhow::Result;
use futures::future::{AbortHandle, Abortable};
use std::time::Instant;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{fs as other_fs, sync::mpsc};

type SignResult = Result<PathBuf>;
type SignCallback = Box<dyn FnOnce(SignResult) + Send>;

use crate::client::assertion_builder::{self, AssertionBuilder};

//use crate::client::c2pa_signer::SignerExecuter;
use crate::client::media_signer_module::MediaManager;

// use signer_queue::SignerQueue;
use tokio::{runtime::Runtime, time::sleep};

// struct SignRequest {
//     input: PathBuf,
//     output: PathBuf,
//     callback: SignCallback,
// }

// struct SignerQueue {
//     tx: mpsc::Sender<SignRequest>,
// }

// impl SignerQueue {
//     fn new() -> Self {
//         let (tx, mut rx) = mpsc::channel::<SignRequest>(100);

//         // ワーカータスク
//         task::spawn(async move {
//             while let Some(req) = rx.recv().await {
//                 let SignRequest {
//                     input,
//                     output,
//                     callback,
//                 } = req;

//                 // 署名処理はブロッキングなので専用スレッドで実行
//                 let result: SignResult = task::spawn_blocking(move || {
//                     // --- C2PA 署名の例 ---
//                     {
//                         println!("start sign_content.");
//                         let input_file_jpeg_string = "src/fixtures/earth_apollo17.jpg";
//                         let output_file_jpeg_string = "target/output.jpg";
//                         let input_file_mp4_string = "src/fixtures/video1_no_manifest.mp4";
//                         let output_file_mp4_string = "target/signed_video.mp4";
//                         let signer = C2paSigner::default();

//                         // 計測開始
//                         let start = Instant::now();
//                         signer.sign_media_file(input_file_jpeg_string, output_file_jpeg_string);
//                         // 経過時間を取得
//                         let duration = start.elapsed();
//                         println!("[JPEG] 署名 処理時間: {:?}\n", duration);

//                         let start = Instant::now();
//                         let unsigned_like_file_jpeg_string = "target/unsigned_like.jpg";
//                         signer.comp_hash_jpeg(
//                             output_file_jpeg_string,
//                             unsigned_like_file_jpeg_string,
//                         );
//                         // 経過時間を取得
//                         let duration = start.elapsed();
//                         println!("[JPEG] 比較 処理時間: {:?}\n", duration);

//                         let start = Instant::now();
//                         signer.sign_media_file(input_file_jpeg_string, output_file_jpeg_string);
//                         let duration = start.elapsed();
//                         println!("[MP4] 署名 処理時間: {:?}\n", duration);

//                         let start = Instant::now();
//                         signer.comp_hash_mp4(input_file_mp4_string, output_file_mp4_string);

//                         let duration = start.elapsed();
//                         println!("[MP4] 比較 処理時間: {:?}\n", duration);
//                     }
//                     Ok(output)
//                 })
//                 .await
//                 .unwrap_or_else(|e| Err(anyhow::anyhow!("join error: {e}")));

//                 // 結果をコールバック
//                 (callback)(result);
//             }
//         });

//         Self { tx }
//     }

//     async fn enqueue<F>(&self, input: PathBuf, output: PathBuf, callback: F)
//     where
//         F: FnOnce(SignResult) + Send + 'static,
//     {
//         let _ = self
//             .tx
//             .send(SignRequest {
//                 input,
//                 output,
//                 callback: Box::new(callback),
//             })
//             .await;
//     }
// }

/*
struct SignRequest {
    input: PathBuf,
    output: PathBuf,
    callback: SignCallback,
}

struct SignerQueue {
    tx: Option<mpsc::Sender<SignRequest>>,
    cancel_flag: Arc<AtomicBool>,
    current_task: Arc<tokio::sync::Mutex<Option<AbortHandle>>>,
}

impl SignerQueue {
    fn new() -> Self {
        let (tx, mut rx) = mpsc::channel::<SignRequest>(100);
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let current_task = Arc::new(tokio::sync::Mutex::new(None));

        let cf = cancel_flag.clone();
        let ct = current_task.clone();

        tokio::spawn(async move {
            while let Some(req) = rx.recv().await {
                if cf.load(Ordering::Relaxed) {
                    (req.callback)(Err(anyhow::anyhow!("canceled")));
                    continue;
                }

                let SignRequest {
                    input,
                    output,
                    callback,
                } = req;
                let cf2 = cf.clone();

                // --- 非同期署名処理を定義 ---
                let fut = async move {
                    // 証明書を非同期で読み込む
                    let key = tokio::fs::read("src/fixtures/certs/es256.pem").await?;
                    let certs = tokio::fs::read("src/fixtures/certs/es256.pub").await?;

                    if cf2.load(Ordering::Relaxed) {
                        anyhow::bail!("canceled before signing");
                    }

                    {
                        println!("start sign_content.");
                        let input_file_jpeg_string = "src/fixtures/earth_apollo17.jpg";
                        let output_file_jpeg_string = "target/output.jpg";
                        let input_file_mp4_string = "src/fixtures/video1_no_manifest.mp4";
                        let output_file_mp4_string = "target/signed_video.mp4";
                        let signer = C2paSigner::default();

                        // 計測開始
                        let start = Instant::now();
                        signer.sign_media_file(input_file_jpeg_string, output_file_jpeg_string);
                        // 経過時間を取得
                        let duration = start.elapsed();
                        println!("[JPEG] 署名 処理時間: {:?}\n", duration);

                        let start = Instant::now();
                        let unsigned_like_file_jpeg_string = "target/unsigned_like.jpg";
                        signer.comp_hash_jpeg(
                            output_file_jpeg_string,
                            unsigned_like_file_jpeg_string,
                        );
                        // 経過時間を取得
                        let duration = start.elapsed();
                        println!("[JPEG] 比較 処理時間: {:?}\n", duration);

                        let start = Instant::now();
                        signer.sign_media_file(input_file_jpeg_string, output_file_jpeg_string);
                        let duration = start.elapsed();
                        println!("[MP4] 署名 処理時間: {:?}\n", duration);

                        let start = Instant::now();
                        signer.comp_hash_mp4(input_file_mp4_string, output_file_mp4_string);

                        let duration = start.elapsed();
                        println!("[MP4] 比較 処理時間: {:?}\n", duration);
                    }
                    Ok::<_, anyhow::Error>(output)
                };

                // Abortable でラップ
                let (abort_handle, reg) = AbortHandle::new_pair();
                *ct.lock().await = Some(abort_handle.clone());

                let abortable = Abortable::new(fut, reg);
                let result = abortable.await;

                let res = match result {
                    Ok(Ok(path)) => Ok(path),
                    Ok(Err(e)) => Err(e),
                    Err(_) => Err(anyhow::anyhow!("task aborted")),
                };

                callback(res);
                *ct.lock().await = None;
            }
        });

        Self {
            tx: Some(tx),
            cancel_flag,
            current_task,
        }
    }

    async fn enqueue<F>(&self, input: PathBuf, output: PathBuf, callback: F)
    where
        F: FnOnce(SignResult) + Send + 'static,
    {
        if let Some(tx) = &self.tx {
            let _ = tx
                .send(SignRequest {
                    input,
                    output,
                    callback: Box::new(callback),
                })
                .await;
        }
    }

    /// キューの受付・実行中タスクをすべて中断
    async fn shutdown_now(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        self.tx = None;
        if let Some(h) = self.current_task.lock().await.take() {
            h.abort();
        }
    }
}
*/

#[tokio::main]
pub async fn main() -> Result<()> {
    // println!("start sign_content.");
    // let input_file_jpeg_string = "src/fixtures/earth_apollo17.jpg";
    // let output_file_jpeg_string = "target/output.jpg";
    // let input_file_mp4_string = "src/fixtures/video1_no_manifest.mp4";
    // let output_file_mp4_string = "target/signed_video.mp4";
    // let signer = C2paSigner::default();

    // // 計測開始
    // let start = Instant::now();
    // signer.sign_media_file(input_file_jpeg_string, output_file_jpeg_string);
    // // 経過時間を取得
    // let duration = start.elapsed();
    // println!("[JPEG] 署名 処理時間: {:?}\n", duration);

    // let start = Instant::now();
    // let unsigned_like_file_jpeg_string = "target/unsigned_like.jpg";
    // signer.comp_hash_jpeg(output_file_jpeg_string, unsigned_like_file_jpeg_string);
    // // 経過時間を取得
    // let duration = start.elapsed();
    // println!("[JPEG] 比較 処理時間: {:?}\n", duration);

    // let start = Instant::now();
    // signer.sign_media_file(input_file_jpeg_string, output_file_jpeg_string);
    // let duration = start.elapsed();
    // println!("[MP4] 署名 処理時間: {:?}\n", duration);

    // let start = Instant::now();
    // signer.comp_hash_mp4(input_file_mp4_string, output_file_mp4_string);

    // let duration = start.elapsed();
    // println!("[MP4] 比較 処理時間: {:?}\n", duration);

    // {
    //     let queue = Arc::new(SignerQueue::new());

    //     for i in 0..3 {
    //         let q = queue.clone();
    //         let input = PathBuf::from(format!("sample{i}.jpg"));
    //         let output = PathBuf::from(format!("signed{i}.jpg"));
    //         q.enqueue(input, output, move |res| match res {
    //             Ok(path) => println!("署名成功: {:?}", path),
    //             Err(e) => eprintln!("署名エラー: {e}"),
    //         })
    //         .await;
    //     }

    //     // キューが終わるのを待つ例
    //     tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    // }
    {
        //let rt = Runtime::new()?;
        //rt.block_on(App::new().run());
        MediaManager::new().run().await;
    }
    {
        // let assertion_builder = AssertionBuilder::default();
        // assertion_builder.write_json();
        // assertion_builder.read_json();
        return Ok(())
    }

    // {
    //     let mut queue = SignerQueue::new();

    //     // 3件投入
    //     for i in 0..3 {
    //         let input = PathBuf::from(format!("sample{i}.jpg"));
    //         let output = PathBuf::from(format!("signed{i}.jpg"));
    //         queue
    //             .enqueue(input, output, move |res| match res {
    //                 Ok(p) => println!("署名成功: {:?}", p),
    //                 Err(e) => eprintln!("エラー: {e}"),
    //             })
    //             .await;
    //     }

    //     // 途中で強制停止
    //     tokio::time::sleep(std::time::Duration::from_secs(6)).await;
    //     queue.shutdown_now().await;
    //     println!("強制停止しました");
    // }

    Ok(())
}
