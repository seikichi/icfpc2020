use crate::data::*;
use core::AstNode;
use std::rc::Rc;

pub enum Command {
    Accelerate {
        ship_id: i64,
        vector: Vector,
    },
    Detonate {
        ship_id: i64,
    },
    Shoot {
        ship_id: i64,
        target: Vector,
        x3: i64,
    },
    Spawn {
        ship_id: i64,
        parameter: (i64, i64, i64, i64),
    },
}

impl Command {
    pub fn to_ast(&self) -> Rc<AstNode> {
        match self {
            Command::Accelerate { ship_id, vector } => AstNode::make_list(&vec![
                AstNode::make_number(0),
                AstNode::make_number(*ship_id),
                AstNode::make_vector(vector.x, vector.y),
            ]),
            Command::Detonate { ship_id } => AstNode::make_list(&vec![
                AstNode::make_number(1),
                AstNode::make_number(*ship_id),
            ]),
            Command::Shoot {
                ship_id,
                target,
                x3,
            } => AstNode::make_list(&vec![
                AstNode::make_number(2),
                AstNode::make_number(*ship_id),
                AstNode::make_vector(target.x, target.y),
                AstNode::make_number(*x3),
            ]),
            Command::Spawn { ship_id, parameter } => AstNode::make_list(&vec![
                AstNode::make_number(3),
                AstNode::make_number(*ship_id),
                AstNode::make_list(&vec![
                    AstNode::make_number(parameter.0),
                    AstNode::make_number(parameter.1),
                    AstNode::make_number(parameter.2),
                    AstNode::make_number(parameter.3),
                ]),
            ]),
        }
    }
}
