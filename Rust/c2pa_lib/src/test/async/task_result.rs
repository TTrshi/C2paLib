
/// タスク実行結果
#[derive(Debug)]
pub enum TaskResult {
    Ok(ResultSuccessData),
    Cancelled(ResultCancelData),
    Err(ResultErrorData),
}

/// 
#[derive(Clone, Debug)]
pub struct ResultSuccessData {
    pub id: u64,
    pub payload: Vec<u8>,
    pub is_retry: bool,
}

/// 
#[derive(Clone, Debug)]
pub struct ResultCancelData {
    pub id: u64,
    pub payload: Vec<u8>,
}

/// 
#[derive(Clone, Debug)]
pub struct ResultErrorData {
    pub id: u64,
    pub payload: Vec<u8>,
}
