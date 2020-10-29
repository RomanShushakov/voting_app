use actix_web::
    {
        HttpServer, App, web, HttpResponse, HttpRequest, HttpMessage, error,
        dev::HttpResponseBuilder, http::header, http::StatusCode,
    };
use actix_files::Files;
use serde::Deserialize;
use derive_more::{Display, Error};


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


#[derive(Debug, Deserialize)]
struct VoteRequest
{
    voter_id: String,
    vote: String,
}


async fn vote(request: HttpRequest, vote_request: web::Json<VoteRequest>) -> Result<HttpResponse, MyError>
{
    if let Ok(cookies) = request.cookies()
    {
        for cookie in cookies.iter()
        {
            if cookie.name() =="voter_id" && cookie.value() == vote_request.voter_id
            {
                return Ok(HttpResponse::Ok().body("Vote registered."))
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
    let bind = "0.0.0.0:8080";
    println!("Starting server at: {}", &bind);

    HttpServer::new(move ||
        {
            App::new()
                .route("/", web::post().to(vote))
                .service(Files::new("", "./web_layout").index_file("index.html"))
        })
    .bind(&bind)?
    .run()
    .await
}
