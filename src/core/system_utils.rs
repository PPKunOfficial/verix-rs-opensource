use std::process::Command;

// execute_shell_command 执行Shell命令并返回输出结果
#[allow(dead_code)]
pub fn execute_shell_command(command: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new(command).output()?;

    if !output.status.success() {
        let error_message = std::str::from_utf8(&output.stderr)?.trim();
        return Err(format!("执行命令出错：{}", error_message).into());
    }

    let output_str = std::str::from_utf8(&output.stdout)?.to_string();
    Ok(output_str)
}

// get_non_system_app_list 获取非系统应用列表
#[allow(dead_code)]
pub fn get_non_system_app_list() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let command = "su -c pm list packages -3 | sed -e 's/^package://'";
    let output = execute_shell_command(command)?;

    // 根据回车切割字符串
    let non_system_apps: Vec<String> = output.trim().split('\n').map(String::from).collect();

    Ok(non_system_apps)
}
