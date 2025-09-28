use crate::task_result::*;

/// タスク完了後のコールバックを定義するトレイト
pub trait TaskHandler: Send + Sync + 'static {
    fn on_task_finished(&mut self, result: TaskResult);
}