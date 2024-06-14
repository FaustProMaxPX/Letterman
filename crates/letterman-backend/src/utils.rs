pub mod snowflake {

    use std::sync::Mutex;

    use lazy_static::lazy_static;
    use snowflake::SnowflakeIdGenerator;

    lazy_static! {
        static ref ID_GENERATOR: Mutex<SnowflakeIdGenerator> =
            Mutex::new(SnowflakeIdGenerator::new(1, 1));
    }
    pub fn next_id() -> i64 {
        let mut gen = ID_GENERATOR.lock().unwrap();
        gen.real_time_generate()
    }
}

pub mod time_utils {
    use chrono::NaiveDateTime;

    pub fn now() -> NaiveDateTime {
        chrono::Utc::now().naive_local()
    }
}

pub mod mongo_utils {
    use futures::StreamExt;
    use serde::{Deserialize, Serialize};

    pub async fn to_vec<T: for<'a> Deserialize<'a> + Serialize + std::fmt::Debug>(
        cursor: mongodb::Cursor<T>,
    ) -> Vec<T> {
        cursor
            .filter_map(|doc| async {
                match doc {
                    Ok(doc) => Some(doc),
                    Err(_) => {
                        eprintln!("error: {:?}", doc);
                        None
                    }
                }
            })
            .collect()
            .await
    }
}
