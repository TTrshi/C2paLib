use std::collections::HashMap;
use std::io;
use std::{
    sync::Arc,
};

mod signing_task;
use signing_task::*;
mod task_handler;
mod task_manager;
use task_handler::*;
use task_manager::*;

mod runtime;
use runtime::*;

mod task_result;
use task_result::*;

// /// グローバル runtime（ライブラリ全体で共有）
// static GLOBAL_RUNTIME: OnceCell<Runtime> = OnceCell::new();

// /// 初期化（呼び出しは一度でよい、Androidならライブラリロード時に呼ぶ）
// pub fn init_runtime() -> Result<()> {
//     GLOBAL_RUNTIME.get_or_try_init(|| {
//         tokio::runtime::Builder::new_multi_thread()
//             .enable_all()
//             .build()
//     })?;
//     Ok(())
// }

// /// シャットダウン（必要なら呼ぶ）
// pub fn shutdown_runtime() {
//     // OnceCell::take を使って Runtime を取り出し、shutdown を呼ぶ
//     // if let Some(rt) = GLOBAL_RUNTIME.take() {
//     //     // 優雅に終了させる
//     //     rt.shutdown_timeout(Duration::from_secs(5));
//     // }
// }




#[derive(Clone, Debug)]
pub struct RetryData {
    pub id: u64,
    pub retry_str: String,
    pub payload: Vec<u8>,
}


struct MyHandler {
    retry_map: HashMap<String, RetryData>,

    retry_counter: usize,
    manager: Option<Arc<SingleTaskManager<MyHandler>>>,
}

// トレイトの実装側
impl TaskHandler for MyHandler {

    // コールバック定義
    fn on_task_finished(&mut self, result: TaskResult) {
        println!("Callback: result={:?}", result);

        match result {
            TaskResult::Ok(_data) => {
                self.retry_counter += 1;
            },
            TaskResult::Cancelled(_data) => {

            },
            TaskResult::Err(_data) => {

            },
        }

        // コールバック内から再度タスクを発行
        if let Some(manager) = &self.manager {
            if self.retry_counter < 3 {
                //mgr.enqueue_task(TaskData { id: data.id + 100, payload: vec![0xFF] });

                manager.enqueue_task(EnvData { id: 0, payload: vec![4,5,6] });
            }
        }
    }
}

impl MyHandler {
    fn clear_retry_counter(&mut self) {
        self.retry_counter = 0;
    }
}

fn wait_key_input() {
    println!("Enterキーを押してください...");
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).expect("行の読み込みに失敗しました");
}

fn main() {
        // runtime 初期化
        init_runtime().unwrap();

        // let manager = TaskManager::new();
        // manager.start();

        let handler = MyHandler { retry_map: HashMap::default(), retry_counter: 0, manager: None };
        let manager = SingleTaskManager::new(handler);

        // handler に manager をセット
        manager.handler.lock().unwrap().manager = Some(Arc::clone(&manager));

        manager.start();

        // manager.enqueue_task(TaskData { id: 1, payload: vec![1,2,3] });
        // manager.enqueue_task(TaskData { id: 2, payload: vec![4,5,6] });

        //std::thread::sleep(std::time::Duration::from_secs(2));
        for i in 0..10 {
            manager.enqueue_task(EnvData { id: i, payload: vec![4,5,6] });
        }
        

        wait_key_input();

        manager.cancel_all();
        
        manager.enqueue_task(EnvData { id: 1, payload: vec![1,2,3] });
        manager.enqueue_task(EnvData { id: 2, payload: vec![4,5,6] });


        // 少し待つ
        //thread::sleep(Duration::from_secs(2));

        wait_key_input();

        // 停止して runtime をシャットダウン
        manager.stop();
        shutdown_runtime();
}

