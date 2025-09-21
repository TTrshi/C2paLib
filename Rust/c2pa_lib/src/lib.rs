use jni::{
    objects::{GlobalRef, JClass, JObject, JString},
    sys::jstring,
    JNIEnv,
};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use android_logger::Config;
use log::{info, warn, error, LevelFilter};

use std::sync::OnceLock;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("Tokio runtime failed"))
}


/// アプリ全体で共有したい状態
struct AppState {
    counter: i32,
}

/// Rust 側のグローバル状態
static GLOBAL_STATE: Lazy<Mutex<AppState>> =
    Lazy::new(|| Mutex::new(AppState { counter: 0 }));

/// Kotlin のコールバックを保持
static KOTLIN_CALLBACK: Lazy<Mutex<Option<GlobalRef>>> =
    Lazy::new(|| Mutex::new(None));

/// Kotlin から Rust のグローバル状態を使う関数
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_c2pajni_NativeBridge_callRustFunction(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {

    // 一度だけ初期化
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Debug) // 出力したいレベル
            .with_tag("MyRustLib"),             // logcat でのタグ
    );

    let text: String = env.get_string(&input).unwrap().into();
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.counter += 1;

    // Runtime を安全に取得
    let rt = get_runtime();

    // 非同期タスクを起動
    let handle = rt.spawn(async move {
        sleep(Duration::from_secs(10)).await;
        info!("Rust spawn!");

        format!("Hello from async, got: {}", "async")
    });

    // 結果をブロックして取得
    //let result = rt.block_on(handle).unwrap();

    let reply = format!("Rust got '{}', counter = {}", text, state.counter);
    env.new_string(reply).unwrap().into_raw()
}

/// Kotlin のコールバックを登録
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_c2pajni_NativeBridge_registerCallback(
    env: JNIEnv,
    _class: JClass,
    callback_obj: JObject,
) {
    let global_ref = env.new_global_ref(callback_obj).unwrap();
    *KOTLIN_CALLBACK.lock().unwrap() = Some(global_ref);
}

/// Rust から Kotlin コールバックを呼び出す
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_c2pajni_NativeBridge_triggerKotlinFromRust(
    mut env: JNIEnv,
    _class: JClass,
) {
    if let Some(cb) = KOTLIN_CALLBACK.lock().unwrap().as_ref() {
        let arg = env.new_string("Hello from Rust!").unwrap();
        let _ = env.call_method(cb.as_obj(), "onResult", "(Ljava/lang/String;)V", &[(&arg).into()]);
    }
}
