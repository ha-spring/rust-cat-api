mod models;
mod schema;
use self::models::*;
use self::schema::cats::dsl::*;
use actix_web::middleware::Logger;
use actix_web::{web, App, Error, HttpResponse, HttpServer, Responder};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sql_types::Integer;
use dotenv::dotenv;
use log::{error, info};
use serde::Deserialize;
use validator::Validate;
use validator_derive::Validate;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

async fn get_cats(pool: web::Data<DbPool>) -> impl Responder {
    info!("get_cats called");
    let mut connection = match pool.get() {
        Ok(connection) => connection,
        Err(_) => {
            error!("Failed to get DB connection from pool");
            return HttpResponse::InternalServerError().finish();
        }
    };

    match cats.limit(100).load::<Cat>(&mut connection) {
        Ok(cats_data) => HttpResponse::Ok().json(cats_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Deserialize, Validate)]
struct CatId {
    #[validate(range(min = 1, max = 150))]
    id: i32,
}

async fn get_cat(pool: web::Data<DbPool>, cat_id: web::Path<CatId>) -> impl Responder {
    if let Err(validation_errors) = cat_id.validate() {
        return HttpResponse::BadRequest().finish();
    }
    let mut connection = match pool.get() {
        Ok(connection) => connection,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    match cats.filter(id.eq(cat_id.id)).first::<Cat>(&mut connection) {
        Ok(cat_data) => HttpResponse::Ok().json(cat_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Faild to create DB connection pool");
    println!("Listening on http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .route("/cats", web::get().to(get_cats))
            .route("/cat/{id}", web::get().to(get_cat))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
