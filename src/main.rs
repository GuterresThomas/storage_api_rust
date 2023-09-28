use warp::{Filter, filters::query};
use tokio_postgres::{NoTls, Error, Client};
use std::sync::Arc;
use warp::reject::custom;

// Define um tipo de erro personalizado que implementa Reject
#[derive(Debug)]
struct CustomError(String);

impl warp::reject::Reject for CustomError {}

// Define uma estrutura de dados para o item
#[derive(serde::Deserialize, serde::Serialize)]
struct StorageItem {
    id: i32,
    name: String,
    amount: String,
    price: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=1234 dbname=postgres", NoTls)
            .await?;
    tokio::spawn(connection);

    let client = Arc::new(client);

    let db = warp::any().map(move || client.clone());


    let create_item = warp::post()
    .and(warp::path("items"))
    .and(warp::body::json())
    .and(db.clone())
    .and_then(| item: StorageItem, client: Arc<Client>| async move {
        let insert_query = format!("INSERT INTO storage_items (name, amount, price) VALUES ('{}', '{}', '{}')", item.name, item.amount, item.price);
        match client.execute(&insert_query, &[]).await {
            Ok(rows) if rows == 1 => {
                 Ok(warp::reply::json(&item))
                }
            _ => {
                let error_message = "Failed to add item".to_string();
                Err(custom(CustomError(error_message)))
            },
        }
    });

    let get_items = warp::get()
    .and(warp::path("items"))
    .and(db.clone())
    .and_then(|client: Arc<Client>| async move {
        let query = format!("SELECT id, name, amount, price FROM storage_items");

        match client.query(&query, &[]).await {
            Ok(rows) => {
                let items: Vec<StorageItem> = rows
                .into_iter()
                .map(|row | StorageItem {
                    id: row.get("id"),
                    name: row.get("name"),
                    amount: row.get("amount"),
                    price: row.get("price"),
                })
                .collect();

            Ok(warp::reply::json(&items))
            }
            Err(err) => {
                let error_message = format!("Error to fetch items: {}", err);
                Err(custom(CustomError(error_message)))
            }
        }
    });




    Ok(())
}
