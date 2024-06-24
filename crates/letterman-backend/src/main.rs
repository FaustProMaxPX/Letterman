pub mod operations;
pub mod routes;
pub mod schema;
pub mod traits;
pub mod types;
pub mod utils;

use std::env;

use actix_web::{
    middleware::Logger,
    web::{delete, get, post, put, resource, scope, Data},
    App, HttpServer,
};
use diesel::{r2d2::ConnectionManager, MysqlConnection};

use mongodb::options::ClientOptions;
use r2d2::Pool;
use routes::{
    common::ping,
    posts::{
        create, delete_post, force_pull, force_push, get_latest_sync_records, get_list, get_post,
        get_sync_records, synchronize, update,
    },
};

extern crate snowflake;

pub fn database_pool(
) -> Result<Pool<ConnectionManager<MysqlConnection>>, Box<dyn std::error::Error>> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::new(db_url);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}

pub async fn mongodb_database() -> Result<mongodb::Database, Box<dyn std::error::Error>> {
    let uri = env::var("MONGODB_CONNECT_STRING").expect("MONGODB_CONNECT_STRING must be set");
    let mut options = ClientOptions::parse(uri).await?;
    options.max_pool_size = Some(20);
    options.min_pool_size = Some(5);
    let client = mongodb::Client::with_options(options)?;
    Ok(client.database("letterman"))
}

fn init_logger() {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}

struct State {
    pub pool: Pool<ConnectionManager<MysqlConnection>>,
    pub mongodb_database: mongodb::Database,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    init_logger();
    let pool = database_pool()?;
    let mongodb_databse = mongodb_database().await?;
    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").map_or(8080_u16, |v| v.parse::<u16>().unwrap());

    HttpServer::new(move || {
        let state = State {
            pool: pool.clone(),
            mongodb_database: mongodb_databse.clone(),
        };

        App::new()
            .app_data(Data::new(state))
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
                    )
                    .service(
                        scope("sync/{post_id}")
                            .service(resource("synchronize").route(put().to(synchronize)))
                            .service(resource("push").route(put().to(force_push)))
                            .service(resource("pull").route(put().to(force_pull)))
                            .service(resource("records").route(get().to(get_sync_records)))
                            .service(
                                resource("records/latest").route(get().to(get_latest_sync_records)),
                            ),
                    ),
            )
            .service(ping)
    })
    .bind((host, port))?
    .run()
    .await
    .map_err(Into::into)
}
