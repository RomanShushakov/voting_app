use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};
use std::collections::HashMap;
use actix_web::web;
use mongodb;
use std::time::Duration;
use serde_json;

use crate::models::{VoteStats, WsResponse};


const STATS_UPDATE_INTERVAL: Duration = Duration::from_secs(1);
const VOTE_VARIANTS: [&'static str; 2] = ["a", "b"];


#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);


#[derive(Message)]
#[rtype(usize)]
pub struct Connect
{
    pub addr: Recipient<Message>,
}


#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect
{
    pub id: usize,
}


#[derive(Clone)]
struct SessionData
{
    recipient: Recipient<Message>,
}


pub struct WebsocketServer
{
    sessions: HashMap<usize, SessionData>,
    rng: ThreadRng,
    client: web::Data<mongodb::sync::Client>,
}


impl Default for WebsocketServer
{
    fn default() -> WebsocketServer
    {
        dotenv::dotenv().ok();
        let mongodb_addr = std::env::var("MONGODB_ADDR").expect("MONGODB_ADDR must be set");
        let client = web::Data::new(mongodb::sync::Client::with_uri_str(&mongodb_addr).unwrap());

        WebsocketServer
        {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
            client,
        }
    }
}


impl WebsocketServer
{
    fn get_statistics(&self, ctx: &mut Context<Self>)
    {
        ctx.run_interval(STATS_UPDATE_INTERVAL, |act, _ctx|
            {
                dotenv::dotenv().ok();
                let mongodb_db_name = std::env::var("MONGODB_DB_NAME").expect("MONGODB_DB_NAME must be set");
                let mongodb_collection_name = std::env::var("MONGODB_COLLECTION_NAME").expect("MONGODB_COLLECTION_NAME must be set");
                let collection = act.client.database(&mongodb_db_name).collection(&mongodb_collection_name);
                let mut statistics = Vec::new();
                for vote_variant in VOTE_VARIANTS.iter()
                {
                    let filter = mongodb::bson::doc! { "vote": vote_variant };
                    if let Ok(quantity) = collection.count_documents(filter, None)
                    {
                        let vote_stats = VoteStats { vote: vote_variant.to_string(), quantity: quantity as u64 };
                        statistics.push(vote_stats);
                    }
                }
                for (_id, session_data) in act.sessions.iter()
                {
                    let response = WsResponse { action: "received_statistics".to_owned(), data: serde_json::to_string(&statistics).unwrap() };
                    let m = serde_json::to_string(&response).unwrap();
                    let _ = session_data.recipient.do_send(Message(m));
                }
            });
    }
}


impl Actor for WebsocketServer
{
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context)
    {
        self.get_statistics(ctx);
    }
}


impl Handler<Connect> for WebsocketServer
{
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result
    {
        println!("Someone connected");

        let id = self.rng.gen::<usize>();
        self.sessions.insert(
            id,
            SessionData { recipient: msg.addr }
        );
        id
    }
}


impl Handler<Disconnect> for WebsocketServer
{
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>)
    {
        println!("Someone disconnected");
        self.sessions.remove(&msg.id);
    }
}
