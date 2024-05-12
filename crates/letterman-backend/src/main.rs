pub mod operations;
pub mod routes;
pub mod schema;
pub mod traits;
pub mod types;
pub mod utils;

use std::{env, error::Error};

use actix_web::{
    middleware::Logger,
    web::{delete, get, post, put, resource, scope, Data},
    App, HttpServer,
};
use diesel::{r2d2::ConnectionManager, MysqlConnection};

use r2d2::Pool;
use routes::{
    common::ping,
    posts::{create, delete_post, get_list, get_post, update},
};

#[macro_use]
extern crate derive_more;

extern crate snowflake;

pub fn database_pool() -> Result<Pool<ConnectionManager<MysqlConnection>>, Box<dyn Error>> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::new(db_url);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}

fn init_config() -> Result<AppConfig, Box<dyn Error>> {
    let github_token = env::var("GITHUB_TOKEN").ok();
    let repository = env::var("GITHUB_REPOSITORY").ok();
    Ok(AppConfig {
        github_token,
        github_repository: repository,
    })
}

fn init_logger() {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}

struct State {
    pub pool: Pool<ConnectionManager<MysqlConnection>>,
}

#[derive(Clone)]
struct AppConfig {
    github_repository: Option<String>,
    github_token: Option<String>,
}

impl AppConfig {
    pub fn get_github_token(&self) -> Option<String> {
        self.github_token.clone()
    }

    pub fn get_github_repository(&self) -> Option<String> {
        self.github_repository.clone()
    }
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    init_logger();
    let config = init_config()?;
    let pool = database_pool()?;
    HttpServer::new(move || {
        let state = State { pool: pool.clone() };

        App::new()
            .app_data(Data::new(state))
            .app_data(Data::new(config.clone()))
            .wrap(Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .service(
                scope("/api/post")
                    .service(resource("/list").route(get().to(get_list)))
                    .service(
                        resource("/{id}")
                            .route(get().to(get_post))
                            .route(delete().to(delete_post)),
                    )
                    .service(
                        resource("")
                            .route(post().to(create))
                            .route(put().to(update)),
                    ),
            )
            .service(ping)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
    .map_err(Into::into)
}
