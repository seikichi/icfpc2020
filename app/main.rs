// extern crate failure;
extern crate reqwest;

use http_body::Body as _;
use hyper::{Body, Client, Method, Request, StatusCode};
use std::env;
use std::process;
// use failure::Error;

/*
pub struct ProxyClient {
    server_url: String,
    player_key: String,
}

impl ProxyClient {
    pub fn new(server_url: &str, player_key: &str) -> Self {
        Self {
            server_url: server_url.to_owned(),
            player_key: player_key.to_owned(),
        }
    }

    fn send(self) -> Result<(), Error> {
        let url = self.server_url + "/alians/send";
        let req = Request::builder()
            .method(Method::POST)
            .uri(url)
            .body(Body::from(format!("{}", self.player_key)))?;

        let client = Client::new();
        let res = client.request(req).await?;

        match res.status() {
            StatusCode::Ok => {
                
                while let Some(chunk) = res.body_mut().data().await {
                    match chunk {
                        Ok(content) => println!("{:?}", content),
                        Err(why) => println!("error reading body: {:?}", why),
                    }
                }
            }
        }

        Ok(())
    }
}
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = &args[2];

    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let client = Client::new();
    let req = Request::builder()
        .method(Method::POST)
        .uri(server_url)
        .body(Body::from(format!("{}", player_key)))?;

    match client.request(req).await {
        Ok(mut res) => match res.status() {
            StatusCode::OK => {
                print!("Server response: ");
                while let Some(chunk) = res.body_mut().data().await {
                    match chunk {
                        Ok(content) => println!("{:?}", content),
                        Err(why) => println!("error reading body: {:?}", why),
                    }
                }
            }
            _ => {
                println!("Unexpected server response:");
                println!("HTTP code: {}", res.status());
                print!("Response body: ");
                while let Some(chunk) = res.body_mut().data().await {
                    match chunk {
                        Ok(content) => println!("{:?}", content),
                        Err(why) => println!("error reading body: {:?}", why),
                    }
                }
                process::exit(2);
            }
        },
        Err(err) => {
            println!("Unexpected server response:\n{}", err);
            process::exit(1);
        }
    }

    Ok(())
}
