use std::time::{SystemTime};
use crate::flags::{FILE_PATH, PhoneVar};

pub fn init() -> PhoneVar {
    let prop_content = PhoneVar::prop_to_map(&FILE_PATH.build_prop);

    PhoneVar::new(
        PhoneVar::get_first_line(&FILE_PATH.board_id),
        PhoneVar::get_prop_value(&prop_content,"ro.vendor.xlp.rom.helper.device"),
        PhoneVar::get_first_line(&FILE_PATH.chip_name),
        PhoneVar::get_prop_value(&prop_content,"ro.system.build.fingerprint"),
        false,
        0,
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap().as_secs(),
    )
}