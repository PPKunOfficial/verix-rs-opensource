use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tracing::{error, info, warn};

pub struct Plog{
    log_handle:File,
    log_context:String,
}
impl Plog{
    pub fn new(lf: &str) ->Self{
        Self{
            log_handle:match Path::new(lf).exists(){
                true=>{
                    if fs::metadata(lf).unwrap().len() > 1 * 1024 * 1024 {
                        // 删除文件
                        fs::remove_file(lf).unwrap();
                        File::create(lf).unwrap()
                    }else{
                        fs::OpenOptions::new()
                            .append(true)
                            .write(true)
                            .read(true)
                            .create(true)
                            .open(lf)
                            .unwrap()
                    }
                },
                false=>File::create(lf).unwrap(),
            },
            log_context: "".to_string(),
        }
    }
    pub fn write(&mut self,msg:String)
    {
        let msg=msg+"\n";
        self.log_context=self.log_context.clone()+&msg;
        self.log_handle.write_all(msg.as_bytes()).unwrap();
    }
    pub fn _close(&mut self){
        self.log_handle.sync_all().unwrap();
        self.log_handle.flush().unwrap();
    }
    pub fn all_log(&mut self)->String{
        self.log_context.clone()
    }
    pub fn info(&mut self,msg: String){
        self.write(format!("[INFO]:{}",msg));
        info!("{}",msg);
    }
    pub fn warn(&mut self,msg:String){
        self.write(format!("[WARN]:{}",msg));
        warn!("{}",msg);
    }
    pub fn error(&mut self,msg:String){
        self.write(format!("[ERR]:{}",msg));
        error!("{}",msg);
    }
}