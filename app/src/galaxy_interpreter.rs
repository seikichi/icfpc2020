use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use std::fs;
use std::rc::Rc;
use std::thread;

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
    children: Vec<Rc<AstNode>>,
}

impl AstNode {
    #[allow(dead_code)]
    fn parse_str(s: &str) -> Rc<Self> {
        let statement = Statement::new(s);
        let (node, index) = AstNode::parse_cells(&statement.cells, 0);
        assert!(index == statement.cells.len() - 1);
        return node;
    }
    fn parse_cells(cells: &Vec<Function>, cell_index: usize) -> (Rc<Self>, usize) {
        let value = cells[cell_index];
        match value {
            Function::Ap => {
                let (left, cell_index) = AstNode::parse_cells(cells, cell_index + 1);
                let (right, cell_index) = AstNode::parse_cells(cells, cell_index + 1);
                let ret = AstNode {
                    value,
                    children: vec![left, right],
                };
                (Rc::new(ret), cell_index)
            }
            _ => {
                let ret = AstNode {
                    value,
                    children: vec![],
                };
                (Rc::new(ret), cell_index)
            }
        }
    }
    fn make_leaf(function: Function) -> Rc<Self> {
        Rc::new(AstNode {
            value: function,
            children: vec![],
        })
    }
}

type Result<T> = std::result::Result<T, EvaluateError>;
#[derive(Debug, Clone)]
struct EvaluateError;
impl fmt::Display for EvaluateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Evaluate Error")
    }
}

fn resolve_ast_node(node: Rc<AstNode>) -> Rc<AstNode> {
    match node.value {
        Function::Add => {
            if let Function::Number(lhs) = node.children[0].value {
                if let Function::Number(rhs) = node.children[1].value {
                    return AstNode::make_leaf(Function::Number(lhs + rhs));
                }
            }
        }
        _ => unimplemented!(),
    }
    panic!("invalid status");
}

fn evaluate(
    node: &Rc<AstNode>,
    ast_nodes: &HashMap<i64, Rc<AstNode>>,
    depth: usize,
) -> Result<Rc<AstNode>> {
    if depth > 10 {
        return Err(EvaluateError);
    }
    match node.value {
        Function::Ap => {
            let lhs = evaluate(&node.children[0], ast_nodes, depth + 1)?;
            let rhs = evaluate(&node.children[1], ast_nodes, depth + 1)?;
            match lhs.value {
                Function::Neg => {
                    if let Function::Number(value) = rhs.value {
                        Ok(AstNode::make_leaf(Function::Number(-value)))
                    } else {
                        let message = format!("{:?} can't Neg non-number", rhs);
                        panic!(message);
                    }
                }
                Function::Add => {
                    let mut children = lhs.children.clone();
                    children.push(rhs.clone());
                    let mut ret = Rc::new(AstNode {
                        value: lhs.value,
                        children: children,
                    });
                    if ret.children.len() == 2 {
                        ret = resolve_ast_node(ret);
                    }
                    Ok(ret)
                }
                _ => unimplemented!(),
            }
        }
        Function::Variable(id) => Ok(evaluate(&ast_nodes[&id], ast_nodes, depth + 1)?),
        _ => Ok(node.clone()),
    }
}

fn main() {
    let stack_size = 1024 * 1024 * 1024;
    let handler = thread::Builder::new()
        .name("interpreter".to_owned())
        .stack_size(stack_size)
        .spawn(move || {
            interpreter();
        })
        .unwrap();
    handler.join().unwrap();
}

fn interpreter() {
    let statements = load();
    let mut ast_nodes = HashMap::<i64, Rc<AstNode>>::new();
    for statement in statements.values() {
        let (node, index) = AstNode::parse_cells(&statement.cells, 0);
        assert!(index == statement.cells.len() - 1);
        ast_nodes.insert(statement.id, node);
    }
    // let node = evaluate(&ast_nodes[&1248], &ast_nodes, 0);
    // println!("{:#?}", node);
    let node = evaluate(&ast_nodes[&1251], &ast_nodes, 0);
    println!("{:#?}", node);
}

#[test]
fn test_parse_ast_node() {
    let node = AstNode::parse_str(":1248 = ap neg 14");
    assert!(node.value == Function::Ap);
    assert!(node.children[0].value == Function::Neg);
    assert!(node.children[0].children.len() == 0);
    assert!(node.children[1].value == Function::Number(14));
    assert!(node.children[1].children.len() == 0);
    let node = AstNode::parse_str(":1029 = ap ap cons 7 ap ap cons 123229502148636 nil");
    assert!(node.value == Function::Ap);
}

#[test]
fn test_node() {
    // TODO
    let statements = load();
    let mut ast_nodes = HashMap::<i64, Rc<AstNode>>::new();
    for statement in statements.values() {
        let (node, index) = AstNode::parse_cells(&statement.cells, 0);
        assert!(index == statement.cells.len() - 1);
        ast_nodes.insert(statement.id, node);
    }
    let node = evaluate(&ast_nodes[&1248], &ast_nodes, 0);
    println!("{:#?}", node);
    let node = evaluate(&ast_nodes[&1251], &ast_nodes, 0);
    println!("{:#?}", node);
}
