#![recursion_limit="512"]
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use lazy_static;
use std::collections::HashMap;
use yew::format::Json;
use anyhow::Error;
use serde::Deserialize;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use dotenv_codegen::dotenv;
use serde_json;


pub const WEBSOCKET_URL: &str = dotenv!("WEBSOCKET_URL");


lazy_static::lazy_static!
{
    static ref VOTE_VARIANTS: HashMap<&'static str, &'static str> =
        {
            let mut h_m = HashMap::new();
            h_m.insert("a", "Cats");
            h_m.insert("b", "Dogs");
            h_m
        };
}


#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct VoteStats
{
    pub vote: String,
    pub quantity: u64,
}


struct State
{
    data: Vec<VoteStats>,
    total_votes: u64,
    is_connected: bool,
}


struct Model
{
    link: ComponentLink<Self>,
    state: State,
    websocket_task: Option<WebSocketTask>,
}


#[derive(Debug, Deserialize)]
pub struct WsResponse
{
    pub action: String,
    pub data: String,
}


pub enum WsAction
{
    Connect,
    Disconnect,
    Lost,
}


pub enum Msg
{
    WsAction(WsAction),
    WsReady(Result<WsResponse, Error>),
    Ignore,
}


impl From<WsAction> for Msg
{
    fn from(action: WsAction) -> Self
    {
        Msg::WsAction(action)
    }
}


enum WsResponseAction
{
    ReceivedStatistics,
}


impl WsResponseAction
{
    pub fn as_str(&self) -> String
    {
        match self
        {
            WsResponseAction::ReceivedStatistics => String::from("received_statistics"),
        }
    }
}




impl Model
{
    fn votes_percent(&self, vote: &str) -> u64
    {

        let votes_quantity =
            {
                self.state.data.iter()
                    .filter(|data| data.vote == vote)
                    .fold(0, |acc, x| acc + x.quantity )
            };
        if self.state.total_votes != 0
        {
            votes_quantity * 100 / self.state.total_votes
        }
        else
        {
            50
        }
    }
}


impl Component for Model
{
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self
    {
        let mut data = Vec::new();
        for vote_variant_key in VOTE_VARIANTS.keys()
        {
            let vote_stats = VoteStats { vote: vote_variant_key.to_string(), quantity: 0 };
            data.push(vote_stats);
        }
        Self { link, state: State { data, total_votes: 0, is_connected: false } , websocket_task: None }
    }


    fn update(&mut self, msg: Self::Message) -> ShouldRender
    {
        match msg
        {
            Msg::WsAction(action) =>
                {
                    match action
                    {
                        WsAction::Connect =>
                            {
                                let callback = self.link.callback(|Json(data)| Msg::WsReady(data));
                                let notification = self.link.callback(|status| match status
                                {
                                    WebSocketStatus::Opened => Msg::Ignore,
                                    WebSocketStatus::Closed | WebSocketStatus::Error => WsAction::Lost.into(),
                                });
                                let task =
                                    WebSocketService::connect(WEBSOCKET_URL, callback, notification)
                                        .unwrap();
                                self.websocket_task = Some(task);
                                self.state.is_connected = true;
                            },
                        WsAction::Disconnect =>
                            {
                                self.websocket_task.take();
                                self.state.is_connected = false;
                            },
                        WsAction::Lost => self.websocket_task = None,
                    }
                },
            Msg::Ignore => return false,
            Msg::WsReady(response) =>
                {
                    if let Some(received_data) = response.ok()
                    {
                        if received_data.action == WsResponseAction::ReceivedStatistics.as_str()
                        {

                            let data: Vec<VoteStats> = serde_json::from_str(&received_data.data).unwrap();
                            if data != self.state.data
                            {
                                self.state.data = data.clone();
                                let mut total_votes = 0;
                                for vote_stats in data
                                {
                                    total_votes += vote_stats.quantity;
                                }
                                self.state.total_votes = total_votes;
                            }
                            else { return false; }
                        }
                        else { return false; }
                    }
                    else { return false; }
                },
        }
        true
    }


    fn change(&mut self, _props: Self::Properties) -> ShouldRender
    {
        false
    }


    fn view(&self) -> Html
    {
        html!
        {
            <>
                <div id="background-stats">
                   <div id="background-stats-1"></div>
                   <div id="background-stats-2"></div>
                </div>
                <div id="content-container">
                    <div id="content-container-center">
                        <div id="choice">
                            <div class="choice cats">
                                <div class="label">{ VOTE_VARIANTS.get("a").unwrap() }</div>
                                <div class="stat">{ self.votes_percent("a") }{ "%" }</div>
                            </div>
                            <div class="divider"></div>
                            <div class="choice dogs">
                                <div class="label">{ VOTE_VARIANTS.get("b").unwrap() }</div>
                                <div class="stat">{ self.votes_percent("b") }{ "%" }</div>
                            </div>
                        </div>
                    </div>
                </div>
                <div id="result">
                    {
                        if self.state.total_votes == 0
                        {
                            html!
                            {
                                <span>{ "No votes yet" }</span>
                            }
                        }
                        else if self.state.total_votes == 1
                        {
                            html!
                            {
                                <span>{ "1 vote" }</span>
                            }
                        }
                        else
                        {
                            html!
                            {
                                <span>{ self.state.total_votes }{ " votes" }</span>
                            }
                        }
                    }
                </div>
            </>
        }
    }


    fn rendered(&mut self, first_render: bool)
    {
        if first_render
        {
            self.link.send_message(Msg::WsAction(WsAction::Connect));
        }
    }

}




#[wasm_bindgen(start)]
pub fn run_app()
{
    App::<Model>::new().mount_to_body();
}
