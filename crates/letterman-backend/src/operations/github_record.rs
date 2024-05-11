use diesel::prelude::*;

use crate::{
    schema,
    traits::DbAction,
    types::github_record::{GithubRecord, QueryGithubRecordError},
};

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

impl From<diesel::result::Error> for QueryGithubRecordError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => QueryGithubRecordError::NotFound,
            _ => QueryGithubRecordError::Database,
        }
    }
}
