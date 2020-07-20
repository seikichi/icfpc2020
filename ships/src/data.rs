use core::AstNode;
use std::fmt;
use std::rc::Rc;

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

    pub fn opponent(&self) -> Role {
        match self {
            Role::Attacker => Role::Defender,
            Role::Defender => Role::Attacker,
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

    pub fn find_ship_info(&self, role: Role) -> ShipAndAppliedCommands {
        let mut min_ship_id = 0;
        let mut ret = None;
        for ship_and_commands in self.ships_and_commands.iter() {
            if ship_and_commands.ship.role == role {
                if ret.is_none() || ship_and_commands.ship.ship_id < min_ship_id {
                    min_ship_id = ship_and_commands.ship.ship_id;
                    ret = Some(ship_and_commands.clone());
                }
            }
        }
        return ret.unwrap();
        // panic!("the role not found: {:?}", role)
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
    pub x4: (i64, i64, i64, i64),
    pub x5: i64,
    pub x6: i64,
    pub x7: i64,
}

impl Ship {
    pub fn from_ast(ast: Rc<AstNode>) -> Self {
        let role_code = ast.get_list_item(0).get_number();
        let role = Role::from_int(role_code);

        let ship_id = ast.get_list_item(1).get_number();
        let position = Vector::from_ast(ast.get_list_item(2));
        let velocity = Vector::from_ast(ast.get_list_item(3));

        let x4_ast = ast.get_list_item(4);
        let x4_0 = x4_ast.get_list_item(0).get_number();
        let x4_1 = x4_ast.get_list_item(1).get_number();
        let x4_2 = x4_ast.get_list_item(2).get_number();
        let x4_3 = x4_ast.get_list_item(3).get_number();

        let x5 = ast.get_list_item(5).get_number();
        let x6 = ast.get_list_item(6).get_number();
        let x7 = ast.get_list_item(7).get_number();

        Self {
            role,
            ship_id,
            position,
            velocity,
            x4: (x4_0, x4_1, x4_2, x4_3),
            x5,
            x6,
            x7,
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
    pub fn dot(&self, rhs: &Vector) -> i64 {
        let mut ret = 0;
        ret += self.x * rhs.x;
        ret += self.y * rhs.y;
        return ret;
    }
    pub fn norm(&self) -> i64 {
        self.dot(self)
    }
    pub fn abs(&self) -> f64 {
        (self.norm() as f64).sqrt()
    }
    pub fn cross(&self, rhs: Vector) -> i64 {
        return self.x * rhs.y - self.y * rhs.x;
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

// Vector
impl std::ops::Neg for Vector {
    type Output = Vector;
    #[inline]
    fn neg(self) -> Vector {
        let mut ret = self;
        ret.x *= -1;
        ret.y *= -1;
        return ret;
    }
}

macro_rules! vector_vector_ops {
    ( $trate:ident, $fname:ident, $op:tt) => {
impl<'a> std::ops::$trate<Vector> for Vector {
    type Output = Vector;
    #[inline]
    fn $fname(self, rhs: Vector) -> Vector {
        let mut ret = self;
        ret.x = ret.x $op rhs.x;
        ret.y = ret.y $op rhs.y;
        return ret;
    }
        }
    };
}
macro_rules! self_self_assign_ops {
    ( $type:ty, $trate:ident, $fname:ident, $op:tt) => {
        impl<'a> std::ops::$trate<$type> for $type {
            #[inline]
            fn $fname(&mut self, rhs: $type) {
                *self = self.clone() $op rhs;
            }
        }
    };
}
vector_vector_ops!(Add, add, +);
vector_vector_ops!(Sub, sub, -);
self_self_assign_ops!(Vector, AddAssign, add_assign, +);
self_self_assign_ops!(Vector, SubAssign, sub_assign, -);
