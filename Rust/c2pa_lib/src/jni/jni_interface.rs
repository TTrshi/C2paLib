/*
use jni::{
    objects::{GlobalRef, JClass, JObject, JString},
    sys::jstring,
    JNIEnv,
};
use once_cell::sync::Lazy;
use std::sync::Mutex;

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
    let text: String = env.get_string(&input).unwrap().into();
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.counter += 1;

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
*/

use jni::{
    objects::{GlobalRef, JClass, JObject, JString},
    JNIEnv, JavaVM,
};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex, RwLock};

/// アプリ全体の状態
#[derive(Debug)]
pub struct AppState {
    pub counter: u64,
}

/// Kotlin コールバックを保持するためのハンドル
#[derive(Clone)]
pub struct CallbackHandle {
    pub inner: Arc<GlobalRef>,
}

/// JNI 全体のコンテキスト
pub struct JniContext {
    pub state: RwLock<AppState>,
    pub callback: Mutex<Option<CallbackHandle>>,
    pub jvm: JavaVM,
}

/// グローバルなシングルトン
static JNI_CONTEXT: Lazy<Mutex<Option<Arc<JniContext>>>> = Lazy::new(|| Mutex::new(None));

/// 初期化
#[no_mangle]
pub extern "system" fn Java_com_example_c2pajni_NativeBridge_init(
    env: JNIEnv,
    _class: JClass,
) {
    let jvm = env.get_java_vm().unwrap();
    let ctx = Arc::new(JniContext {
        state: RwLock::new(AppState { counter: 0 }),
        callback: Mutex::new(None),
        jvm,
    });

    *JNI_CONTEXT.lock().unwrap() = Some(ctx);
}

/// コールバックを登録
#[no_mangle]
pub extern "system" fn Java_com_example_c2pajni_NativeBridge_registerCallback(
    env: JNIEnv,
    _class: JClass,
    callback_obj: JObject,
) {
    let global = env.new_global_ref(callback_obj).unwrap();
    let handle = CallbackHandle { inner: Arc::new(global) };
    if let Some(ctx) = JNI_CONTEXT.lock().unwrap().as_ref() {
        *ctx.callback.lock().unwrap() = Some(handle);
    }
}

/// カウンタを増やす
#[no_mangle]
pub extern "system" fn Java_com_example_c2pajni_NativeBridge_incrementCounter(
    env: JNIEnv,
    _class: JClass,
    label: JString,
) {
    let msg: String = env.get_string(&label).unwrap().into();
    if let Some(ctx) = JNI_CONTEXT.lock().unwrap().as_ref() {
        {
            let mut state = ctx.state.write().unwrap();
            state.counter += 1;
            log::info!("{} -> counter = {}", msg, state.counter);
        }

        // コールバック呼び出し
        if let Some(cb) = ctx.callback.lock().unwrap().clone() {
            let env_attached = ctx.jvm.attach_current_thread().unwrap();
            let jmsg = env_attached.new_string(format!("{}:{}", msg, "done")).unwrap();
            let _ = env_attached.call_method(
                cb.inner.as_obj(),
                "onResult",
                "(Ljava/lang/String;)V",
                &[jmsg.into()],
            );
        }
    }
}