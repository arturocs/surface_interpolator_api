#![allow(dead_code, unused_variables)]
mod curves;
mod surfaces;
use actix_web::{get, web, App, HttpServer, Responder};
use async_once::AsyncOnce;
use futures::TryStreamExt;
use lazy_static::lazy_static;
use mongodb::{bson::doc, Client, Database};
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
lazy_static! {
    static ref MYDB: AsyncOnce<Database> = AsyncOnce::new(async {
        let client = Client::with_uri_str("mongodb://mongo:27017").await.unwrap();
        let database = client.database("mydb");
        database
    });
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ExtraBookInfo {
    pages: u16,
    description: String,
}

impl ExtraBookInfo {
    fn new(pages: u16, description: impl ToString) -> Self {
        Self {
            pages,
            description: description.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Book {
    title: String,
    author: String,
    extra_info: Option<ExtraBookInfo>,
}

impl Book {
    fn new(title: impl ToString, author: impl ToString, extra_info: Option<ExtraBookInfo>) -> Self {
        Book {
            title: title.to_string(),
            author: author.to_string(),
            extra_info,
        }
    }
}

#[get("/hello/{name}")]
async fn hello(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    format!("Hello {}!", name)
}

#[get("/books/{name}")]
async fn books(path: web::Path<String>) -> Result<impl Responder> {
    let name = path.into_inner();
    let cursor = MYDB
        .get()
        .await
        .collection::<Book>("books")
        .find(doc! { "title":name }, None)
        .await?;
    let books: Vec<Book> = cursor.try_collect().await?;
    Ok(web::Json(books))
}

#[tokio::main]
async fn main() -> Result<()> {
    let n_books = MYDB
        .get()
        .await
        .collection::<Book>("books")
        .count_documents(None, None)
        .await?;
    if n_books == 0 {
        let docs = vec![
            Book::new("1984", "George Orwell", None),
            Book::new(
                "Animal Farm",
                "George Orwell",
                Some(ExtraBookInfo::new(112, "The poorly-run Manor Farm near Willingdon, England, is ripened for rebellion...")),
            ),
            Book::new("The Great Gatsby", "F. Scott Fitzgerald", None),
        ];
        println!("Inserting books");
        MYDB.get()
            .await
            .collection::<Book>("books")
            .insert_many(docs, None)
            .await?;
    }
    let url = "0.0.0.0:8000";
    println!("Serving at: {}", url);
    HttpServer::new(|| App::new().service(hello).service(books))
        .bind(url)?
        .run()
        .await?;
    Ok(())
}
