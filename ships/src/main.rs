mod command;
mod data;

extern crate failure;
extern crate reqwest;
#[macro_use]
extern crate log;

use env_logger::Builder;
use failure::Error;
use failure::Fail;
use log::LevelFilter;
use rand::Rng;
use std::env;
use std::rc::Rc;
use std::thread;
use std::f64::consts::PI;

use crate::command::*;
use crate::data::*;
use core::{demodulate, modulate, AstNode};

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

    pub fn start(&self, role: Role) -> Result<GameResponse, Error> {
        let args = AstNode::make_list(&vec![
            AstNode::make_number(3),
            AstNode::make_number(self.player_key),
            match role {
                // max 392, 0, 4, 4
                // max 388, 1, 4, 4
                // max 232, 40, 4, 4
                Role::Attacker => AstNode::make_list(&vec![
                    AstNode::make_number(86),
                    AstNode::make_number(58),
                    AstNode::make_number(16),
                    AstNode::make_number(1),
                ]),
                Role::Defender => AstNode::make_list(&vec![
                    AstNode::make_number(232),
                    AstNode::make_number(0),
                    AstNode::make_number(8),
                    AstNode::make_number(60),
                ]),
            },
        ]);
        let resp = self.send(args, "START")?;
        info!("START: resp={}", resp);
        Ok(GameResponse::from_ast(resp))
    }

    pub fn commands(&self, commands: &[Command]) -> Result<GameResponse, Error> {
        let command_asts: Vec<_> = commands.iter().map(|c| c.to_ast()).collect();
        let commands_ast = AstNode::make_list(&command_asts);

        let args = AstNode::make_list(&vec![
            AstNode::make_number(4),
            AstNode::make_number(self.player_key),
            commands_ast,
        ]);
        let resp = self.send(args, "COMMANDS")?;
        info!("COMMANDS: resp={}", resp);
        Ok(GameResponse::from_ast(resp))
    }
}

#[derive(Fail, Debug)]
#[fail(display = "Request failed")]
pub struct RequestFailedError {}

fn normalize_dir(v: Vector) -> Vector {
    let mut best = Vector::new(0, 0);
    let mut best_cosine = 0.0;
    for y in -1..=1 {
        for x in -1..=1 {
            let cosine = cosine_sim(v, Vector::new(x, y));
            if cosine > best_cosine {
                best = Vector::new(x, y);
                best_cosine = cosine;
            }
        }
    }
    best
}

fn sgn(x: i64) -> i64 {
    if x > 0 {
        1
    } else if x < 0 {
        -1
    } else {
        0
    }
}

fn simulate_next(mut pos: Vector, mut vel: Vector) -> (Vector, Vector) {
    if pos.x.abs() == pos.y.abs() {
        vel.x += -sgn(pos.x);
        vel.y += -sgn(pos.y);
    } else if pos.x.abs() > pos.y.abs() {
        vel.x += -sgn(pos.x);
    } else {
        vel.y += -sgn(pos.y);
    }
    pos += vel;
    (pos, vel)
}

// 星にぶつかるまでの時間をかえす
fn simulate_orbit_to_planet(
    mut pos: Vector,
    mut vel: Vector,
    n: isize,
    planet_radius: i64,
) -> isize {
    if pos.x.abs() <= planet_radius && pos.y.abs() <= planet_radius {
        return 0;
    }
    for i in 0..n {
        let (next_pos, next_vel) = simulate_next(pos, vel);
        pos = next_pos;
        vel = next_vel;
        if pos.x.abs() <= planet_radius && pos.y.abs() <= planet_radius {
            return i + 1;
        }
    }
    return n + 1;
}

// 安全なエリアから出るまでの時間をかえす
fn simulate_orbit_out_of_safe_area(
    mut pos: Vector,
    mut vel: Vector,
    n: isize,
    safe_radius: i64,
) -> isize {
    if pos.x.abs() > safe_radius && pos.y.abs() > safe_radius {
        return 0;
    }
    for i in 0..n {
        let (next_pos, next_vel) = simulate_next(pos, vel);
        pos = next_pos;
        vel = next_vel;
        if pos.x.abs() > safe_radius || pos.y.abs() > safe_radius {
            return i + 1;
        }
    }
    return n + 1;
}

fn simulate_in_orbit(
    pos: Vector,
    vel: Vector,
    n: isize,
    planet_radius: i64,
    safe_radius: i64,
) -> bool {
    if simulate_orbit_to_planet(pos, vel, n, planet_radius) <= n {
        return false;
    }
    if simulate_orbit_out_of_safe_area(pos, vel, n, safe_radius) <= n {
        return false;
    }
    return true;
}

fn cosine_sim(v1: Vector, v2: Vector) -> f64 {
    return (v1.dot(&v2) as f64) / (v1.abs() * v2.abs());
}

fn is_good_attack_angle(relative_pos: Vector) -> bool {
    relative_pos.x.abs() <= 1 ||
    relative_pos.y.abs() <= 1 ||
    (relative_pos.x.abs() - relative_pos.y.abs()).abs() <= 1
}

fn should_shoot_regardless_of_angle(opponent: &Ship) -> bool {
    opponent.x4.2 == 0 || (opponent.x6 - opponent.x5) <= 10
}

const PLANET_RADIUS: i64 = 16;
const SAFE_AREA: i64 = 128;
fn play(client: ProxyClient) -> Result<(), Error> {
    let mut rng = rand::thread_rng();

    info!("Player: {}", client.player_key);

    let resp = client.join()?;
    if resp.stage == GameStage::Finished {
        return Ok(());
    }
    let role = resp.static_game_info.role;
    info!("Role: {:?}", role);
    info!("GameResponse: {:?}", resp);

    let resp = client.start(role)?;
    if resp.stage == GameStage::Finished {
        return Ok(());
    }

    let game_state = resp.game_state.unwrap();
    let ship_id = game_state.find_ship_info(role).ship.ship_id;

    let mut tick = 0;
    let mut prev_pos = Vector::new(0, 0);
    let mut prev_vel = Vector::new(0, 0);
    let mut prev_x4 = (0, 0, 0, 0);
    let mut prev_x5 = 0; // ヒート的な値
    let mut prev_x6 = 0; // オーバーヒートの上限
    let mut prev_x7 = 0;
    let mut next_should_move = false;
    let mut prev_opponent_pos = Vector::new(0, 0);
    let mut prev_opponent_vel = Vector::new(0, 0);
    let mut prev_opponent = game_state.find_ship_info(role.opponent()).ship;

    loop {
        let collide_steps = simulate_orbit_to_planet(prev_pos, prev_vel, 8, PLANET_RADIUS + 10);
        let out_of_bound_steps = simulate_orbit_out_of_safe_area(prev_pos, prev_vel, 5, SAFE_AREA);

        let orbit_v = {
            let v1 = normalize_dir(Vector::new(-prev_pos.y, prev_pos.x));
            let v2 = normalize_dir(Vector::new(prev_pos.y, -prev_pos.x));
            let n = prev_pos.cross(prev_vel);
            if sgn(prev_pos.cross(-v1)) == sgn(n) {
                v1
            } else {
                v2
            }
        };

        let mut commands = if collide_steps <= 8 {
            info!("@@@@ [{:?}] v={}, planet_collide", role, orbit_v);
            let acc = Command::Accelerate {
                ship_id: ship_id,
                vector: orbit_v,
            };
            vec![acc]
        } else if out_of_bound_steps <= 5 {
            let v = normalize_dir(Vector::new(prev_vel.x, prev_vel.y));
            info!("@@@@ [{:?}] v={}, out_of_bound", role, v);
            let acc = Command::Accelerate {
                ship_id: ship_id,
                vector: v,
            };
            vec![acc]
        } else {
            vec![]
        };

        if commands.is_empty() && next_should_move {
            next_should_move = false;
            info!("@@@@ [{:?}] v={}, after spawn", role, orbit_v);
            let acc = Command::Accelerate {
                ship_id: ship_id,
                vector: orbit_v,
            };
            commands.push(acc);
        }

        let (next_opponent_pos, _) = simulate_next(prev_opponent_pos, prev_opponent_vel);
        let (next_pos, _) = simulate_next(prev_pos, prev_vel);
        if (next_opponent_pos - next_pos).abs() < 20.0 {
            // 敵と接近するとき
            // 動く (50%)
            if commands.is_empty() && rng.gen_range(0, 2) == 0 {
                info!("@@@@ [{:?}] v={}, in_danger", role, orbit_v);
                let acc = Command::Accelerate {
                    ship_id: ship_id,
                    vector: orbit_v,
                };
                commands.push(acc);
            }
        }
        if role == Role::Attacker {
            let relative_pos = next_opponent_pos - next_pos;
            // 殴る
            let room_for_attack = prev_x5 + prev_x4.1 <= prev_x6;
            if room_for_attack && (is_good_attack_angle(relative_pos) || should_shoot_regardless_of_angle(&prev_opponent)) {
                info!("@@@@ [{:?}] shoot", role);
                // 温度が大丈夫そうなら
                let beam = Command::Shoot {
                    ship_id: ship_id,
                    target: next_opponent_pos,
                    x3: prev_x4.1,
                };
                commands.push(beam);
            }
        }
        if role == Role::Defender && commands.len() == 0 && prev_x4.3 > 1 {
            if simulate_in_orbit(prev_pos, prev_vel, 256 - tick, PLANET_RADIUS, SAFE_AREA) {
                info!("@@@@ [{:?}] spawn", role);
                let spawn = Command::Spawn {
                    ship_id: ship_id,
                    parameter: (0, 0, 0, 1),
                };
                commands.push(spawn);
                next_should_move = true;
            } else {
                let dx = [1, 1, 1, 0, -1, -1, -1, 0];
                let dy = [-1, 0, 1, 1, 1, 0, -1, -1];
                for i in 0..8 {
                    let tvel = Vector::new(prev_vel.x + dx[i], prev_vel.y + dy[i]);
                    if simulate_in_orbit(prev_pos, tvel, 256 - tick, PLANET_RADIUS, SAFE_AREA) {
                        let dir = Vector::new(-dx[i], -dy[i]);
                        info!("@@@@ [{:?}] ac for spawn {:?}", role, dir);
                        let ac = Command::Accelerate {
                            ship_id: ship_id,
                            vector: Vector::new(-dx[i], -dy[i]),
                        };
                        commands.push(ac);
                        break;
                    }
                }
            }
        }

        let resp = client.commands(&commands)?;
        // info!("[{:?}] GameResponse: {:?}", role, resp);
        let game_state = resp.game_state.unwrap();

        let ship = game_state.find_ship_info(role).ship;
        info!(
            "@@@@ [{:?}] pos={}, vel={}",
            role, ship.position, ship.velocity
        );
        if resp.stage == GameStage::Finished {
            return Ok(());
        }
        if resp.stage == GameStage::NotStarted {
            panic!(
                "[{:?}] Unexpected game stage NotStarted (after COMMANDS)",
                role
            );
        }

        prev_vel = ship.velocity;
        prev_pos = ship.position;
        prev_x4 = ship.x4;
        prev_x5 = ship.x5;
        prev_x6 = ship.x6;
        prev_x7 = ship.x7;
        info!("[{:?}] {:?} {} {}", role, prev_x4, prev_x5, prev_x6);

        let opponent = game_state.find_ship_info(role.opponent()).ship;
        prev_opponent_pos = opponent.position;
        prev_opponent_vel = opponent.velocity;
        prev_opponent = opponent;
        tick += 1;
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
    Builder::from_default_env().init();

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
