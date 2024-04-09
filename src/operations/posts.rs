use diesel::{Connection, RunQueryDsl};

use crate::{traits::DbAction, types::posts::{CreatePostError, ValidatedPostCreation}};

pub struct PostCreator(pub ValidatedPostCreation);

impl DbAction for PostCreator {
    type Item = usize;

    type Error = CreatePostError;

    fn db_action(
        self,
        conn: &mut diesel::prelude::MysqlConnection,
    ) -> Result<Self::Item, Self::Error> {
        let (post, content) = self.0.to_post_po();
        println!("post: {:?}, content: {:?}", post, content);
        conn.transaction(|conn| {
            diesel::insert_into(crate::schema::t_post::table)
                .values(&post)
                .execute(conn)?;
            diesel::insert_into(crate::schema::t_post_content::table)
                .values(&content)
                .execute(conn)
        })
        .map_err(CreatePostError::from)
    }
}

#[cfg(test)]
mod post_db_test {


    use crate::database_pool;

    use super::*;

    #[actix_rt::test]
    async fn insert_test() {
        let post = ValidatedPostCreation {
            title: "title".to_string(),
            metadata: "{}".to_string(),
            content: "content".to_string(),
        };
        let pool = database_pool().unwrap();
        PostCreator(post).execute(pool).await.unwrap();
    }
}