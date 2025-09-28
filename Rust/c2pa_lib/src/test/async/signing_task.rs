use std::time::Duration;
use tokio::time;
use tokio_util::sync::CancellationToken;

use crate::task_result::*;


/// タスクデータ
#[derive(Clone, Debug)]
pub struct EnvData {
    pub id: u64,
    pub payload: Vec<u8>,
}

/// 署名タスク構造体
pub struct SigningTask {
    pub data: EnvData,
}

impl SigningTask {
    pub fn new(data: EnvData) -> Self { Self { data } }

    pub async fn run(&mut self, cancel: CancellationToken) -> TaskResult {
        let steps = 10;
        for _i in 0..steps {
            tokio::select! {
                _ = cancel.cancelled() => {
                    return TaskResult::Cancelled(
                        ResultCancelData{
                            id: 42, 
                            payload: vec![]
                        }
                    )
                }
                _ = time::sleep(Duration::from_millis(100)) => {
                    // 擬似処理ステップ
                }
            }
        }
        
        TaskResult::Ok(
            ResultSuccessData{
                id: 42, 
                payload: vec![],
                is_retry: true
            }
        )
    }
}