use once_cell::sync::OnceCell;
use tokio::{
    runtime::Runtime,
};
use anyhow::Result;


/// グローバル runtime（ライブラリ全体で共有）
pub static GLOBAL_RUNTIME: OnceCell<Runtime> = OnceCell::new();

/// 初期化（呼び出しは一度でよい、Androidならライブラリロード時に呼ぶ）
pub fn init_runtime() -> Result<()> {
    GLOBAL_RUNTIME.get_or_try_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
    })?;
    Ok(())
}

/// シャットダウン（必要なら呼ぶ）
pub fn shutdown_runtime() {
    // OnceCell::take を使って Runtime を取り出し、shutdown を呼ぶ
    // if let Some(rt) = GLOBAL_RUNTIME.take() {
    //     // 優雅に終了させる
    //     rt.shutdown_timeout(Duration::from_secs(5));
    // }
}

