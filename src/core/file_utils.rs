use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, Read, Write};
use crate::core::aes_utils::{aes_decrypt, aes_encrypt};

#[allow(dead_code)]
pub fn read_file_line_by_line(file_path: &str) ->io::Result<Vec<String>>{
    let file_point=File::open(file_path)?;
    let reader=io::BufReader::new(file_point);

    let mut lines=Vec::new();

    for lines_result in reader.lines(){
        let line=lines_result?;
        lines.push(line)
    }

    Ok(lines)
}

#[allow(dead_code)]
pub fn parse_build_prop(file_path: &str) -> io::Result<HashMap<String, String>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut properties = HashMap::new();

    for line_result in reader.lines() {
        let line = line_result?;
        let parts: Vec<&str> = line.trim().splitn(2, '=').collect();

        if parts.len() == 2 {
            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();
            properties.insert(key, value);
        }
    }

    Ok(properties)
}


pub fn write_data_to_file<T>(data: &T, key: Option<&Vec<u8>>, iv: Option<&Vec<u8>>, file_path: &str) -> io::Result<()>
    where
        T: serde::Serialize,
{
    // 将数据序列化
    let serialized_data = bincode::serialize(data).unwrap();

    // 如果提供了 IV，则使用 IV 对数据进行加密
    let encrypted_data = if let (Some(key_value), Some(iv_value)) = (key, iv) {
        aes_encrypt(key_value, iv_value, &serialized_data).unwrap()
    } else {
        serialized_data
    };

    // 写入文件
    let mut file = File::create(file_path)?;
    file.write_all(&encrypted_data)?;

    Ok(())
}

pub fn read_data_from_file<T>(key: Option<&Vec<u8>>,file_path: &str)->io::Result<T>
    where
        T: serde::de::DeserializeOwned,
{
    let mut new_db=File::open(&file_path).expect("Failed to open file");
    let mut content=Vec::new();
    new_db.read_to_end(&mut content).unwrap_or_default();
    // 解密
    let oc=if let Some(key_value)=key{
        aes_decrypt(key_value,&content).unwrap()
    }else {
        content
    };

    bincode::deserialize(&oc).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}