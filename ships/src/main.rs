extern crate failure;
extern crate reqwest;
#[macro_use]
extern crate log;

use core::{demodulate, modulate, AstNode};
use env_logger::Builder;
use failure::Error;
use failure::Fail;
use log::LevelFilter;
use std::env;
use std::rc::Rc;
use std::thread;

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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Role {
    Attacker,
    Defender,
}

impl Role {
    pub fn from_int(i: i64) -> Role {
        match i {
            0 => Role::Attacker,
            1 => Role::Defender,
            _ => panic!("Unknown role: {}", i),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct StaticGameInfo {
    role: Role,
}

impl StaticGameInfo {
    pub fn from_ast(ast: Rc<AstNode>) -> Self {
        let role_code = ast.get_list_item(1).get_number();
        let role = Role::from_int(role_code);
        Self { role }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct GameState {
    game_tick: i64,
    // x1,
    ships_and_commands: Vec<ShipAndAppliedCommands>,
}

impl GameState {
    pub fn from_ast(ast: Rc<AstNode>) -> Option<Self> {
        if ast.is_nil() {
            return None;
        }
        let game_tick = ast.get_list_item(0).get_number();
        let _ship_and_commands_ast = ast.get_list_item(2);
        // TODO: ships_and_commands
        Some(Self {
            game_tick,
            ships_and_commands: vec![],
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ShipAndAppliedCommands {
    ship: Ship,
    // appliedCommands
}

impl ShipAndAppliedCommands {
    pub fn from_ast(ast: Rc<AstNode>) -> Self {
        let ship = Ship::from_ast(ast.get_list_item(0));
        Self { ship }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Ship {
    role: Role,
    ship_id: i64,
    position: Vector,
    velocity: Vector,
    // x4,
    // x5,
    // x6,
    // x7,
}

impl Ship {
    pub fn from_ast(ast: Rc<AstNode>) -> Self {
        let role_code = ast.get_list_item(0).get_number();
        let role = Role::from_int(role_code);

        let ship_id = ast.get_list_item(1).get_number();
        let position = Vector::from_ast(ast.get_list_item(2));
        let velocity = Vector::from_ast(ast.get_list_item(3));
        Self {
            role,
            ship_id,
            position,
            velocity,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Vector {
    x: i64,
    y: i64,
}

impl Vector {
    pub fn from_ast(ast: Rc<AstNode>) -> Self {
        let x = ast.children[0].get_number();
        let y = ast.children[1].get_number();
        Self { x, y }
    }
}

#[derive(Debug)]
pub struct GameResponse {
    stage: GameStage,
    static_game_info: StaticGameInfo,
    game_state: Option<GameState>,
}

impl GameResponse {
    pub fn from_ast(ast: Rc<AstNode>) -> Self {
        let stage_code = ast.get_list_item(1).get_number();
        let stage = GameStage::from_int(stage_code);

        let static_game_info_ast = ast.get_list_item(2);
        let static_game_info = StaticGameInfo::from_ast(static_game_info_ast);

        let game_state_ast = ast.get_list_item(3);
        let game_state = GameState::from_ast(game_state_ast);

        Self {
            stage,
            static_game_info,
            game_state,
        }
    }
}

fn send(
    api_key: Option<String>,
    server_url: &str,
    args: Rc<AstNode>,
    purpose: &str, /* for logging */
) -> Result<Rc<AstNode>, Error> {
    let param = api_key.map_or_else(|| "".to_owned(), |k| "?apiKey=".to_owned() + &k);
    let url = server_url.to_owned() + "/aliens/send" + &param;

    info!("Request({}): url={}, body={}", purpose, url, args);

    let encoded_args = modulate(args);
    let client = reqwest::blocking::Client::new();
    let resp = client.post(&url).body(encoded_args.to_owned()).send()?;

    let status = resp.status();
    if !status.is_success() {
        error!(
            "RequestFailed({}): status={}, body={}",
            purpose,
            status,
            resp.text()?
        );
        let e = RequestFailedError {};
        return Err(From::from(e));
    }

    let body = resp.text()?;
    let decoded_body = demodulate(&body);

    if decoded_body.get_list_item(0).get_number() == 0 {
        error!(
            "ErrorResponse({}): status={}, body={}",
            purpose, status, decoded_body
        );
        let e = RequestFailedError {};
        return Err(From::from(e));
    }
    info!(
        "Response({}): status={}, body={}",
        purpose, status, decoded_body
    );
    Ok(decoded_body)
}

#[derive(Debug, Clone)]
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
        purpose: &str, /* for logging */
    ) -> Result<Rc<AstNode>, Error> {
        send(self.api_key.clone(), &self.server_url, args, purpose)
    }

    pub fn join(&self) -> Result<GameResponse, Error> {
        let args = AstNode::make_list(&vec![
            AstNode::make_number(2),
            AstNode::make_number(self.player_key),
            AstNode::make_nil(),
        ]);
        let resp = self.send(args, "JOIN")?;
        info!("JOIN: resp={}", resp);
        Ok(GameResponse::from_ast(resp))
    }

    pub fn start(&self) -> Result<GameResponse, Error> {
        let args = AstNode::make_list(&vec![
            AstNode::make_number(3),
            AstNode::make_number(self.player_key),
            AstNode::make_list(&vec![
                AstNode::make_number(1),
                AstNode::make_number(1),
                AstNode::make_number(1),
                AstNode::make_number(1),
            ]),
        ]);
        let resp = self.send(args, "START")?;
        info!("START: resp={}", resp);
        Ok(GameResponse::from_ast(resp))
    }

    pub fn commands(&self) -> Result<GameResponse, Error> {
        let args = AstNode::make_list(&vec![
            AstNode::make_number(4),
            AstNode::make_number(self.player_key),
            AstNode::make_nil(),
        ]);
        let resp = self.send(args, "COMMANDS")?;
        info!("COMMANDS: resp={}", resp);
        Ok(GameResponse::from_ast(resp))
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Request failed")]
pub struct RequestFailedError {}

fn play(client: ProxyClient) -> Result<(), Error> {
    info!("Player: {}", client.player_key);

    let resp = client.join()?;
    if resp.stage == GameStage::Finished {
        return Ok(());
    }
    info!("Role: {:?}", resp.static_game_info.role);
    info!("GameResponse: {:?}", resp);

    let resp = client.start()?;
    if resp.stage == GameStage::Finished {
        return Ok(());
    }

    loop {
        let resp = client.commands()?;
        if resp.stage == GameStage::Finished {
            return Ok(());
        }
        if resp.stage == GameStage::NotStarted {
            panic!("Unexpected game stage NotStarted (after COMMANDS)");
        }
    }
}

fn create_players(api_key: Option<String>, server_url: &str) -> Result<(i64, i64), Error> {
    let args = AstNode::make_list(&vec![AstNode::make_number(1), AstNode::make_number(0)]);
    let resp = send(api_key, server_url, args, "CREATE")?;
    let pair = resp.get_list_item(1);
    let attacker_info = pair.get_list_item(0);
    let defender_info = pair.get_list_item(1);
    let ids = (
        attacker_info.get_list_item(1).get_number(),
        defender_info.get_list_item(1).get_number(),
    );
    Ok(ids)
}

#[derive(Debug, Eq, PartialEq)]
enum Mode {
    Local,
    Remote,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Builder::from_default_env()
        .filter(None, LevelFilter::Info)
        .init();

    let args: Vec<String> = env::args().collect();

    let default_server_url = "https://icfpc2020-api.testkontur.ru".to_owned();
    let api_key = "c793f2239e4f4b4bbb842c399878dec4".to_owned();

    let server_url = if args.len() >= 2 {
        args[1].clone()
    } else {
        default_server_url
    };
    let mode = if args.len() == 3 {
        Mode::Remote
    } else {
        Mode::Local
    };

    info!("Mode: {:?}, ServerUrl: {}", mode, server_url);

    match mode {
        Mode::Local => {
            let (attacker_id, defender_id) = create_players(Some(api_key.clone()), &server_url)?;

            let api_key_copy = api_key.clone();
            let server_url_copy = server_url.clone();
            let attacker = thread::spawn(move || {
                let client = ProxyClient::new(&server_url_copy, attacker_id, Some(api_key_copy));
                play(client).unwrap()
            });

            let api_key_copy2 = api_key.clone();
            let server_url_copy2 = server_url.clone();
            let defender = thread::spawn(move || {
                let client = ProxyClient::new(&server_url_copy2, defender_id, Some(api_key_copy2));
                play(client).unwrap()
            });

            attacker.join().unwrap();
            defender.join().unwrap();
            Ok(())
        }
        Mode::Remote => {
            let player_key = args[2].parse::<i64>()?;
            let client = ProxyClient::new(&server_url, player_key, None);
            play(client)?;
            Ok(())
        }
    }
}
