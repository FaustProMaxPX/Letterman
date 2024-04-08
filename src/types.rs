use serde::Serialize;

pub mod posts;

#[derive(Serialize)]
pub struct CommonResult<T> {
    pub msg: String,
    pub data: T,
}

impl<T> CommonResult<T> {
    pub fn new() -> CommonResult<()> {
        CommonResult {
            msg: "OK".to_string(),
            data: (),
        }  
    }
    pub fn with_msg(msg: &str) -> CommonResult<()> {
        CommonResult {
            msg: msg.to_string(),
            data: (),
        }
        
    }
    pub fn with_data(msg: String, data: T) -> CommonResult<T> {
        CommonResult { msg, data }
    }
}
