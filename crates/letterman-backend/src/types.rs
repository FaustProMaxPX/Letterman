use core::fmt;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use mongodb::bson::Bson;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use crate::traits::Validate;

pub mod github_record;
pub mod posts;

#[derive(Serialize)]
pub struct CommonResult<T> {
    pub success: bool,
    pub code: i32,
    pub message: String,
    pub data: T,
}

impl<T> CommonResult<T> {
    pub fn success() -> CommonResult<()> {
        CommonResult {
            success: true,
            code: 0,
            message: "OK".to_string(),
            data: (),
        }
    }
    pub fn with_msg(success: bool, code: i32, msg: &str) -> CommonResult<()> {
        CommonResult {
            success,
            message: msg.to_string(),
            data: (),
            code,
        }
    }
    pub fn with_data(success: bool, code: i32, msg: &str, data: T) -> CommonResult<T> {
        CommonResult {
            success,
            message: msg.to_string(),
            data,
            code,
        }
    }
    pub fn success_with_data(data: T) -> CommonResult<T> {
        CommonResult::with_data(true, 0, "OK", data)
    }
    pub fn fail() -> CommonResult<()> {
        CommonResult::<()>::with_msg(false, 1, "fail")
    }
    pub fn fail_with_msg(msg: &str) -> CommonResult<()> {
        CommonResult::<()>::with_msg(false, 1, msg)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageReq {
    pub page: i32,
    pub page_size: i32,
}

#[derive(Error, Debug)]
#[error("Invalid Page Request: {field}: {msg}")]
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

fn serialize_as_string<S>(x: &i64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&x.to_string())
}
fn deserialize_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<i64>().map_err(serde::de::Error::custom)
}

fn serialize_metadata<S>(x: &HashMap<String, String>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = serde_json::to_value(x).map_err(serde::ser::Error::custom)?;
    value.serialize(s)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Platform {
    Github,
}

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Platform::Github => write!(f, "Github"),
        }
    }
}

impl From<Platform> for Bson {
    fn from(item: Platform) -> Self {
        match item {
            Platform::Github => Bson::String("Github".to_string()),
        }
    }
}
