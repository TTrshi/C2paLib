use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use tokio::{sync::Notify, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::runtime::*;
use crate::signing_task::*;
use crate::task_handler::*;

/// TaskManager 本体
pub struct SingleTaskManager<H: TaskHandler> {
    // タスク格納用のキュー
    task_queue: Mutex<VecDeque<SigningTask>>,

    // キュー到着通知
    notify: Notify,

    // キュー中 + 実行中の合計数
    pending_counter: AtomicUsize,

    // 実行中タスクをキャンセルするためのトークン（None or Some(token)）
    running_cancel: Mutex<Option<CancellationToken>>,

    // ワーカー終了フラグ
    shutdown: AtomicBool,

    // ワーカースレッド（tokio::JoinHandle）を格納しておく（start/stop 用）
    worker_handle: Mutex<Option<JoinHandle<()>>>,

    // ハンドラー
    pub(crate) handler: Mutex<H>,
}

impl<H: TaskHandler> SingleTaskManager<H> {
    pub fn new(handler: H) -> Arc<Self> {
        Arc::new(Self {
            task_queue: Mutex::new(VecDeque::new()),
            notify: Notify::new(),
            pending_counter: AtomicUsize::new(0),
            running_cancel: Mutex::new(None),
            shutdown: AtomicBool::new(false),
            worker_handle: Mutex::new(None),
            handler: Mutex::new(handler),
        })
    }

    /// ワーカースレッドを開始
    pub fn start(self: &Arc<Self>) {
        let rt = GLOBAL_RUNTIME.get().expect("Runtime not initialized");

        // スレッドの開始
        let manager = Arc::clone(self);
        let handle = rt.spawn(async move { manager.worker_loop().await });
        
        // ハンドルの保持
        let mut handle_guard = self.worker_handle.lock().unwrap();
        *handle_guard = Some(handle);
    }

    /// ワーカースレッドを停止
    pub fn stop(&self) {

        // worker_loop をbreak
        self.shutdown.store(true, Ordering::SeqCst);

        // 止まっている worker_loop を起こす
        self.notify.notify_one();

        // ハンドルの削除
        let mut handle_guard = self.worker_handle.lock().unwrap();
        if let Some(handle) = handle_guard.take() {
            let rt = GLOBAL_RUNTIME.get().expect("Runtime not initialized");
            //let _ = handle.await;
            let _ = rt.block_on(handle);
        }
    }

    /// タスクの発行
    pub fn enqueue_task(&self, data: EnvData) {
        // タスクをキューに入れる
        let mut task_queue = self.task_queue.lock().unwrap();
        let task = SigningTask::new(data.clone());
        task_queue.push_back(task);

        // 保留数を増やす
        self.pending_counter.fetch_add(1, Ordering::SeqCst);

        // 止まっているworker_loopを起こす
        self.notify.notify_one();
    }

    /// 保留中のタスク数をを取得
    pub fn pending_counter(&self) -> usize {
        self.pending_counter.load(Ordering::SeqCst)
    }

    /// 保留中のタスクをすべてキャンセル
    pub fn cancel_all(&self) {

        // 削除したタスク数を取得
        let clear_len = {
            let mut task_queue = self.task_queue.lock().unwrap();
            let len = task_queue.len();
            task_queue.clear();
            len
        };

        // 保留数を減らす
        self.pending_counter.fetch_sub(clear_len, Ordering::SeqCst);

        // 実行中のタスクをキャンセル
        if let Some(token) = self.running_cancel.lock().unwrap().take() {
            token.cancel();
        }
    }

    // ワーカーループ
    async fn worker_loop(self: Arc<Self>) {
        loop {
            // 積まれたタスクを取得
            let task = {
                let mut task_queue = self.task_queue.lock().unwrap();
                task_queue.pop_front()
            };

            match task {
                Some(mut signing_task) => {
                    let cancel = CancellationToken::new();
                    *self.running_cancel.lock().unwrap() = Some(cancel.clone());

                    //
                    //let mut signing = SigningTask::new(data.clone());


                    let result = signing_task.run(cancel.clone()).await;

                    // 保留中のタスク数を減らす
                    self.pending_counter.fetch_sub(1, Ordering::SeqCst);

                    // コールバック呼び出し
                    let mut h = self.handler.lock().unwrap();
                    h.on_task_finished(result);
                },
                None => {
                    // loopを抜ける
                    if self.shutdown.load(Ordering::SeqCst) {
                        break;
                    }

                    // タスクが無い場合はここで待機する
                    self.notify.notified().await;
                },
            }

            // if let Some(signing_task_) = signing_task {
            //     let cancel = CancellationToken::new();
            //     *self.running_cancel.lock().unwrap() = Some(cancel.clone());

            //     //
            //     //let mut signing = SigningTask::new(data.clone());


            //     let result = signing_task.run(cancel.clone()).await;

            //     // 保留中のタスク数を減らす
            //     self.pending_counter.fetch_sub(1, Ordering::SeqCst);

            //     // コールバック呼び出し
            //     let mut h = self.handler.lock().unwrap();
            //     h.on_task_complete(&signing_task.data, result);
            // } else {

            //     // loopを抜ける
            //     if self.shutdown.load(Ordering::SeqCst) {
            //         break;
            //     }

            //     // タスクが無い場合はここで待機する
            //     self.notify.notified().await;
            // }
        }
    }
}
