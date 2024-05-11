use std::time::Duration;

use actix_web::web::Data;

use crate::{
    operations::github_record::GithubRecordQueryerByPostId,
    traits::DbAction,
    types::github_record::{GithubArticleRecord, GithubSyncError},
    AppConfig, State,
};

pub struct GithubSyncer(pub i64, Data<State>, Data<AppConfig>);

impl GithubSyncer {
    pub async fn sync(self) -> Result<(), GithubSyncError> {
        let records = GithubRecordQueryerByPostId(self.0)
            .execute(self.1.pool.clone())
            .await?;
        if records.is_empty() {
            // TODO: push to github
        }
        let latest = records.first().unwrap();
        let url = format!(
            "https://api.github.com/repos/{repo}/contents/{path}",
            repo = latest.repository,
            path = latest.path
        );
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()?;
        let resp = client
            .get(url)
            .header("User-Agent", "letterman")
            .header("accept", "application/vnd.github+json")
            .header("Authorization", self.2.get_github_token())
            .send()
            .await?;
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
        // TODO: force mode
        Ok(())
    }
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
