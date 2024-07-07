use async_trait::async_trait;
use mongodb::{bson::doc, Cursor};

use crate::{
    traits::{DocumentConvert, MongoAction},
    types::github_record::{GithubRecord, InsertableGithubRecord, QueryGithubRecordError},
    utils,
};

use super::{constants, remote::github::CreateGithubRecordError};

pub struct GithubRecordQueryerByPostId(pub i64);

#[async_trait]
impl MongoAction for GithubRecordQueryerByPostId {
    type Item = Vec<GithubRecord>;

    type Error = QueryGithubRecordError;

    async fn mongo_action(self, db: mongodb::Database) -> Result<Self::Item, Self::Error> {
        let filter = doc! {"post_id": self.0};
        let cursor: Cursor<GithubRecord> = db
            .collection(constants::SYNC_RECORDS_COLLECTION)
            .find(filter, None)
            .await?;
        Ok(utils::mongo_utils::to_vec(cursor).await)
    }
}

pub struct GithubRecordCreator(pub InsertableGithubRecord);

#[async_trait]
impl MongoAction for GithubRecordCreator {
    type Item = ();

    type Error = CreateGithubRecordError;

    async fn mongo_action(self, db: mongodb::Database) -> Result<Self::Item, Self::Error> {
        let doc = self.0.to_doc();
        db.collection(constants::SYNC_RECORDS_COLLECTION)
            .insert_one(doc, None)
            .await?;
        Ok(())
    }
}

impl From<mongodb::error::Error> for CreateGithubRecordError {
    fn from(item: mongodb::error::Error) -> Self {
        CreateGithubRecordError::Database(item)
    }
}

impl From<mongodb::error::Error> for QueryGithubRecordError {
    fn from(value: mongodb::error::Error) -> Self {
        QueryGithubRecordError::Database(value)
    }
}

#[cfg(test)]
mod github_record_test {

    use crate::{database_pool, mongodb_database};

    use super::*;

    #[actix_web::test]
    async fn insert_test() {
        dotenv::dotenv().ok();
        let db = mongodb_database().await.unwrap();
        let _pool = database_pool().unwrap();
        let record = InsertableGithubRecord::new(
            1,
            1,
            String::from("path"),
            String::from("sha"),
            String::from("repository"),
            String::from("url"),
        );
        let res = GithubRecordCreator(record).execute(db.clone()).await;
        assert!(res.is_ok());
    }

    #[actix_web::test]
    async fn query_test() {
        dotenv::dotenv().ok();
        let db = mongodb_database().await.unwrap();
        let _pool = database_pool().unwrap();
        let res = GithubRecordQueryerByPostId(7191634464299159554).execute(db.clone()).await;
        assert!(res.is_ok());
        println!("{:?}", res.unwrap());
    }
}
