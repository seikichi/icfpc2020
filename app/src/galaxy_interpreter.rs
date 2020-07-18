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

fn need_children(function: Function) -> Vec<usize> {
    match function {
        Function::Neg => vec![0],
        Function::Add | Function::Mul | Function::Div | Function::Lt | Function::Equal => {
            vec![0, 1]
        }
        Function::Cons => vec![],
        Function::Car => vec![0],
        Function::Cdr => vec![0],
        Function::Isnil => vec![0],
        Function::True => vec![0],
        Function::False => vec![1],
        Function::Icombinator => vec![0],
        _ => unimplemented!(),
    }
}

fn resolve_ast_node(
    node: Rc<AstNode>,
    ast_nodes: &HashMap<i64, Rc<AstNode>>,
    depth: usize,
) -> Rc<AstNode> {
    let wants = need_children(node.value);
    let evaluated_children: Vec<Rc<AstNode>> = node
        .children
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if !wants.contains(&i) {
                c.clone()
            } else {
                match c.value {
                    Function::Ap => evaluate(c.clone(), ast_nodes, depth),
                    Function::Variable(id) => evaluate(ast_nodes[&id].clone(), ast_nodes, depth),
                    _ => c.clone(),
                }
            }
        })
        .collect();
    match node.value {
        Function::Neg => match evaluated_children[0].value {
            Function::Number(v) => return AstNode::make_leaf(Function::Number(-v)),
            _ => unreachable!(),
        },
        Function::Add => {
            if let Function::Number(lhs) = evaluated_children[0].value {
                if let Function::Number(rhs) = evaluated_children[1].value {
                    return AstNode::make_leaf(Function::Number(lhs + rhs));
                }
            }
        }
        Function::Mul => {
            if let Function::Number(lhs) = evaluated_children[0].value {
                if let Function::Number(rhs) = evaluated_children[1].value {
                    return AstNode::make_leaf(Function::Number(lhs * rhs));
                }
            }
        }
        Function::Div => {
            if let Function::Number(lhs) = evaluated_children[0].value {
                if let Function::Number(rhs) = evaluated_children[1].value {
                    return AstNode::make_leaf(Function::Number(lhs / rhs));
                }
            }
        }
        Function::Lt => {
            if let Function::Number(lhs) = evaluated_children[0].value {
                if let Function::Number(rhs) = evaluated_children[1].value {
                    let ret = if lhs < rhs {
                        Function::True
                    } else {
                        Function::False
                    };
                    return AstNode::make_leaf(ret);
                }
            }
        }
        Function::Equal => {
            if let Function::Number(lhs) = evaluated_children[0].value {
                if let Function::Number(rhs) = evaluated_children[1].value {
                    let ret = if lhs == rhs {
                        Function::True
                    } else {
                        Function::False
                    };
                    return AstNode::make_leaf(ret);
                }
            }
        }
        Function::Cons => {
            return Rc::new(AstNode {
                value: node.value,
                children: evaluated_children,
            });
        }
        Function::True => {
            return evaluated_children[0].clone();
        }
        Function::False => {
            return evaluated_children[1].clone();
        }
        Function::Car => {
            let cons_cell = evaluated_children[0].clone();
            return evaluate(cons_cell.children[0].clone(), ast_nodes, depth);
        }
        Function::Cdr => {
            let cons_cell = evaluated_children[0].clone();
            return evaluate(cons_cell.children[1].clone(), ast_nodes, depth);
        }
        Function::Isnil => {
            let ret = if evaluated_children[0].value == Function::Nil {
                Function::True
            } else {
                Function::False
            };
            return AstNode::make_leaf(ret);
        }
        Function::Icombinator => {
            return evaluated_children[0].clone();
        }
        _ => unimplemented!(),
    }
    panic!("invalid status");
}

fn evaluate(node: Rc<AstNode>, ast_nodes: &HashMap<i64, Rc<AstNode>>, depth: usize) -> Rc<AstNode> {
    if depth > 100 {
        panic!("too deep!!")
    }
    match node.value {
        Function::Ap => {
            let lhs = evaluate(node.children[0].clone(), ast_nodes, depth + 1);
            let rhs = &node.children[1];
            let mut children = lhs.children.clone();
            children.push(rhs.clone());
            let mut ret = Rc::new(AstNode {
                value: lhs.value,
                children: children,
            });
            match lhs.value {
                Function::Neg
                | Function::Car
                | Function::Cdr
                | Function::Isnil
                | Function::Icombinator => {
                    if ret.children.len() == 1 {
                        ret = resolve_ast_node(ret, ast_nodes, depth);
                    }
                    ret
                }
                Function::Add
                | Function::Mul
                | Function::Div
                | Function::Cons
                | Function::Lt
                | Function::Equal
                | Function::True
                | Function::False => {
                    if ret.children.len() == 2 {
                        ret = resolve_ast_node(ret, ast_nodes, depth)
                    }
                    ret
                }
                _ => unimplemented!(),
            }
        }
        Function::Variable(id) => evaluate(ast_nodes[&id].clone(), ast_nodes, depth + 1),
        _ => node.clone(),
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
    let node = evaluate(ast_nodes[&1248].clone(), &ast_nodes, 0);
    println!("{:#?}", node);
    // let node = evaluate(&ast_nodes[&1251].clone(), &ast_nodes, 0);
    // println!("{:#?}", node);
    // let node = evaluate(&ast_nodes[&1109].clone(), &ast_nodes, 0);
    // println!("{:#?}", node);
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
fn test_lazy_evaluation() {
    let node = AstNode::parse_str(":111 = ap add ap ap add 1 2");
    let node = evaluate(node, &HashMap::new(), 0);
    assert!(node.value == Function::Add);
    assert!(node.children[0].value == Function::Ap);
    assert!(node.children[0].children.len() == 2);
    let node = AstNode::parse_str(":112 = ap ap add ap ap add 1 2 3");
    let node = evaluate(node, &HashMap::new(), 0);
    assert!(node.value == Function::Number(6));
    assert!(node.children.len() == 0);
}

#[test]
fn test_lasy_evaluation_cons() {
    let node1 = AstNode::parse_str(":111 = :111");
    let mut ast_nodes = HashMap::new();
    ast_nodes.insert(111, node1);

    let node = AstNode::parse_str(":112 = ap ap cons ap neg :111 nil");
    let node = evaluate(node, &ast_nodes, 0);
    // println!("{:#?}", node);
    assert!(node.value == Function::Cons);
    assert!(node.children[0].value == Function::Ap);
    assert!(node.children[0].children.len() == 2);

    let node = AstNode::parse_str(":112 = ap car ap ap cons ap neg 1 :111");
    let node = evaluate(node, &ast_nodes, 0);
    // println!("{:#?}", node);
    assert!(node.value == Function::Number(-1));

    let node = AstNode::parse_str(":112 = ap cdr ap ap cons ap neg :111 nil");
    let node = evaluate(node, &ast_nodes, 0);
    // println!("{:#?}", node);
    assert!(node.value == Function::Nil);
}

#[test]
fn test_lazy_true_false() {
    let node1 = AstNode::parse_str(":111 = :111");
    let mut ast_nodes = HashMap::new();
    ast_nodes.insert(111, node1);

    let node = AstNode::parse_str(":111 = ap ap t 0 1");
    let node = evaluate(node, &HashMap::new(), 0);
    assert!(node.value == Function::Number(0));

    let node = AstNode::parse_str(":111 = ap ap f 0 1");
    let node = evaluate(node, &HashMap::new(), 0);
    assert!(node.value == Function::Number(1));

    let node = AstNode::parse_str(":112 = ap ap t 1 :111");
    let node = evaluate(node, &ast_nodes, 0);
    assert!(node.value == Function::Number(1));

    let node = AstNode::parse_str(":112 = ap ap f :111 1");
    let node = evaluate(node, &ast_nodes, 0);
    assert!(node.value == Function::Number(1));
}

#[test]
fn test_cmp() {
    let node1 = AstNode::parse_str(":111 = :111");
    let mut ast_nodes = HashMap::new();
    ast_nodes.insert(111, node1);

    let node = AstNode::parse_str(":112 = ap ap lt 0 1");
    let node = evaluate(node, &ast_nodes, 0);
    // println!("{:#?}", node);
    assert!(node.value == Function::True);
    let node = AstNode::parse_str(":112 = ap ap lt 0 0");
    let node = evaluate(node, &ast_nodes, 0);
    // println!("{:#?}", node);
    assert!(node.value == Function::False);
    let node = AstNode::parse_str(":112 = ap ap eq 0 0");
    let node = evaluate(node, &ast_nodes, 0);
    // println!("{:#?}", node);
    assert!(node.value == Function::True);
    let node = AstNode::parse_str(":112 = ap ap eq 1 0");
    let node = evaluate(node, &ast_nodes, 0);
    // println!("{:#?}", node);
    assert!(node.value == Function::False);
}

#[test]
fn test_isnil() {
    let node = AstNode::parse_str(":112 = ap isnil nil");
    let node = evaluate(node, &HashMap::new(), 0);
    assert!(node.value == Function::True);
    let node = AstNode::parse_str(":112 = ap isnil ap ap cons 1 nil");
    let node = evaluate(node, &HashMap::new(), 0);
    assert!(node.value == Function::False);
    return;
}

#[test]
fn test_icombinator() {
    let node = AstNode::parse_str(":112 = ap i 1");
    let node = evaluate(node, &HashMap::new(), 0);
    assert!(node.value == Function::Number(1));

    let node = AstNode::parse_str(":112 = ap i i");
    let node = evaluate(node, &HashMap::new(), 0);
    assert!(node.value == Function::Icombinator);
    return;
}
