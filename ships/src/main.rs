extern crate failure;
extern crate reqwest;

use failure::Error;
use failure::Fail;
use std::rc::Rc;
use std::env;
use core::{AstNode, modulate, demodulate};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum GameStage {
    NotStarted,
    Started,
    Finished,
}

impl GameStage {
    pub fn from_int(i: i64) -> GameStage {
        match i {
            0 => GameStage::NotStarted,
            1 => GameStage::Started,
            2 => GameStage::Finished,
            _ => panic!("Unknown stage: {}", i),
        }
    }
}

#[derive(Debug)]
pub struct GameResponse {
    stage: GameStage,
}

impl GameResponse {
    pub fn from_ast(ast: Rc<AstNode>) -> Self {
        let stage_code = ast.get_list_item(1).get_number();
        let stage = GameStage::from_int(stage_code);
        Self {
            stage: stage,
        }
    }
}

#[derive(Debug)]
pub struct ProxyClient {
    server_url: String,
    player_key: i64,
    api_key: Option<String>,
}

impl ProxyClient {
    pub fn new(server_url: &str, player_key: i64, api_key: Option<String>) -> Self {
        Self {
            server_url: server_url.to_owned(),
            player_key: player_key,
            api_key: api_key,
        }
    }

    fn send(
        &self,
        args: Rc<AstNode>,
        purpose: &str /* for logging */
    ) -> Result<Rc<AstNode>, Error> {
        let encoded_args = modulate(args);

        let param = self.api_key.as_ref()
            .map_or_else(|| "".to_owned(), |k| "?apiKey=".to_owned() + &k);
        let url = self.server_url.clone() + "/aliens/send" + &param;

        println!("Request({}): url={}, body={}", purpose, url, encoded_args);

        let client = reqwest::blocking::Client::new();
        let resp = client.post(&url).body(encoded_args.to_owned()).send()?;

        if !resp.status().is_success() {
            println!("RequestFailed: status={}, body={}", resp.status(), resp.text()?);
            let e = RequestFailedError {}; // TODO: レスポンスの情報を埋める
            return Err(From::from(e));
        }

        let body = resp.text()?;
        let decoded_body = demodulate(&body);
        Ok(decoded_body)
    }

    pub fn join(&self) -> Result<GameResponse, Error> {
        let args = AstNode::make_list(&vec![
            AstNode::make_number(2),
            AstNode::make_number(self.player_key),
            AstNode::make_nil(),
        ]);
        let resp = self.send(args, "JOIN")?;
        println!("JOIN: resp={}", resp);
        Ok(GameResponse::from_ast(resp))
    }

    pub fn start(&self) -> Result<GameResponse, Error> {
        let args = AstNode::make_list(&vec![
            AstNode::make_number(3),
            AstNode::make_number(self.player_key),
            AstNode::make_list(&vec![
                AstNode::make_number(510),
                AstNode::make_number(1),
                AstNode::make_number(1),
                AstNode::make_number(1),
            ]),
        ]);
        let resp = self.send(args, "START")?;
        println!("START: resp={}", resp);
        Ok(GameResponse::from_ast(resp))
    }

    pub fn commands(&self) -> Result<GameResponse, Error> {
        let args = AstNode::make_list(&vec![
            AstNode::make_number(4),
            AstNode::make_number(self.player_key),
            AstNode::make_nil(),
        ]);
        let resp = self.send(args, "COMMANDS")?;
        println!("COMMANDS: resp={}", resp);
        Ok(GameResponse::from_ast(resp))
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Request failed")]
pub struct RequestFailedError {}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let default_server_url = "https://icfpc2020-api.testkontur.ru";
    let default_player_key: i64 = 123456789;
    let api_key = None;

    let server_url = if args.len() >= 2 { &args[1] } else { default_server_url };
    let player_key = if args.len() >= 3 { args[2].parse::<i64>()? } else { default_player_key };

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let client = ProxyClient::new(server_url, player_key, api_key);

    client.join()?;

    let resp = client.start()?;
    if resp.stage == GameStage::Finished {
        return Ok(())
    }

    loop {
        let resp = client.commands()?;
        if resp.stage == GameStage::Finished {
            return Ok(())
        }
        if resp.stage == GameStage::NotStarted {
            panic!("Unexpected game stage NotStarted (after COMMANDS)");
        }
    }
}
