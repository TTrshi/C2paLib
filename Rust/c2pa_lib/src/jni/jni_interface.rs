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

/*
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
*/

use jni::objects::{GlobalRef, JObject, JString};
use jni::sys::jlong;
use jni::{JNIEnv, JavaVM};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u64)]
enum CallbackKind {
    OnSuccess = 1,
    OnError = 2,
    OnProgress = 3,
}

// let mut map: HashMap<CallbackKind, String> = HashMap::new();
// map.insert(CallbackKind::OnSuccess, "Success Callback".to_string());

// let value = map.get(&CallbackKind::OnSuccess).unwrap();
// println!("{}", value); // => Success Callback

/// アプリ全体の状態
#[derive(Debug)]
pub struct AppState {
    pub counter: u64,
}

/// 1つのコールバック情報
#[derive(Debug, Clone)]
pub struct Callback {
    pub obj: Arc<GlobalRef>,
    pub kind: String,
}

/// JNI 側の全体コンテキスト
pub struct CallbackManager {
    pub vm: JavaVM,
    pub state: RwLock<AppState>,
    pub callbacks: Mutex<HashMap<u64, Callback>>,
    pub next_id: Mutex<u64>,
}

/// シングルトンとして保持
static MANAGER: OnceCell<Arc<CallbackManager>> = OnceCell::new();

impl CallbackManager {
    /// Rust API からも呼べる register
    pub fn register_callback(&self, env: &JNIEnv, callback: JObject, kind: String) -> u64 {
        let global = env.new_global_ref(callback).unwrap();
        let mut id_guard = self.next_id.lock().unwrap();
        let id = *id_guard;
        *id_guard += 1;

        self.callbacks.lock().unwrap().insert(
            id,
            Callback {
                obj: Arc::new(global),
                kind,
            },
        );

        id
    }

    /// Rust API からも呼べる call
    pub fn call_callback(&self, id: u64, message: &str) {
        let env = self.vm.attach_current_thread().unwrap();

        if let Some(cb) = self.callbacks.lock().unwrap().get(&id).cloned() {
            let jmsg = env.new_string(message).unwrap();
            let _ = env.call_method(cb.obj.as_obj(), "onResult", "(Ljava/lang/String;)V", &[jmsg.into()]);
        } else {
            eprintln!("Callback with id {} not found", id);
        }
    }

    /// Rust API からも呼べる unregister
    pub fn unregister_callback(&self, id: u64) {
        self.callbacks.lock().unwrap().remove(&id);
    }

    /// シングルトンを取得
    pub fn instance() -> Arc<Self> {
        MANAGER.get().expect("CallbackManager not initialized").clone()
    }

    /// 初期化
    pub fn init(env: &JNIEnv) -> Arc<Self> {
        let vm = env.get_java_vm().unwrap();
        let manager = Arc::new(CallbackManager {
            vm,
            state: RwLock::new(AppState { counter: 0 }),
            callbacks: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        });
        MANAGER.set(manager.clone()).ok();
        manager
    }
}

/// --- JNI 関数 ---

#[no_mangle]
pub extern "system" fn Java_com_example_c2pajni_NativeBridge_initManager(
    env: JNIEnv,
    _this: JObject,
) {
    CallbackManager::init(&env);
}

#[no_mangle]
pub extern "system" fn Java_com_example_c2pajni_NativeBridge_registerCallback(
    env: JNIEnv,
    _this: JObject,
    callback: JObject,
    kind: JString,
) -> jlong {
    let kind: String = env.get_string(&kind).unwrap().into();
    let id = CallbackManager::instance().register_callback(&env, callback, kind);
    id as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_example_c2pajni_NativeBridge_callCallback(
    _env: JNIEnv,
    _this: JObject,
    id: jlong,
    message: JString,
) {
    let text = unsafe { _env.get_string(&message).unwrap().into() };
    CallbackManager::instance().call_callback(id as u64, &text);
}

#[no_mangle]
pub extern "system" fn Java_com_example_c2pajni_NativeBridge_unregisterCallback(
    _env: JNIEnv,
    _this: JObject,
    id: jlong,
) {
    CallbackManager::instance().unregister_callback(id as u64);
}


//■使用方法
// fn rust_api_example(env: &JNIEnv, callback: JObject) {
//     // シングルトン取得
//     let manager = CallbackManager::instance();

//     // コールバック登録
//     let id = manager.register_callback(env, callback, "onEvent".to_string());

//     // 呼び出し
//     manager.call_callback(id, "Hello from Rust API");

//     // 削除
//     manager.unregister_callback(id);
// }