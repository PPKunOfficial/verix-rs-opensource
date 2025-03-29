use std::time::SystemTime;

use axum::{Form, Router};
use axum::extract::Query;
use axum::Json;
use axum::response::Html;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{FLAGS, OPEN_IP, PANEL_PORT, PLOG};
use crate::check::{online_check, reboot_shell};
use crate::core::aes_utils::generate_aes_iv;
use crate::core::file_utils::write_data_to_file;
use crate::core::system_utils::execute_shell_command;
use crate::flags::{FILE_PATH, PhoneVar};
use crate::KEY_BYTES;

#[derive(Deserialize)]
struct QueryParams {
    pwd: String,
}

#[derive(Deserialize)]
struct ShellBody {
    // 在这里定义与你的JSON主体匹配的结构体
    // 例如，如果JSON主体包含一个名为 "name" 的字段，可以像下面这样定义：
    cmd: String,
}

#[derive(Serialize)]
struct Result {
    id: String,
    verify: bool,
}
const NO_VERIFY:&[u8]=include_bytes!("public/index.impl");
const VERIFY:&[u8]=include_bytes!("public/already_login.impl");
const DEV_PASSWORD:&str="114514";

struct MyRouter;
impl MyRouter{
    fn dev() -> Router {
        Router::new()
            .route("/dev/reboot",get(DevApi::dev_reboot))
            .route("/dev/now",get(DevApi::dev_value))
            .route("/dev/log",get(DevApi::dev_log))
            .route("/dev/cmd", post(DevApi::dev_shell))
    }
    fn api() -> Router {
        Router::new()
            .route("/api/open/now",get(UserApi::pv_json))
            .route("/api/verify",post(UserApi::verify_api))
    }
    fn all_router()->Router{
        Router::new()
            .route("/",get(index))
            .merge(MyRouter::api())
            .merge(MyRouter::dev())
    }
}

async fn index() -> Html<String> {
    let unlock=FLAGS.read().expect("获取锁失败").to_owned();
    if unlock.verify_status{
        let v=VERIFY.to_vec();
        Html(String::from_utf8(v).unwrap_or_default())
    }else {
        let v=NO_VERIFY.to_vec();
        Html(String::from_utf8(v).unwrap_or_default())
    }
}
struct UserApi;
impl UserApi {
    async fn verify_api() -> Json<Value> {
        let res = online_check().await.unwrap_or(false);
        let unlock = FLAGS.read().expect("获取锁失败").to_owned();
        let res_json = Result {
            id: unlock.board_id,
            verify: res,
        };
        if res {
            FLAGS.write().expect("获取锁失败").verify_status = true;
            FLAGS.write().expect("获取锁失败").check_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let r = FLAGS.read().expect("获取锁失败").to_owned();
            let iv = generate_aes_iv().unwrap();
            let _ = write_data_to_file(&r, Some(&KEY_BYTES.to_vec()), Some(&iv), &FILE_PATH.data_binary);
            PLOG.write().unwrap().info(format!("Verify Success: {:?}",FLAGS.read().expect("获取锁失败").to_owned()));
        }
        Json(serde_json::to_value(res_json).unwrap_or(Value::from(false)))
    }

    async fn pv_json() -> Json<PhoneVar> {
        let state = FLAGS.read().expect("获取锁失败").to_owned();
        Json(state.clone())
    }
}
struct DevApi;
impl DevApi {
    async fn dev_value(Query(params): Query<QueryParams>)-> Json<Value> {
        match params.pwd{
            res=> {
                if res==DEV_PASSWORD{
                    Json(serde_json::to_value(FLAGS.read().expect("获取锁失败").to_owned()).unwrap_or_default())
                }else {
                    Json(serde_json::to_value("Error").unwrap_or_default())
                }
            }
        }
    }
    async fn dev_reboot(Query(params): Query<QueryParams>) -> Html<String> {
        //返回OK
        //执行重启操作
        PLOG.write().unwrap().info("dev reboot".to_string());
        match params.pwd{
            res=> {
                if res==DEV_PASSWORD{
                    reboot_shell("Test");
                    Html(format!("<h1>Reboot</h1>"))
                }else {
                    Html(format!("<h1>Error</h1>"))
                }
            }
        }
    }
    async fn dev_log(Query(params): Query<QueryParams>) -> Html<String> {
        match params.pwd{
            res=> {
                if res==DEV_PASSWORD{
                    Html(PLOG.write().expect("获取锁失败").all_log())
                }else {
                    Html(format!("<h1>Error</h1>"))
                }
            }
        }
    }
    async fn dev_shell(request_body: Form<ShellBody>) -> String {
        match execute_shell_command(&request_body.cmd){
            Ok(res) => {res}
            Err(err) => {err.to_string()}
        }
    }
}

pub async fn start_panel_service(){
    axum::Server::bind(&format!("{}:{}",OPEN_IP,PANEL_PORT).parse().unwrap())
        .serve(MyRouter::all_router().into_make_service())
        .await
        .unwrap();
}