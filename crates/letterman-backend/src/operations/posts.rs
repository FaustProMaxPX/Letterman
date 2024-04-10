use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl,
    RunQueryDsl,
};

use crate::{
    schema::t_post_content,
    traits::DbAction,
    types::{
        posts::{
            BasePost, CreatePostError, Post, PostContent, QueryPostError, ValidatedPostCreation,
        },
        Page, PageReq,
    },
};

use super::pagination::Paginate;

pub struct PostCreator(pub ValidatedPostCreation);

impl DbAction for PostCreator {
    type Item = usize;

    type Error = CreatePostError;

    fn db_action(
        self,
        conn: &mut diesel::prelude::MysqlConnection,
    ) -> Result<Self::Item, Self::Error> {
        let (post, content) = self.0.to_post_po();
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

pub struct PostPageQueryer(pub PageReq);

impl DbAction for PostPageQueryer {
    type Item = Page<Post>;

    type Error = QueryPostError;

    fn db_action(
        self,
        conn: &mut diesel::prelude::MysqlConnection,
    ) -> Result<Self::Item, Self::Error> {
        let (page, total) = crate::schema::t_post::table
            .order_by(crate::schema::t_post::id.desc())
            .paginate(self.0.page)
            .page_size(self.0.page_size)
            .load_and_count_pages::<BasePost>(conn)?;
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
            .map(|c| (c.post_id, c))
            .collect::<std::collections::HashMap<_, _>>();

        let page = page
            .into_iter()
            .map(|p| {
                let content = contents_map.get(&p.post_id).unwrap();
                Post::new(p, content.clone())
            })
            .collect::<Vec<Post>>();

        Ok(Page::new(total, self.0.page, page, self.0.page_size))
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
        let resp = PostPageQueryer(PageReq {
            page: 1,
            page_size: 10,
        })
        .execute(database_pool().unwrap())
        .await
        .unwrap();
        println!("{:#?}", resp);
    }
}
