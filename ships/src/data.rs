use std::rc::Rc;
use std::fmt;
use core::AstNode;

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
    pub maybe_field_size: i64,
    pub role: Role,
}

impl StaticGameInfo {
    pub fn from_ast(ast: Rc<AstNode>) -> Self {
        let maybe_field_size = ast.get_list_item(0).get_number();
        let role_code = ast.get_list_item(1).get_number();
        let role = Role::from_int(role_code);
        Self {
            maybe_field_size,
            role,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct GameState {
    pub game_tick: i64,
    // x1,
    pub ships_and_commands: Vec<ShipAndAppliedCommands>,
}

impl GameState {
    pub fn from_ast(ast: Rc<AstNode>) -> Option<Self> {
        if ast.is_nil() {
            return None;
        }
        let game_tick = ast.get_list_item(0).get_number();
        let ships_and_commands_ast = ast.get_list_item(2);
        let mut ships_and_commands = vec![];
        ships_and_commands_ast.for_each(|ast| {
            ships_and_commands.push(ShipAndAppliedCommands::from_ast(ast));
        });
        Some(Self {
            game_tick,
            ships_and_commands,
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ShipAndAppliedCommands {
    pub ship: Ship,
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
    pub role: Role,
    pub ship_id: i64,
    pub position: Vector,
    pub velocity: Vector,
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
    pub x: i64,
    pub y: i64,
}

impl Vector {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub fn from_ast(ast: Rc<AstNode>) -> Self {
        let x = ast.children[0].get_number();
        let y = ast.children[1].get_number();
        Self { x, y }
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug)]
pub struct GameResponse {
    pub stage: GameStage,
    pub static_game_info: StaticGameInfo,
    pub game_state: Option<GameState>,
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
