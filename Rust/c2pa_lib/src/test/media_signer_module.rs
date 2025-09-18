use crate::client::file_manager::FileManager;
use crate::client::signer_executer::SignerExecuter;
use anyhow::Result;
use std::path::PathBuf;
use tokio::{runtime::Runtime, time::sleep};

pub struct MediaManager {
    queue: SignerExecuter,
}

impl MediaManager {
    pub fn new() -> Self {
        let mut q = SignerExecuter::new();

        // // 任意のコールバックを設定
        // q.set_callback(|res| match res {
        //     Ok(p) => println!("[custom] 署名成功: {:?}", p),
        //     Err(e) => eprintln!("[custom] エラー: {e}"),
        // });

        {
            let dir = "src/fixtures"; // 調べたいディレクトリのパス
            let extension = "jpg"; // 取得したい拡張子（ドットなし）

            let file_manager = FileManager;
            let files = file_manager.list_files_with_extension(dir, extension).unwrap();

            println!("{} ファイル一覧:", extension);
            for file in files {
                println!("{}", file);
            }
        }

        Self { queue: q }
    }

    pub async fn enqueue_jobs(&self) {
        for i in 0..3 {
            let input = PathBuf::from(format!("sample{i}.jpg"));
            let output = PathBuf::from(format!("signed{i}.jpg"));
            self.queue.enqueue(input, output).await;
        }
    }

    pub async fn run(mut self) -> Result<()> {
        self.enqueue_jobs().await;
        sleep(std::time::Duration::from_secs(5)).await;
        self.queue.shutdown_now().await;
        println!("全ジョブを停止しました");
        Ok(())
    }
}
