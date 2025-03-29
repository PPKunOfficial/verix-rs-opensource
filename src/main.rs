use std::ops::Deref;
use std::panic;
use std::sync::{Arc, RwLock};
use lazy_static::lazy_static;

use crate::check::{wait_check, fake_id, create_and_read_local_file, reboot_shell};
use crate::flags::*;
use crate::init::init;
use crate::log::Plog;
use crate::panel::start_panel_service;

mod check;
mod init;
mod flags;
mod panel;
mod log;

mod core {
    pub mod file_utils;
    pub mod aes_utils;
    pub mod system_utils;
}
fn custom_panic_hook(info: &panic::PanicInfo) {
    // 在 panic 发生前运行自定义的代码
    PLOG.write().unwrap().info(format!("Panic occurred: {:?}", info));
    reboot_shell("Panic!")
}
lazy_static! {
    static ref PLOG: Arc<RwLock<Plog>> = Arc::new(RwLock::new(Plog::new(&FILE_PATH.log_file)));
    static ref FLAGS: Arc<RwLock<PhoneVar>> = Arc::new(RwLock::new(init()));
    static ref FIRST_FLAG: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));
}
const WAIT_TIME: u64 = 3600;
const KEY_BYTES: &[u8] = include_bytes!("key");
const PANEL_PORT: &str = "8888";
const OPEN_IP: &str = "0.0.0.0";
#[tokio::main]
async fn main() {
    if cfg!(not(debug_assertions)){
        panic::set_hook(Box::new(custom_panic_hook));
    }
    tracing_subscriber::fmt::init();
    PLOG.write().unwrap().info("Verix Run!".to_string());
    let file_val = create_and_read_local_file();
    FLAGS.read().unwrap().info_all();
    file_val.info_all();
    PLOG.write().unwrap().info(serde_json::to_string(FLAGS.read().unwrap().deref()).unwrap_or_default());
    PLOG.write().unwrap().info(serde_json::to_string(&file_val).unwrap_or_default());
    // 初步检查
    let is_safe = fake_id(&FILE_PATH, &file_val);
    PLOG.write().unwrap().info(format!("设备环境检查:{}",is_safe));
    match is_safe {
        true => {}
        false => { reboot_shell("Phone Environment Unsafe") }
    }
    // 启动检测线程
    let check_handle=tokio::spawn(wait_check());
    PLOG.write().unwrap().info(format!("Panel URL: {}:{}",
        match OPEN_IP{
            "0.0.0.0"=>"localhost",
            _=>OPEN_IP,
        }
        ,PANEL_PORT));
    start_panel_service().await;
    check_handle.await.unwrap();
}