use std::ops::Deref;
use std::path::Path;
use std::string::String;
use std::time::{Duration, SystemTime};

use axum::http;

use crate::core::aes_utils::{aes_decrypt, generate_aes_iv};
use crate::core::file_utils::{read_data_from_file, read_file_line_by_line, write_data_to_file};
use crate::core::system_utils::execute_shell_command;
use crate::flags::FILE_PATH;
use crate::{PhoneVar, UseFilePath, FIRST_FLAG, FLAGS, KEY_BYTES, PLOG, WAIT_TIME};

pub fn fake_id(f_path: &UseFilePath, v_local: &PhoneVar) -> bool {
    let bid = read_file_line_by_line(&f_path.board_id)
        .ok()
        .and_then(|lines| lines.get(0).cloned())
        .unwrap_or_default();
    let r = execute_shell_command("mount | grep -i serial_number").unwrap_or_default();
    PLOG.write().unwrap().info(format!("挂载表检测:{}", r));
    PLOG.write()
        .unwrap()
        .info(format!("主板ID新旧对比:{} {}", bid, v_local.board_id));
    r.is_empty() && bid == v_local.board_id
}

pub fn reboot_shell(reason: &str) {
    PLOG.write()
        .unwrap()
        .error(format!("Reboot reason:{}", reason));
    if cfg!(not(debug_assertions)) {
        PLOG.write().unwrap().error(format!("Reboot"));
        exec_cmd("reboot");
        exec_cmd("setprop sys.powerctl reboot");
    }
}

pub async fn wait_check() {
    let mut install_time = FLAGS.read().unwrap().install_time;
    let mut now_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    {
        if install_time > now_time {
            FLAGS.write().unwrap().install_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            install_time = FLAGS.read().unwrap().install_time;
            now_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            write_data_to_file(
                FLAGS.read().unwrap().deref(),
                Some(&KEY_BYTES.to_vec()),
                Some(&generate_aes_iv().unwrap()),
                &FILE_PATH.data_binary,
            )
            .unwrap();
        }
    }
    let pass_time = now_time - install_time;
    let sleep_time = match WAIT_TIME > pass_time {
        true => WAIT_TIME - pass_time,
        false => 0,
    };
    PLOG.write()
        .unwrap()
        .info(format!("等待时间:{}", sleep_time));
    PLOG.write().unwrap().info(format!(
        "第一次启动:{}",
        FIRST_FLAG.read().unwrap().to_owned()
    ));
    PLOG.write().unwrap().info(format!(
        "当前时间:{} 安装时间:{} 经过时间:{} 休眠时间:{}",
        now_time, install_time, pass_time, sleep_time
    ));
    match FIRST_FLAG.read().unwrap().to_owned() {
        true => {
            std::thread::sleep(Duration::from_secs(3600));
        }
        false => {
            if sleep_time > 0 {
                std::thread::sleep(Duration::from_secs(sleep_time));
            }
        }
    }
    if FLAGS.read().unwrap().to_owned().verify_status {
        PLOG.write().unwrap().info("验证通过!".to_string());
        return;
    }
    reboot_shell("验证超时");
}

pub async fn online_check() -> Result<bool, Box<dyn std::error::Error>> {
    let unlock_v_local = FLAGS.read().expect("Failed to get lock").to_owned();
    let full_url = format!("{}/{}-{}.vf", "", unlock_v_local.phone_model, "rs");
    let online = reqwest::get(full_url).await?;
    if online.status() != http::StatusCode::OK {
        return Ok(false);
    }
    let online_data_byte = online.bytes();
    let vec = online_data_byte.await?.to_vec();
    let decrypt = aes_decrypt(&KEY_BYTES.to_vec(), &vec).unwrap();
    let online_id: Vec<String> = bincode::deserialize(&decrypt).unwrap();
    match online_id.iter().any(|x| x == &unlock_v_local.board_id) {
        true => Ok(true),
        false => Ok(false),
    }
}

pub fn create_and_read_local_file() -> PhoneVar {
    let file_val: PhoneVar;
    let key = KEY_BYTES.to_vec();
    // 判断本地数据文件
    let db_status = Path::new(FILE_PATH.data_binary).exists();
    if db_status {
        PLOG.write().unwrap().info("Found File".to_string());
        // 读取文件并且反序列化
        file_val = read_data_from_file(Some(&key), &FILE_PATH.data_binary).unwrap();
        FLAGS.write().expect("获取锁失败").install_time = file_val.install_time.clone();
        FLAGS.write().expect("获取锁失败").check_time = file_val.check_time.clone();
        FLAGS.write().expect("获取锁失败").verify_status = file_val.verify_status.clone();
    } else {
        PLOG.write().unwrap().warn("Not Found".to_string());
        // 将local_val序列化写入到文件中
        let iv = generate_aes_iv().unwrap();
        let un_lock_now = FLAGS.read().expect("获取锁失败").to_owned();
        let _ = write_data_to_file(&un_lock_now, Some(&key), Some(&iv), &FILE_PATH.data_binary);
        file_val = un_lock_now.clone();
        *FIRST_FLAG.write().unwrap() = true;
    }
    file_val
}

pub fn exec_cmd(cmd: &str) {
    match execute_shell_command(cmd) {
        Ok(result) => {
            PLOG.write().unwrap().info(result.clone());
        }
        Err(err) => {
            PLOG.write().unwrap().error(err.to_string());
        }
    }
}
