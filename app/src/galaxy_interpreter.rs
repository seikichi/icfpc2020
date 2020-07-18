use std::collections::HashMap;
use std::convert::From;
use std::fs;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Function {
    Ap,
    Cons,
    Car,
    Cdr,
    Isnil,
    Nil,
    Neg,
    Add,
    Mul,
    Div,
    Lt,
    Equal,
    Bcombinator,
    Ccombinator,
    Scombinator,
    Icombinator,
    True,
    False,
    Number(i64),
    Variable(i64),
}

impl From<&str> for Function {
    fn from(item: &str) -> Self {
        match item {
            "ap" => Function::Ap,
            "cons" => Function::Cons,
            "car" => Function::Car,
            "cdr" => Function::Cdr,
            "isnil" => Function::Isnil,
            "nil" => Function::Nil,
            "neg" => Function::Neg,
            "add" => Function::Add,
            "mul" => Function::Mul,
            "div" => Function::Div,
            "lt" => Function::Lt,
            "eq" => Function::Equal,
            "b" => Function::Bcombinator,
            "c" => Function::Ccombinator,
            "s" => Function::Scombinator,
            "i" => Function::Icombinator,
            "t" => Function::True,
            "f" => Function::False,
            v if v.chars().nth(0).unwrap().is_digit(10) || &v[..1] == "-" => Function::Number(
                v.parse::<i64>()
                    .expect(format!("{} is not number", v).as_str()),
            ),
            v if &v[..1] == ":" && v.chars().nth(1).unwrap().is_digit(10) => Function::Variable(
                v[1..]
                    .parse::<i64>()
                    .expect(format!("{} is not variable", v).as_str()),
            ),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Statement {
    pub id: i64,
    pub cells: Vec<Function>,
}

impl Statement {
    pub fn new(s: &str) -> Self {
        let items: Vec<&str> = s.split(" ").collect();
        let id = if items[0] == "galaxy" {
            0
        } else {
            items[0][1..]
                .parse::<i64>()
                .expect(format!("{} is not id", items[0]).as_str())
        };
        let mut cells = vec![];
        for &item in items[2..].iter() {
            cells.push(Function::from(item));
        }
        Statement { id, cells }
    }
}

pub fn load() -> HashMap<i64, Statement> {
    fs::read_to_string("resource/galaxy.txt")
        .unwrap()
        .split("\n")
        .map(|s| {
            let statement = Statement::new(s);
            (statement.id, statement)
        })
        .collect()
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct AstNode {
    value: Function,
    left: Option<Rc<AstNode>>,
    right: Option<Rc<AstNode>>,
}

impl AstNode {
    fn parse_cells(cells: &Vec<Function>, cell_index: usize) -> (Rc<Self>, usize) {
        let value = cells[cell_index];
        match value {
            Function::Ap => {
                let (left, cell_index) = AstNode::parse_cells(cells, cell_index + 1);
                let (right, cell_index) = AstNode::parse_cells(cells, cell_index + 1);
                let ret = AstNode {
                    value,
                    left: Some(left),
                    right: Some(right),
                };
                (Rc::new(ret), cell_index)
            }
            _ => {
                let ret = AstNode {
                    value,
                    left: None,
                    right: None,
                };
                (Rc::new(ret), cell_index)
            }
        }
    }
}

fn main() {
    let statements = load();
    let mut ast_nodes = HashMap::<i64, Rc<AstNode>>::new();
    for statement in statements.values() {
        let (node, index) = AstNode::parse_cells(&statement.cells, 0);
        assert!(index == statement.cells.len() - 1);
        ast_nodes.insert(statement.id, node);
    }
}
