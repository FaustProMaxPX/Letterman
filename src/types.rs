use serde::{Deserialize, Serialize};

use crate::traits::Validate;

pub mod posts;

#[derive(Serialize)]
pub struct CommonResult<T> {
    pub code: i32,
    pub msg: String,
    pub data: T,
}

impl<T> CommonResult<T> {
    pub fn success() -> CommonResult<()> {
        CommonResult {
            code: 0,
            msg: "OK".to_string(),
            data: (),
        }
    }
    pub fn with_msg(code: i32, msg: &str) -> CommonResult<()> {
        CommonResult {
            msg: msg.to_string(),
            data: (),
            code,
        }
    }
    pub fn with_data(code: i32, msg: &str, data: T) -> CommonResult<T> {
        CommonResult {
            msg: msg.to_string(),
            data,
            code,
        }
    }
    pub fn success_with_data(data: T) -> CommonResult<T> {
        CommonResult::with_data(0, "OK", data)
    }
    pub fn fail() -> CommonResult<()> {
        CommonResult::<()>::with_msg(1, "fail")
    }
    pub fn fail_with_msg(msg: &str) -> CommonResult<()> {
        CommonResult::<()>::with_msg(1, msg)
    }
}

#[derive(Deserialize)]
pub struct PageReq {
    pub page: i32,
    pub page_size: i32,
}

#[derive(Error, Debug, Display)]
#[display(fmt = "field: {}, msg: {}", field, msg)]
pub struct PageValidationError {
    pub field: &'static str,
    pub msg: &'static str,
}

impl Validate for PageReq {
    type Item = PageReq;

    type Error = PageValidationError;

    fn validate(self) -> Result<Self::Item, Self::Error> {
        if self.page <= 0 {
            return Err(PageValidationError {
                field: "page",
                msg: "page must be greater than 0",
            });
        };
        if self.page_size <= 0 {
            return Err(PageValidationError {
                field: "page_size",
                msg: "page_size must be greater than 0",
            });
        }
        Ok(self)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Page<T> {
    pub total: i32,
    pub prev: i32,
    pub next: i32,
    pub data: Vec<T>,
}

impl<T> Page<T> {
    pub fn new(total: i32, current: i32, data: Vec<T>, page_size: i32) -> Page<T> {
        Page {
            total,
            prev: current - 1,
            next: if total <= current * page_size {
                0
            } else {
                current + 1
            },
            data,
        }
    }
}
