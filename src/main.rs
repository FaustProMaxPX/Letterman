pub mod routes;
pub mod schema;
pub mod traits;
pub mod types;
pub mod utils;
mod operations;

use std::{env, error::Error};

use actix_web::{web::Data, App, HttpServer};
use diesel::{r2d2::ConnectionManager, MysqlConnection};
use r2d2::Pool;

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

struct State {
    pub pool: Pool<ConnectionManager<MysqlConnection>>,
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = database_pool()?;
    HttpServer::new(move || {
        let state = State { pool: pool.clone() };

        App::new().app_data(Data::new(state)).wrap(
            actix_cors::Cors::default()
                .allow_any_header()
                .allow_any_method()
                .allow_any_origin(),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
    .map_err(Into::into)
}
