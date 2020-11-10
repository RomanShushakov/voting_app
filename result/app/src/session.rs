use std::time::{Duration, Instant};

use actix::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;


use crate::server;


const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);


pub async fn start_ws(
        req: HttpRequest, stream: web::Payload,
        srv: web::Data<Addr<server::WebsocketServer>>,
    )
    -> Result<HttpResponse, Error>
{
    ws::start(
        WsChatSession
        {
            id: 0,
            hb: Instant::now(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}


struct WsChatSession
{
    id: usize,
    hb: Instant,
    addr: Addr<server::WebsocketServer>,
}


impl Actor for WsChatSession
{
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context)
    {
        self.hb(ctx);
        let addr = ctx.address();
        self.addr
            .send(server::Connect
            {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx|
                {
                    match res
                    {
                        Ok(res) => act.id = res,
                        _ => ctx.stop(),
                    }
                    fut::ready(())
                })
            .wait(ctx);
    }


    fn stopping(&mut self, _: &mut Self::Context) -> Running
    {
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}


impl Handler<server::Message> for WsChatSession
{
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context)
    {
        ctx.text(msg.0);
    }
}


impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession
{
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context,)
    {
        let msg = match msg
        {
            Err(_) =>
                {
                    ctx.stop();
                    return;
                },
            Ok(msg) => msg,
        };

        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg
        {
            ws::Message::Ping(msg) =>
                {
                    self.hb = Instant::now();
                    ctx.pong(&msg);
                },
            ws::Message::Pong(_) =>
                {
                    self.hb = Instant::now();
                },
            ws::Message::Text(_) => println!("Stats was sent"),
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) =>
                {
                    ctx.close(reason);
                    ctx.stop();
                },
            ws::Message::Continuation(_) =>
                {
                    ctx.stop();
                },
            ws::Message::Nop => (),
        }
    }
}


impl WsChatSession
{
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>)
    {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx|
            {
                if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT
                {
                    println!("Websocket Client heartbeat failed, disconnecting!");
                    act.addr.do_send(server::Disconnect { id: act.id });
                    ctx.stop();
                    return;
                }
                ctx.ping(b"");
            });
    }
}
