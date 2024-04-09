pub mod operations;
pub mod routes;
pub mod schema;
pub mod traits;
pub mod types;
pub mod utils;

use std::{env, error::Error};

use actix_web::{
    middleware::Logger, web::{post, resource, scope, Data}, App, HttpServer
};
use diesel::{r2d2::ConnectionManager, MysqlConnection};

use r2d2::Pool;
use routes::{common::ping, posts::create};

#[macro_use]
extern crate derive_more;

extern crate snowflake;

pub fn database_pool() -> Result<Pool<ConnectionManager<MysqlConnection>>, Box<dyn Error>> {
    dotenv::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::new(db_url);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}

fn init_logger() {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
}

struct State {
    pub pool: Pool<ConnectionManager<MysqlConnection>>,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    init_logger();
    let pool = database_pool()?;
    HttpServer::new(move || {
        let state = State { pool: pool.clone() };

        App::new()
            .app_data(Data::new(state))
            .wrap(Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .service(scope("/api/posts").service(resource("").route(post().to(create))))
            .service(ping)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
    .map_err(Into::into)
}
