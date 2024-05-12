use std::time::Duration;

use diesel::{r2d2::ConnectionManager, MysqlConnection};
use r2d2::Pool;
use reqwest::header::{self, HeaderValue};

use crate::{
    operations::{
        github_record::{GithubRecordCreator, GithubRecordQueryerByPostId},
        posts::LatestPostQueryerByPostId,
    },
    traits::DbAction,
    types::github_record::{
        CreateContentParam, GithubArticleRecord, GithubSyncError, InsertableGithubRecord,
        WriteContentResp,
    },
};

pub struct GithubSyncer {
    pub post_id: i64,
    /// path is required for the first time sync
    pub path: Option<String>,
    pub pool: Pool<ConnectionManager<MysqlConnection>>,
    pub repository: Option<String>,
    pub token: String,
}

impl GithubSyncer {
    pub async fn sync(self) -> Result<(), GithubSyncError> {
        let post = LatestPostQueryerByPostId(self.post_id)
            .execute(self.pool.clone())
            .await?;
        let records = GithubRecordQueryerByPostId(self.post_id)
            .execute(self.pool.clone())
            .await?;
        let client = init_client(&self.token)?;
        if records.is_empty() {
            if self.path.is_none() || self.repository.is_none() {
                return Err(GithubSyncError::UserError(
                    "path and repository is required for the first time sync".to_string(),
                ));
            }
            let repo = self.repository.unwrap();
            let path = self.path.unwrap();
            let url = format!("https://api.github.com/repos/{repo}/contents/{path}",);
            let param = CreateContentParam::new(format!("create {}", path), post.get_content());
            let resp = client.post(url).json(&param).send().await?;
            if !resp.status().is_success() {
                return Err(GithubSyncError::Request);
            }
            let resp = resp.json::<WriteContentResp>().await?;
            GithubRecordCreator(InsertableGithubRecord::new(
                self.post_id,
                post.get_version(),
                resp.path,
                resp.sha,
                repo.clone(),
                resp.url,
            ))
            .execute(self.pool.clone())
            .await?;
        }
        let latest = records.first().unwrap();
        let url = format!(
            "https://api.github.com/repos/{repo}/contents/{path}",
            repo = latest.repository,
            path = latest.path
        );
        let resp = client.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(GithubSyncError::Request);
        }
        let content = resp.json::<GithubArticleRecord>().await?;
        if latest.sha == content.sha {
            return Ok(());
        }
        if records
            .iter()
            .map(|x| x.sha.clone())
            .any(|x| x == content.sha)
        {
            // TODO: push current version to github
        }
        Err(GithubSyncError::Ambiguous)
    }
}

fn init_client(token: &str) -> Result<reqwest::Client, reqwest::Error> {
    let mut header_map = header::HeaderMap::new();
    header_map.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("letterman"),
    );
    header_map.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );
    header_map.insert(
        header::ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .default_headers(header_map)
        .build()?;
    Ok(client)
}

mod github_sync_test {
    use std::env;

    use super::*;
    use base64::prelude::*;

    #[actix_web::test]
    async fn get_content_test() {
        dotenv::dotenv().ok();
        let github_token = env::var("GITHUB_TOKEN").unwrap_or_default();
        let client = reqwest::Client::new();
        let url = "https://api.github.com/repos/ZephyrZenn/letterman/contents/README.md";
        let resp = client
            .get(url)
            .header("User-Agent", "letterman")
            .header("accept", "application/vnd.github+json")
            .header("Authorization", github_token)
            .send()
            .await
            .unwrap();
        // let content = resp.text().await.unwrap();
        // println!("{:#?}", content);
        let content = resp.json::<GithubArticleRecord>().await.unwrap();
        let content = content.decode_content().unwrap();
        println!("{:?}", content);
    }
}
