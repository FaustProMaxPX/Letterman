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

pub mod sha_utils {

    use std::collections::HashMap;

    use base64::Engine;
    use sha256::digest;

    pub fn sha(input: &str) -> String {
        digest(input)
    }

    pub fn sha_post(title: &str, metadata: &str, content: &str) -> String {
        let input = format!("---\n{}\n---\n{}\n{}", metadata, title, content);
        sha(&input)
    }

    pub fn sha_post2(
        title: &str,
        metadata: &HashMap<String, Vec<String>>,
        content: &str,
    ) -> String {
        let metadata = serde_yaml::to_string(metadata).unwrap();
        let input = format!("{}\n{}\n{}", metadata, title, content);
        sha(&input)
    }

    #[test]
    fn sha_post_test() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), vec!["1".to_string()]);
        let title = "aaa";
        let content = "test";
        let sha1 = sha_post2(title, &map, content);

        let sha2 = sha_post2(title, &map, content);
        assert_eq!(sha1, sha2)
    }

    #[test]
    fn base64_test() {
        base64::prelude::BASE64_STANDARD.decode("LS0tCnkxOiAnMScKdGl0bGU6IOi/meaYr+S4gOevh+a1i+ivleaWh+eroAoKLS0tCgojIFRFU1QKCua1i+ivleS4gOS4i1BVTEwKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgoKCgo=").unwrap();
    }
}

