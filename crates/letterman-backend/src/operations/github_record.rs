use diesel::prelude::*;

use crate::{
    schema,
    traits::DbAction,
    types::github_record::{
        GithubRecord, InsertableGithubRecord, QueryGithubRecordError,
    },
};

use super::remote::github::CreateGithubRecordError;

pub struct GithubRecordQueryerByPostId(pub i64);

impl DbAction for GithubRecordQueryerByPostId {
    type Item = Vec<GithubRecord>;

    type Error = QueryGithubRecordError;

    fn db_action(
        self,
        conn: &mut diesel::prelude::MysqlConnection,
    ) -> Result<Self::Item, Self::Error> {
        use schema::t_github_post_record::dsl::*;
        let records: Vec<GithubRecord> = t_github_post_record
            .filter(post_id.eq(self.0))
            .order_by(version.desc())
            .load(conn)?;
        Ok(records)
    }
}

pub struct GithubRecordCreator(pub InsertableGithubRecord);

impl DbAction for GithubRecordCreator {
    type Item = ();

    type Error = CreateGithubRecordError;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error> {
        use schema::t_github_post_record::dsl::*;
        diesel::insert_into(t_github_post_record)
            .values(&self.0)
            .execute(conn)?;
        Ok(())
    }
}

impl From<diesel::result::Error> for QueryGithubRecordError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => QueryGithubRecordError::NotFound,
            _ => QueryGithubRecordError::Database,
        }
    }
}

#[cfg(test)]
mod github_record_test {

    use crate::database_pool;

    use super::*;

    #[actix_web::test]
    async fn insert_test() {
        dotenv::dotenv().ok();
        let pool = database_pool().unwrap();
        let record = InsertableGithubRecord::new(
            1,
            1,
            String::from("path"),
            String::from("sha"),
            String::from("repository"),
            String::from("url"),
        );
        let res = GithubRecordCreator(record).execute(pool).await;
        assert!(res.is_ok());
    }
}
