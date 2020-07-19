extern crate failure;
extern crate reqwest;

use failure::Error;
use failure::Fail;
use std::env;
use core::{AstNode, modulate};

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

    fn send(&self, encoded_args: &str, purpose: &str /* for logging */) -> Result<String, Error> {
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
        Ok(body)
    }

    pub fn join(&self) -> Result<(), Error> {
        let args = AstNode::make_list(&vec![
            AstNode::make_number(2),
            AstNode::make_number(self.player_key),
            AstNode::make_nil(),
        ]);
        let encoded_args = modulate(args);
        let resp = self.send(&encoded_args, "JOIN")?;
        println!("JOIN: resp={}", resp);
        Ok(())
    }

    pub fn start(&self) -> Result<(), Error> {
        let args = AstNode::make_list(&vec![
            AstNode::make_number(3),
            AstNode::make_number(self.player_key),
            AstNode::make_list(&vec![
                AstNode::make_number(0),
                AstNode::make_number(0),
                AstNode::make_number(0),
                AstNode::make_number(0),
            ]),
        ]);
        let encoded_args = modulate(args);
        let resp = self.send(&encoded_args, "START")?;
        println!("START: resp={}", resp);
        Ok(())
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Request failed")]
pub struct RequestFailedError {}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let default_server_url = "https://icfpc2020-api.testkontur.ru";
    let default_player_key: i64 = 123456789;
    let api_key = Some("c793f2239e4f4b4bbb842c399878dec4".to_owned());

    let server_url = if args.len() >= 2 { &args[1] } else { default_server_url };
    let player_key = if args.len() >= 3 { args[2].parse::<i64>()? } else { default_player_key };

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let client = ProxyClient::new(server_url, player_key, api_key);

    client.join()?;
    client.start()?;

    Ok(())
}
