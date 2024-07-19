use std::collections::HashMap;

use async_trait::async_trait;
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, MysqlConnection, QueryDsl, RunQueryDsl,
};
use mongodb::{bson::doc, options::FindOptions, Cursor};

use crate::{
    schema::{self, t_post_content},
    traits::{DbAction, MongoAction},
    types::{
        posts::{
            BasePost, CreatePostError, DeletePostError, InsertableBasePost, InsertablePostContent,
            Post, PostContent, PostPageReq, QueryPostError, QuerySyncRecordError, RevertPostError,
            SyncRecord, UpdatePostError, ValidatedPostCreation, ValidatedPostUpdate,
        },
        Page, Platform,
    },
    utils::{self},
};

use super::{constants, pagination::Paginate};

pub struct PostCreator(pub ValidatedPostCreation);

impl DbAction for PostCreator {
    type Item = ();

    type Error = CreatePostError;

    fn db_action(
        self,
        conn: &mut diesel::prelude::MysqlConnection,
    ) -> Result<Self::Item, Self::Error> {
        let (post, content) = self.0.to_post_po(None);

        insert_post(conn, post, content).map_err(CreatePostError::from)
    }
}

pub struct PostDirectCreator(pub Post);

impl DbAction for PostDirectCreator {
    type Item = ();

    type Error = CreatePostError;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error> {
        let (base, content) = self.0.to_po(true);
        conn.transaction(|conn| {
            remove_old_head(base.post_id, conn)?;
            insert_post(conn, base, content).map_err(CreatePostError::from)
        })
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
            schema::t_post::table
                .filter(schema::t_post::head.eq(true))
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
                        .and(t_post_content::version.eq(&p.version)),
                )
            });
        let contents: Vec<PostContent> = query.load::<PostContent>(conn)?;

        // convert contents to a map. Key is post_id, value is PostContent
        let contents_map = contents
            .into_iter()
            .map(|c| ((c.post_id, c.version.clone()), c))
            .collect::<std::collections::HashMap<_, _>>();

        let page = page
            .into_iter()
            .map(|p| {
                let content = contents_map.get(&(p.post_id, p.version.clone())).unwrap();
                Post::package(p, content.clone())
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
            .filter(
                schema::t_post::post_id
                    .eq(prev.post_id)
                    .and(schema::t_post::head.eq(true)),
            )
            .first(conn)?;
        if prev_latest.version != prev.version {
            return Err(UpdatePostError::NotLatestVersion);
        }
        let new_version =
            utils::sha_utils::sha_post(&self.0.title, &self.0.metadata, &self.0.content);
        let base = InsertableBasePost {
            id: utils::snowflake::next_id(),
            post_id: prev.post_id,
            title: self.0.title,
            metadata: self.0.metadata,
            version: new_version.clone(),
            prev_version: prev.version.clone(),
            head: true,
        };
        let content = InsertablePostContent {
            id: utils::snowflake::next_id(),
            post_id: prev.post_id,
            version: new_version,
            content: self.0.content,
            prev_version: prev.version,
            head: true,
        };
        conn.transaction(|conn| {
            remove_old_head(base.post_id, conn)?;

            insert_post(conn, base.clone(), content.clone()).map_err(UpdatePostError::from)
        })?;
        Ok((base, content).into())
    }
}

fn insert_post(
    conn: &mut MysqlConnection,
    post: InsertableBasePost,
    content: InsertablePostContent,
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

fn remove_old_head(post_id: i64, conn: &mut MysqlConnection) -> Result<(), diesel::result::Error> {
    diesel::update(schema::t_post::table)
        .filter(
            schema::t_post::post_id
                .eq(post_id)
                .and(schema::t_post::head.eq(true)),
        )
        .set(schema::t_post::head.eq(false))
        .execute(conn)?;
    diesel::update(schema::t_post_content::table)
        .filter(
            schema::t_post_content::post_id
                .eq(post_id)
                .and(schema::t_post_content::head.eq(true)),
        )
        .set(schema::t_post_content::head.eq(false))
        .execute(conn)?;
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
                    .and(schema::t_post_content::version.eq(&post.version)),
            )
            .first(conn)?;
        Ok(Post::package(post, content))
    }
}

pub struct BatchPostQueryerByPostIdAndVersion(pub Vec<(i64, String)>);

impl DbAction for BatchPostQueryerByPostIdAndVersion {
    type Item = HashMap<(i64, String), Post>;
    type Error = QueryPostError;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error> {
        use schema::t_post::dsl::*;
        let list: Vec<BasePost> = self
            .0
            .iter()
            .fold(t_post.into_boxed(), |query, p| {
                query.or_filter(post_id.eq(p.0).and(version.eq(&p.1)))
            })
            .load(conn)?;
        let query = list
            .iter()
            .fold(t_post_content::table.into_boxed(), |query, p| {
                query.or_filter(
                    t_post_content::post_id
                        .eq(p.post_id)
                        .and(t_post_content::version.eq(&p.version)),
                )
            });
        let contents: Vec<PostContent> = query.load(conn)?;
        let contents_map = contents
            .into_iter()
            .map(|c| ((c.post_id, c.version.clone()), c))
            .collect::<std::collections::HashMap<_, _>>();

        let ret: HashMap<(i64, String), Post> = list
            .into_iter()
            .map(|p| {
                let content = contents_map.get(&(p.post_id, p.version.clone())).unwrap();
                Post::package(p, content.clone())
            })
            .map(|p| ((p.post_id(), p.version().to_string()), p))
            .collect();
        Ok(ret)
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

pub struct LatestPostQueryerByPostId(pub i64);

impl DbAction for LatestPostQueryerByPostId {
    type Item = Post;

    type Error = QueryPostError;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error> {
        use schema::t_post::dsl::*;
        use schema::t_post_content::dsl::*;
        let base: BasePost = t_post
            .filter(
                schema::t_post::post_id
                    .eq(self.0)
                    .and(schema::t_post::head.eq(true)),
            )
            .first(conn)?;
        let post_content: PostContent = t_post_content
            .filter(
                schema::t_post_content::post_id
                    .eq(self.0)
                    .and(schema::t_post_content::version.eq(&base.version)),
            )
            .first(conn)?;
        Ok(Post::package(base, post_content))
    }
}

pub struct LatestPostQueryerByPostIds(pub Vec<i64>);

impl DbAction for LatestPostQueryerByPostIds {
    type Item = Vec<Post>;

    type Error = QueryPostError;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error> {
        use schema::t_post::dsl::*;
        let base_posts: Vec<BasePost> = t_post
            .filter(
                schema::t_post::post_id
                    .eq_any(self.0)
                    .and(schema::t_post::head.eq(true)),
            )
            .order_by(id.desc())
            .load(conn)?;
        let query = base_posts
            .iter()
            .fold(t_post_content::table.into_boxed(), |query, p| {
                query.or_filter(
                    t_post_content::post_id
                        .eq(p.post_id)
                        .and(t_post_content::version.eq(&p.version)),
                )
            });
        let contents: Vec<PostContent> = query.load::<PostContent>(conn)?;

        // convert contents to a map. Key is post_id, value is PostContent
        let contents_map = contents
            .into_iter()
            .map(|c| ((c.post_id, c.version.clone()), c))
            .collect::<std::collections::HashMap<_, _>>();

        let posts = base_posts
            .into_iter()
            .map(|p| {
                let content = contents_map.get(&(p.post_id, p.version.clone())).unwrap();
                Post::package(p, content.clone())
            })
            .collect::<Vec<Post>>();
        Ok(posts)
    }
}

pub struct PostQueryerByPostId(pub i64, pub PostPageReq);

impl DbAction for PostQueryerByPostId {
    type Item = Page<Post>;

    type Error = QueryPostError;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error> {
        use schema::t_post::dsl::*;
        let (bases, total) = t_post
            .filter(post_id.eq(self.0))
            .order_by(id.desc())
            .paginate(self.1.page)
            .page_size(self.1.page_size)
            .load_and_count_pages::<BasePost>(conn)?;
        use schema::t_post_content;
        let content: Vec<PostContent> = t_post_content::table
            .filter(t_post_content::post_id.eq(self.0))
            .load(conn)?;
        let content_map = content
            .into_iter()
            .map(|c| ((c.post_id, c.version.clone()), c))
            .collect::<std::collections::HashMap<_, _>>();
        let data = bases
            .into_iter()
            .map(|b| {
                Post::package(
                    b.clone(),
                    content_map
                        .get(&(b.post_id, b.version.clone()))
                        .unwrap()
                        .clone(),
                )
            })
            .collect::<Vec<Post>>();
        let len = data.len() as i32;
        Ok(Page::new(total, self.1.page, data, len))
    }
}

pub struct PagePostSyncRecordQueryer(pub i64, pub i32, pub i32, pub Platform);

#[async_trait]
impl MongoAction for PagePostSyncRecordQueryer {
    type Item = Page<SyncRecord>;

    type Error = QuerySyncRecordError;

    async fn mongo_action(self, db: mongodb::Database) -> Result<Self::Item, Self::Error> {
        let filter = doc! {"post_id": self.0, "platform": self.3.to_string()};

        let skip = (self.1 - 1) * self.2;
        let cursor: Cursor<SyncRecord> = db
            .collection(constants::SYNC_RECORDS_COLLECTION)
            .find(
                filter.clone(),
                FindOptions::builder()
                    .skip(skip as u64)
                    .limit(self.2 as i64)
                    .build(),
            )
            .await?;
        // TODO: 这里的操作不是原子性的
        let total = db
            .collection::<SyncRecord>(constants::SYNC_RECORDS_COLLECTION)
            .count_documents(filter.clone(), None)
            .await?;
        let records = utils::mongo_utils::to_vec(cursor).await;
        let len = records.len() as i32;

        Ok(Page::new(total as i32, self.1, records, len))
    }
}

pub struct PostLatestSyncRecordQueryer(pub i64);

#[async_trait]
impl MongoAction for PostLatestSyncRecordQueryer {
    type Item = Vec<SyncRecord>;
    type Error = QuerySyncRecordError;

    async fn mongo_action(self, db: mongodb::Database) -> Result<Self::Item, Self::Error> {
        let pipeline = vec![
            doc! { "$match": {"post_id": self.0}},
            doc! {"$sort": {"create_time": -1}},
            doc! {
             "$group": {
                 "_id": "$platform",
                    "record": {"$first": "$$ROOT"}
             }
            },
        ];
        let cursor = db
            .collection::<bson::Document>(constants::SYNC_RECORDS_COLLECTION)
            .aggregate(pipeline, None)
            .await?;
        let records = utils::mongo_utils::to_vec(cursor).await;
        let records = records
            .into_iter()
            .filter_map(|record| {
                let record = record.get_document("record").unwrap();
                let sync_record = bson::from_bson(bson::Bson::Document(record.clone()));
                match sync_record {
                    Ok(r) => Some(r),
                    Err(e) => {
                        eprintln!("error: {}", e);
                        None
                    }
                }
            })
            .collect();
        Ok(records)
    }
}

pub struct PostReverter(pub i64);

impl DbAction for PostReverter {
    type Item = ();

    type Error = RevertPostError;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error> {
        use schema::t_post::dsl::*;
        let base: BasePost = t_post.find(self.0).first(conn)?;
        let content: PostContent = schema::t_post_content::table
            .filter(
                schema::t_post_content::post_id
                    .eq(base.post_id)
                    .and(schema::t_post_content::version.eq(&base.version)),
            )
            .first(conn)?;
        conn.transaction(|conn| {
            diesel::delete(t_post)
                .filter(create_time.gt(&base.create_time))
                .execute(conn)?;
            diesel::delete(schema::t_post_content::table)
                .filter(schema::t_post_content::create_time.gt(&content.create_time))
                .execute(conn)?;
            diesel::update(t_post)
                .filter(id.eq(base.id))
                .set(head.eq(true))
                .execute(conn)?;
            diesel::update(schema::t_post_content::table)
                .filter(schema::t_post_content::id.eq(content.id))
                .set(schema::t_post_content::head.eq(true))
                .execute(conn)
        })?;
        Ok(())
    }
}

pub struct PostQueryerByPostIdAndVersion(pub i64, pub String);

impl DbAction for PostQueryerByPostIdAndVersion {
    type Item = Post;
    type Error = QueryPostError;

    fn db_action(self, conn: &mut MysqlConnection) -> Result<Self::Item, Self::Error> {
        use schema::t_post::dsl::*;
        let base: BasePost = t_post
            .filter(post_id.eq(self.0).and(version.eq(self.1.clone())))
            .first(conn)?;
        let content: PostContent = schema::t_post_content::table
            .filter(
                schema::t_post_content::post_id
                    .eq(self.0)
                    .and(schema::t_post_content::version.eq(self.1)),
            )
            .first(conn)?;
        Ok(Post::package(base, content))
    }
}

#[cfg(test)]
mod post_db_test {

    use crate::{database_pool, mongodb_database};

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

    #[actix_rt::test]
    async fn query_sync_record_test() {
        dotenv::dotenv().ok();
        let db = mongodb_database().await.unwrap();
        let page = PagePostSyncRecordQueryer(1, 1, 10, Platform::Github)
            .execute(db)
            .await;
        assert!(page.is_ok());
    }
}
