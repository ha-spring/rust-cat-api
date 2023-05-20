mod models;
mod schema;
use self::models::*;
use self::schema::cats::dsl::*;
use actix_web::{web, App, Error, HttpResponse, HttpServer, Responder};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::sql_types::Integer;
use dotenv::dotenv;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

async fn get_cats(pool: web::Data<DbPool>) -> impl Responder {
    let mut connection = pool.get().expect("Can't get DB connection from pool");
    match cats.limit(100).load::<Cat>(&mut connection) {
        Ok(cats_data) => HttpResponse::Ok().json(cats_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_cat(pool: web::Data<DbPool>, cat_id: web::Path<i32>) -> impl Responder {
    let mut connection = pool.get().expect("Can't get DB connection from pool");
    match cats
        .filter(id.eq(cat_id.into_inner()))
        .first::<Cat>(&mut connection)
    {
        Ok(cat_data) => HttpResponse::Ok().json(cat_data),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Faild to create DB connection pool");
    println!("Listening on http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/cats", web::get().to(get_cats))
            .route("/cat/{id}", web::get().to(get_cat))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
