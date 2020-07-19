extern crate failure;
extern crate reqwest;

use failure::Error;
use failure::Fail;
use std::env;
use core::{AstNode, modulate};

pub struct ProxyClient {
    server_url: String,
    player_key: i64,
}

impl ProxyClient {
    pub fn new(server_url: &str, player_key: i64) -> Self {
        Self {
            server_url: server_url.to_owned(),
            player_key: player_key,
        }
    }

    fn send(self, encoded_args: &str, purpose: &str /* for logging */) -> Result<String, Error> {
        let url = self.server_url + "/alians/send";

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

    pub fn join(self) -> Result<(), Error> {
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
}

#[derive(Fail, Debug)]
#[fail(display = "Request failed")]
pub struct RequestFailedError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = args[2].parse::<i64>()?;

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let client = ProxyClient::new(server_url, player_key);

    client.join()?;

    Ok(())
}
