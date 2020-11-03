use serde::Deserialize;
use serde_json;
use redis::AsyncCommands;
use mongodb;


#[derive(Debug, Deserialize)]
struct Vote
{
    voter_id: String,
    vote: String,
}


async fn connect_to_redis() -> redis::RedisResult<redis::aio::Connection>
{
    dotenv::dotenv().ok();
    let redis_addr = std::env::var("REDIS_ADDR").expect("REDIS_ADDR must be set");
    let client = redis::Client::open(redis_addr)?;
    let connection = client.get_async_connection().await?;
    Ok(connection)
}


async fn connect_to_mongodb() -> mongodb::error::Result<mongodb::Client>
{
    dotenv::dotenv().ok();
    let mongodb_addr = std::env::var("MONGODB_ADDR").expect("MONGODB_ADDR must be set");
    let client = mongodb::Client::with_uri_str(&mongodb_addr).await?;
    Ok(client)
}


#[tokio::main]
async fn main()
{
    dotenv::dotenv().ok();
    let redis_key = std::env::var("REDIS_KEY").expect("REDIS_KEY must be set");
    let mongodb_db_name = std::env::var("MONGODB_DB_NAME").expect("MONGODB_DB_NAME must be set");
    let mongodb_collection_name = std::env::var("MONGODB_COLLECTION_NAME").expect("MONGODB_COLLECTION_NAME must be set");
    if let Ok(mut connection) = connect_to_redis().await
    {
        if let Ok(client) = connect_to_mongodb().await
        {
            let database = client.database(&mongodb_db_name);
            let collection = database.collection(&mongodb_collection_name);
            loop
            {
                if let Ok(data) = connection.lpop::<_, String>(&redis_key).await
                {
                    let vote: Vote = serde_json::from_str(&data).unwrap();

                    let filter = mongodb::bson::doc! { "voter_id": vote.voter_id };
                    let updated_document = mongodb::bson::doc! { "$set": { "vote": vote.vote } };
                    let update_modifications = mongodb::options::UpdateModifications::Document(updated_document);
                    let find_one_and_update_options = mongodb::options::FindOneAndUpdateOptions::builder().upsert(true).build();
                    if let Ok(doc) = collection.find_one_and_update(filter, update_modifications, find_one_and_update_options).await
                    {
                        if let Some(_) = doc
                        {
                            println!("Vote was updated.");
                        }
                        else
                        {
                            println!("New vote was registered.")
                        }
                    }
                }
                else
                {
                    println!("No votes in redis.");
                }
                tokio::time::delay_for(tokio::time::Duration::from_millis(500)).await;
            }
        }
        else
        {
            println!("Could not connect to mongodb!!!");
        }
    }
    else
    {
        println!("Could not connect to redis!!!");
    }
}
