use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl,
};

use crate::{
    schema::{self, t_post_content},
    traits::DbAction,
    types::{
        posts::{
            BasePost, CreatePostError, DeletePostError, Post, PostContent, PostPageReq,
            QueryPostError, UpdatePostError, ValidatedPostCreation, ValidatedPostUpdate,
        },
        Page,
    },
    utils::{Snowflake, TimeUtil},
};

use super::pagination::Paginate;

pub struct PostCreator(pub ValidatedPostCreation);

impl DbAction for PostCreator {
    type Item = ();

    type Error = CreatePostError;

    fn db_action(
        self,
        conn: &mut diesel::prelude::MysqlConnection,
    ) -> Result<Self::Item, Self::Error> {
        let (post, content) = self.0.to_post_po();

        insert_post(conn, post, content).map_err(CreatePostError::from)
    }
}

pub struct PostPageQueryer(pub PostPageReq);

impl DbAction for PostPageQueryer {
    type Item = Page<Post>;

    type Error = QueryPostError;

    fn db_action(
        self,
        conn: &mut diesel::prelude::MysqlConnection,
    ) -> Result<Self::Item, Self::Error> {
        use crate::schema::t_post::dsl::*;
        let (page, total) = if self.0.all.is_some() && self.0.all.unwrap() {
            crate::schema::t_post::table
                .order_by(crate::schema::t_post::id.desc())
                .paginate(self.0.page)
                .page_size(self.0.page_size)
                .load_and_count_pages::<BasePost>(conn)?
        } else {
            let max_versions = t_post
                .select(diesel::dsl::sql::<(
                    diesel::sql_types::BigInt,
                    diesel::sql_types::Integer,
                )>("post_id, MAX(version)"))
                .group_by(post_id)
                .load::<(i64, i32)>(conn)?;
            max_versions
                .iter()
                .fold(t_post.into_boxed(), |query, p| {
                    query.or_filter(post_id.eq(p.0).and(version.eq(p.1)))
                })
                .order_by(id.desc())
                .paginate(self.0.page)
                .page_size(self.0.page_size)
                .load_and_count_pages::<BasePost>(conn)?
        };
        let query = page
            .iter()
            .fold(t_post_content::table.into_boxed(), |query, p| {
                query.or_filter(
                    t_post_content::post_id
                        .eq(p.post_id)
                        .and(t_post_content::version.eq(p.version)),
                )
            });
        let contents: Vec<PostContent> = query.load::<PostContent>(conn)?;

        // convert contents to a map. Key is post_id, value is PostContent
        let contents_map = contents
            .into_iter()
            .map(|c| ((c.post_id, c.version), c))
            .collect::<std::collections::HashMap<_, _>>();

        let page = page
            .into_iter()
            .map(|p| {
                let content = contents_map.get(&(p.post_id, p.version)).unwrap();
                Post::new(p, content.clone())
            })
            .collect::<Vec<Post>>();

        Ok(Page::new(total, self.0.page, page, self.0.page_size))
    }
}

pub struct PostUpdater(pub ValidatedPostUpdate);

impl DbAction for PostUpdater {
    type Item = Post;

    type Error = UpdatePostError;

    fn db_action(
        self,
        conn: &mut diesel::prelude::MysqlConnection,
    ) -> Result<Self::Item, Self::Error> {
        let prev: BasePost = schema::t_post::table.find(self.0.id).first(conn)?;
        let prev_latest: BasePost = schema::t_post::table
            .filter(schema::t_post::post_id.eq(prev.post_id))
            .order_by(schema::t_post::version.desc())
            .first(conn)?;
        if prev_latest.version != prev.version {
            return Err(UpdatePostError::NotLatestVersion);
        }
        let new_version = prev.version + 1;
        let base = BasePost {
            id: Snowflake::next_id(),
            post_id: prev.post_id,
            title: self.0.title,
            metadata: self.0.metadata,
            version: new_version,
            prev_version: prev.version,
            create_time: prev.create_time,
            update_time: TimeUtil::now(),
        };
        let content = PostContent {
            id: Snowflake::next_id(),
            post_id: prev.post_id,
            version: new_version,
            content: self.0.content,
            prev_version: prev.version,
            create_time: prev.create_time,
            update_time: TimeUtil::now(),
        };

        insert_post(conn, base.clone(), content.clone())?;
        Ok(Post::new(base, content))
    }
}

fn insert_post(
    conn: &mut MysqlConnection,
    post: BasePost,
    content: PostContent,
) -> Result<(), diesel::result::Error> {
    conn.transaction(|conn| {
        diesel::insert_into(crate::schema::t_post::table)
            .values(&post)
            .execute(conn)?;
        diesel::insert_into(crate::schema::t_post_content::table)
            .values(&content)
            .execute(conn)
    })?;
    Ok(())
}

pub struct PostQueryer(pub i64);

impl DbAction for PostQueryer {
    type Item = Post;

    type Error = QueryPostError;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error> {
        let post: BasePost = schema::t_post::table.find(self.0).first(conn)?;
        let content: PostContent = schema::t_post_content::table
            .filter(
                schema::t_post_content::post_id
                    .eq(post.post_id)
                    .and(schema::t_post_content::version.eq(post.version)),
            )
            .first(conn)?;
        Ok(Post::new(post, content))
    }
}

pub struct PostDeleter(pub i64);

impl DbAction for PostDeleter {
    type Item = ();

    type Error = DeletePostError;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error> {
        use schema::t_post::dsl::*;
        use schema::t_post_content::dsl::*;

        let post: BasePost = schema::t_post::table.find(self.0).first(conn)?;
        let pid = post.post_id;
        let p_version = post.version;
        diesel::delete(t_post.filter(schema::t_post::id.eq(post.id))).execute(conn)?;
        diesel::delete(
            t_post_content.filter(
                schema::t_post_content::post_id
                    .eq(pid)
                    .and(schema::t_post_content::version.eq(p_version)),
            ),
        )
        .execute(conn)?;
        Ok(())
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

    #[actix_rt::test]
    async fn page_query_test() {
        let resp = PostPageQueryer(PostPageReq {
            page: 1,
            page_size: 10,
            all: None,
        })
        .execute(database_pool().unwrap())
        .await
        .unwrap();
        println!("{:#?}", resp);
    }
}
