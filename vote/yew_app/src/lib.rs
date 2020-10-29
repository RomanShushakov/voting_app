#![recursion_limit="512"]

use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response, FetchOptions, Credentials};
use lazy_static;
use std::collections::HashMap;
use yew::format::Json;
use anyhow::Error;
use serde::Serialize;
use web_sys;
use wasm_bindgen::JsCast;
use uuid::Uuid;


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


enum Vote
{
    A,
    B,
}


impl Vote
{
    pub fn as_str(&self) -> &str
    {
        match self
        {
            Vote::A => "a",
            Vote::B => "b",
        }
    }
}


struct State
{
    id: String,
    vote: Option<Vote>,
}


struct Model
{
    link: ComponentLink<Self>,
    state: State,
    fetch_task: Option<FetchTask>,
}


enum Msg
{
    Vote(&'static str),
    VoteSuccessful(Result<String, Error>),
    VoteNotSuccessful
}


#[derive(Serialize)]
struct VoteRequest
{
    voter_id: String,
    vote: String,
}


impl Model
{
    fn make_vote(&self, vote: &str) -> FetchTask
    {
        let vote_request = VoteRequest { voter_id: self.state.id.to_owned(), vote: vote.to_owned() };
        let callback = self.link.callback(
            move |response: Response<Result<String, Error>>|
                {
                    let (meta, message) = response.into_parts();
                    if meta.status.is_success()
                    {
                        Msg::VoteSuccessful(message)
                    }
                    else
                    {
                        Msg::VoteNotSuccessful
                    }
                },
            );
        let request = Request::post("/")
            .header("Content-Type", "application/json")
            .body(Json(&vote_request))
            .unwrap();
        let options = FetchOptions
            {
                credentials: Some(Credentials::SameOrigin),
                ..FetchOptions::default()
            };
        FetchService::fetch_with_options(request, options, callback).unwrap()
    }
}


fn create_id() -> Result<String, JsValue>
{
    let new_uuid = Uuid::new_v4().to_string();
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let html_document = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
    let cookies = html_document.cookie()?;
    if !cookies.is_empty()
    {
        for cookie in cookies.split("; ").into_iter().collect::<Vec<&str>>()
        {
            let splitted_cookie = cookie.split("=").collect::<Vec<&str>>();
            if splitted_cookie.len() == 2 && splitted_cookie[0] == "voter_id"
            {
                return Ok(splitted_cookie[1].to_owned());
            }
        }
        let new_cookie = "voter_id=".to_owned() + &new_uuid;
        html_document.set_cookie(&new_cookie)?;
        Ok(new_uuid)
    }
    else
    {
        let new_cookie = "voter_id=".to_owned() + &new_uuid;
        html_document.set_cookie(&new_cookie)?;
        Ok(new_uuid)
    }
}


impl Component for Model
{
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self
    {
        let id =
            {
                if let Ok(id) = create_id()
                {
                    id
                }
                else
                {
                    Uuid::new_v4().to_string()
                }
            };
        Self { link, state: State { id, vote: None } , fetch_task: None }
    }


    fn update(&mut self, msg: Self::Message) -> ShouldRender
    {
        match msg
        {
            Msg::Vote(vote) =>
                {
                    if vote == Vote::A.as_str()
                    {
                        self.state.vote = Some(Vote::A);
                    }
                    if vote == Vote::B.as_str()
                    {
                        self.state.vote = Some(Vote::B);
                    }
                    let task = self.make_vote(vote);
                    self.fetch_task = Some(task);
                },
            Msg::VoteSuccessful(_message) => (),
            Msg::VoteNotSuccessful => return false,
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
            <div id="content-container">
                <div id="content-container-center">
                    <h3>{ VOTE_VARIANTS.get("a").unwrap() }{ " vs " }{ VOTE_VARIANTS.get("b").unwrap() }</h3>
                    {
                        if let Some(vote) = &self.state.vote
                        {
                            match vote
                            {
                                Vote::A =>
                                    {
                                        html!
                                        {
                                            <div id="choice">
                                                <button id="a" class="a" disabled=true>
                                                    { VOTE_VARIANTS.get("a").unwrap() }
                                                    <i class="fa fa-check-circle"></i>
                                                </button>
                                                <button id="b" class="b" style="opacity: 0.5;"
                                                    onclick=self.link.callback(|_| Msg::Vote("b"))>
                                                    { VOTE_VARIANTS.get("b").unwrap() }
                                                </button>
                                            </div>
                                        }
                                    },
                                Vote::B =>
                                    {
                                        html!
                                        {
                                            <div id="choice">
                                                <button id="a" class="a" style="opacity: 0.5;"
                                                onclick=self.link.callback(|_| Msg::Vote("a"))>
                                                    { VOTE_VARIANTS.get("a").unwrap() }
                                                </button>
                                                <button id="b" class="b" disabled=true>
                                                    { VOTE_VARIANTS.get("b").unwrap() }
                                                    <i class="fa fa-check-circle"></i>
                                                </button>
                                            </div>
                                        }
                                    },
                            }
                        }
                        else
                        {
                            html!
                            {
                                <div id="choice">
                                    <button id="a" class="a" onclick=self.link.callback(|_| Msg::Vote("a"))>
                                        { VOTE_VARIANTS.get("a").unwrap() }
                                    </button>
                                    <button id="b" class="b" onclick=self.link.callback(|_| Msg::Vote("b"))>
                                        { VOTE_VARIANTS.get("b").unwrap() }
                                    </button>
                                </div>
                            }
                        }
                    }
                    <div id="tip">
                        { "(Tip: you can change your vote)" }
                    </div>
                    <div id="hostname">
                        { "Processed by container ID " }{ &self.state.id }
                    </div>
                </div>
            </div>
        }
    }
}


#[wasm_bindgen(start)]
pub fn run_app()
{
    App::<Model>::new().mount_to_body();
}
