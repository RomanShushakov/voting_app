use serde::Serialize;


#[derive(Debug, Serialize, Clone)]
pub struct VoteStats
{
    pub vote: String,
    pub quantity: u64,
}


#[derive(Serialize, Debug)]
pub struct WsResponse
{
    pub action: String,
    pub data: String,
}