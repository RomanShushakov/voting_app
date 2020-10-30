use actix_web::
    {
        HttpServer, App, web, HttpResponse, HttpRequest, HttpMessage, error,
        dev::HttpResponseBuilder, http::header, http::StatusCode, middleware,
        Error as AWError,
    };
use actix_files::Files;
use serde::{Deserialize, Serialize};
use derive_more::{Display, Error};
use actix_redis::{Command, RedisActor};
use actix::prelude::*;
use futures::future::join_all;
use redis_async::{resp::RespValue, resp_array};
use serde_json;


#[derive(Debug, Display, Error)]
enum MyError
{
    #[display(fmt = "Unauthorized")]
    Unauthorized,
}


impl error::ResponseError for MyError
{
    fn error_response(&self) -> HttpResponse
    {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode
    {
        match *self
        {
            MyError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }
}


#[derive(Debug, Deserialize, Serialize)]
struct VoteRequest
{
    voter_id: String,
    vote: String,
}


async fn vote(request: HttpRequest, vote_request: web::Json<VoteRequest>, redis: web::Data<Addr<RedisActor>>)
    -> Result<HttpResponse, MyError>
{
    if let Ok(cookies) = request.cookies()
    {
        for cookie in cookies.iter()
        {
            if cookie.name() =="voter_id" && cookie.value() == vote_request.voter_id
            {
                let vote = serde_json::to_string(&vote_request.into_inner()).unwrap();
                let cmd = redis.send(Command(resp_array!["RPUSH", "votes", vote]));
                let res: Vec<Result<RespValue, AWError>> =
                    join_all(vec![cmd].into_iter())
                        .await
                        .into_iter()
                        .map(|item|
                            {
                                item.map_err(AWError::from)
                                    .and_then(|res| res.map_err(AWError::from))
                            })
                        .collect();

                return if !res.iter().all(|res| match res
                    {
                        Ok(RespValue::Integer(_)) => true,
                        _ => false,
                    })
                {
                    Err(MyError::Unauthorized)
                }
                else
                {
                    Ok(HttpResponse::Ok().body("Your vote was registered."))
                }
            }
        }
        Err(MyError::Unauthorized)
    }
    else
    {
        Err(MyError::Unauthorized)
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()>
{
    std::env::set_var("RUST_LOG", "actix_web=trace,actix_redis=trace");
    env_logger::init();

    let bind = "0.0.0.0:8080";
    println!("Starting server at: {}", &bind);

    dotenv::dotenv().ok();
    let redis_addr = std::env::var("REDIS_ADDR").expect("REDIS_ADDR must be set");

    HttpServer::new(move ||
        {
            let redis_addr = RedisActor::start(&redis_addr);
            App::new()
                .data(redis_addr)
                .wrap(middleware::Logger::default())
                .route("/", web::post().to(vote))
                .service(Files::new("", "./web_layout").index_file("index.html"))
        })
    .bind(&bind)?
    .run()
    .await
}
