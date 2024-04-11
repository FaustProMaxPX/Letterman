pub struct Snowflake;
use std::sync::Mutex;

use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use snowflake::SnowflakeIdGenerator;

lazy_static! {
    static ref ID_GENERATOR: Mutex<SnowflakeIdGenerator> =
        Mutex::new(SnowflakeIdGenerator::new(1, 1));
}

impl Snowflake {
    pub fn next_id() -> i64 {
        let mut gen = ID_GENERATOR.lock().unwrap();
        gen.real_time_generate()
    }
}

pub struct TimeUtil;

impl TimeUtil {
    pub fn now() -> NaiveDateTime {
        chrono::Utc::now().naive_local()
    }
}
