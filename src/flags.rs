use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use crate::core::file_utils::{parse_build_prop, read_file_line_by_line};

#[derive(Debug)]
pub struct UseFilePath {
    pub(crate) build_prop: &'static str,
    pub(crate) board_id: &'static str,
    pub(crate) chip_name: &'static str,
    pub(crate) data_binary: &'static str,
    pub(crate) log_file: &'static str,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PhoneVar {
    pub(crate) board_id: String,
    pub(crate) phone_model: String,
    pub(crate) chip_name: String,
    pub(crate) finger: String,
    pub(crate) verify_status: bool,
    pub(crate) check_time: u64,
    pub(crate) install_time: u64,
}

impl PhoneVar {
    // 构造函数，用于创建新的 PhoneVar 实例
    pub fn new(
        board_id: String,
        phone_model: String,
        chip_name: String,
        finger: String,
        verify_status: bool,
        check_time: u64,
        install_time: u64,
    ) -> Self {
        Self {
            board_id,
            phone_model,
            chip_name,
            finger,
            verify_status,
            check_time,
            install_time,
        }
    }
    pub fn get_first_line(filename:&str) -> String {
        read_file_line_by_line(&filename)
            .ok()
            .and_then(|lines| lines.get(0).cloned())
            .unwrap_or_default()
    }
    pub fn get_prop_value(prop_map:&HashMap<String,String>,prop_name:&str) -> String {
        prop_map
            .get(prop_name)
            .cloned()
            .unwrap_or_default()
    }
    pub fn prop_to_map(prop_str:&str) -> HashMap<String,String> {
        match parse_build_prop(prop_str) {
            Ok(content) => content,
            Err(err) => {
                error!("Error: {:?}", err);
                HashMap::new()
            }
        }
    }
     pub fn info_all(&self){
        info!("{:?}", self);
    }
}


// 根据目标系统判断运行所需文件的路径
pub const FILE_PATH: UseFilePath = if cfg!(target_os="macos") {
    UseFilePath {
        build_prop: "/Users/pp/CLionProjects/verix-rs/test/build.prop",
        board_id: "/Users/pp/CLionProjects/verix-rs/test/serial_number",
        chip_name: "/Users/pp/CLionProjects/verix-rs/test/chip_name",
        data_binary: "/Users/pp/CLionProjects/verix-rs/test/test.bin",
        log_file: "/Users/pp/CLionProjects/verix-rs/test/log.txt",
    }
} else if cfg!(target_os="android") {
    UseFilePath {
        build_prop: "/odm/etc/build.prop",
        board_id: "/sys/devices/soc0/serial_number",
        chip_name: "/sys/devices/soc0/chip_name",
        data_binary: "/data/system/battery",
        log_file: "/data/system/log.txt",
    }
} else {
    UseFilePath {
        build_prop: "test/build.prop",
        board_id: "test/serial_number",
        chip_name: "test/chip_name",
        data_binary: "test/test.bin",
        log_file: "test/log.txt",
    }
};