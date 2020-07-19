use std::collections::HashMap;
use std::convert::From;
use std::fmt;
use std::fs;
use std::rc::Rc;
use std::thread;

const USE_LIST: bool = false;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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
    List,
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
        let s = s.trim();
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

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct AstNode {
    pub value: Function,
    pub children: Vec<Rc<AstNode>>,
}
impl fmt::Display for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            Function::Cons => write!(f, "({} {})", self.children[0], self.children[1]),
            Function::Number(v) => write!(f, "{}", v),
            Function::Nil => write!(f, "nil"),
            Function::List => {
                write!(f, "[")?;
                for i in 0..self.children.len() {
                    if i != 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", self.children[i])?;
                }
                write!(f, "]")
            }
            _ => unimplemented!(),
        }
    }
}

impl AstNode {
    #[allow(dead_code)]
    pub fn parse_str(s: &str) -> Rc<Self> {
        let statement = Statement::new(s);
        let (node, index) = AstNode::parse_cells(&statement.cells, 0);
        assert!(index == statement.cells.len() - 1);
        return node;
    }
    pub fn parse_cells(cells: &Vec<Function>, cell_index: usize) -> (Rc<Self>, usize) {
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
    pub fn make_leaf(function: Function) -> Rc<Self> {
        Rc::new(AstNode {
            value: function,
            children: vec![],
        })
    }
    #[allow(dead_code)]
    pub fn make_nil() -> Rc<Self> {
        Self::make_leaf(Function::Nil)
    }
    #[allow(dead_code)]
    pub fn make_number(v: i64) -> Rc<Self> {
        Self::make_leaf(Function::Number(v))
    }
    #[allow(dead_code)]
    pub fn make_vector(x: i64, y: i64) -> Rc<Self> {
        Rc::new(AstNode {
            value: Function::Cons,
            children: vec![
                Self::make_leaf(Function::Number(x)),
                Self::make_leaf(Function::Number(y)),
            ],
        })
    }
    #[allow(dead_code)]
    pub fn make_cons(l: Rc<AstNode>, r: Rc<AstNode>) -> Rc<Self> {
        Rc::new(AstNode {
            value: Function::Cons,
            children: vec![l.clone(), r.clone()],
        })
    }
    #[allow(dead_code)]
    pub fn get_list_item(&self, index: usize) -> Rc<AstNode> {
        assert!(self.value == Function::Cons);
        if index == 0 {
            self.children[0].clone()
        } else {
            self.children[1].get_list_item(index - 1)
        }
    }
}

fn modulate_number(v: i64) -> String {
    let mut v = v;
    let mut ret = "".to_string();
    if v == 0 {
        return "010".to_string();
    } else if v > 0 {
        ret += "01";
    } else {
        ret += "10";
        v *= -1;
    }
    let mut n_4bits = 0;
    let mut tmp = v;
    while tmp > 0 {
        n_4bits += 1;
        tmp >>= 4;
    }
    for _i in 0..n_4bits {
        ret += "1";
    }
    ret += "0";
    ret += &format!("{:0width$b}", v, width = n_4bits * 4);
    ret
}

#[allow(dead_code)]
pub fn modulate(node: Rc<AstNode>) -> String {
    match node.value {
        Function::Nil => "00".to_string(),
        Function::Number(v) => modulate_number(v),
        Function::Cons => {
            "11".to_string()
                + &modulate(node.children[0].clone())
                + &modulate(node.children[1].clone())
        }
        Function::List => {
            let mut ret = "".to_string();
            for child in node.children.iter() {
                ret += "11";
                ret += &modulate(child.clone());
            }
            ret += "00";
            ret
        }
        _ => unimplemented!(),
    }
}

fn need_children(function: Function) -> Vec<usize> {
    match function {
        Function::Neg => vec![0],
        Function::Add | Function::Mul | Function::Div | Function::Lt | Function::Equal => {
            vec![0, 1]
        }
        Function::Cons => vec![],
        Function::Car => vec![],
        Function::Cdr => vec![],
        Function::Nil => vec![],
        Function::Isnil => vec![0],
        Function::True => vec![0],
        Function::False => vec![1],
        Function::Icombinator => vec![0],
        Function::Bcombinator => vec![],
        Function::Ccombinator => vec![],
        Function::Scombinator => vec![],
        _ => unimplemented!(),
    }
}

fn eq_dfs(lhs: Rc<AstNode>, rhs: Rc<AstNode>) -> bool {
    if lhs.value != rhs.value || lhs.children.len() != rhs.children.len() {
        return false;
    } else {
        for i in 0..lhs.children.len() {
            if !eq_dfs(lhs.children[i].clone(), rhs.children[i].clone()) {
                return false;
            }
        }
    }
    return true;
}

fn resolve_ast_node_with_memo(
    node: Rc<AstNode>,
    ast_nodes: &mut HashMap<i64, Rc<AstNode>>,
    memo: &mut HashMap<Rc<AstNode>, Rc<AstNode>>,
    depth: usize,
    use_memo: bool,
) -> Rc<AstNode> {
    if use_memo && memo.contains_key(&node) {
        return memo[&node].clone();
    }
    let ret = resolve_ast_node(node.clone(), ast_nodes, memo, depth, use_memo);
    if use_memo {
        memo.insert(node, ret.clone());
    }
    return ret;
}

fn resolve_ast_node(
    node: Rc<AstNode>,
    ast_nodes: &mut HashMap<i64, Rc<AstNode>>,
    memo: &mut HashMap<Rc<AstNode>, Rc<AstNode>>,
    depth: usize,
    use_memo: bool,
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
                    Function::Ap => evaluate(c.clone(), ast_nodes, memo, depth, use_memo),
                    Function::Variable(id) => {
                        evaluate(ast_nodes[&id].clone(), ast_nodes, memo, depth, use_memo)
                    }
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
            let ret = if eq_dfs(evaluated_children[0].clone(), evaluated_children[1].clone()) {
                Function::True
            } else {
                Function::False
            };
            return AstNode::make_leaf(ret);
        }
        Function::Cons => {
            let leaf = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![node.children[2].clone(), node.children[0].clone()],
            });
            let parent = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![leaf, node.children[1].clone()],
            });
            return evaluate(parent, ast_nodes, memo, depth, use_memo);
        }
        Function::True => {
            return evaluated_children[0].clone();
        }
        Function::False => {
            return evaluated_children[1].clone();
        }
        Function::Car => {
            let leaf = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![node.children[0].clone(), AstNode::make_leaf(Function::True)],
            });
            return evaluate(leaf, ast_nodes, memo, depth, use_memo);
        }
        Function::Cdr => {
            let leaf = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![
                    node.children[0].clone(),
                    AstNode::make_leaf(Function::False),
                ],
            });
            return evaluate(leaf, ast_nodes, memo, depth, use_memo);
        }
        Function::Nil => {
            return AstNode::make_leaf(Function::True);
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
        Function::Ccombinator => {
            let leaf = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![node.children[0].clone(), node.children[2].clone()],
            });
            let parent = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![leaf, node.children[1].clone()],
            });
            return evaluate(parent, ast_nodes, memo, depth, use_memo);
        }
        Function::Bcombinator => {
            let leaf = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![node.children[1].clone(), node.children[2].clone()],
            });
            let parent = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![node.children[0].clone(), leaf],
            });
            return evaluate(parent, ast_nodes, memo, depth, use_memo);
        }
        Function::Scombinator => {
            let left = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![node.children[0].clone(), node.children[2].clone()],
            });
            let right = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![node.children[1].clone(), node.children[2].clone()],
            });
            let parent = Rc::new(AstNode {
                value: Function::Ap,
                children: vec![left, right],
            });
            return evaluate(parent, ast_nodes, memo, depth, use_memo);
        }
        _ => unimplemented!(),
    }
    println!("{:#?}", node.value);
    panic!("invalid status");
}

pub fn evaluate(
    node: Rc<AstNode>,
    ast_nodes: &mut HashMap<i64, Rc<AstNode>>,
    memo: &mut HashMap<Rc<AstNode>, Rc<AstNode>>,
    depth: usize,
    use_memo: bool,
) -> Rc<AstNode> {
    match node.value {
        Function::Ap => {
            let lhs = evaluate(
                node.children[0].clone(),
                ast_nodes,
                memo,
                depth + 1,
                use_memo,
            );
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
                | Function::Nil
                | Function::Isnil
                | Function::Icombinator => {
                    if ret.children.len() == 1 {
                        ret = resolve_ast_node_with_memo(ret, ast_nodes, memo, depth + 1, use_memo);
                    }
                    ret
                }
                Function::Add
                | Function::Mul
                | Function::Div
                | Function::Lt
                | Function::Equal
                | Function::True
                | Function::False => {
                    if ret.children.len() == 2 {
                        ret = resolve_ast_node_with_memo(ret, ast_nodes, memo, depth + 1, use_memo);
                    }
                    ret
                }
                Function::Ccombinator
                | Function::Bcombinator
                | Function::Scombinator
                | Function::Cons => {
                    if ret.children.len() == 3 {
                        ret = resolve_ast_node_with_memo(ret, ast_nodes, memo, depth + 1, use_memo);
                    }
                    ret
                }
                _ => unimplemented!(),
            }
        }
        Function::Variable(id) => {
            // println!("{}", id);
            evaluate(ast_nodes[&id].clone(), ast_nodes, memo, depth + 1, use_memo)
        }
        _ => node.clone(),
    }
}

pub fn usual(
    node: Rc<AstNode>,
    ast_nodes: &mut HashMap<i64, Rc<AstNode>>,
    memo: &mut HashMap<Rc<AstNode>, Rc<AstNode>>,
    depth: usize,
) -> Rc<AstNode> {
    let evaluated_children: Vec<Rc<AstNode>> = node
        .children
        .iter()
        .enumerate()
        .map(|(_i, c)| match c.value {
            Function::Ap => evaluate(c.clone(), ast_nodes, memo, depth + 1, true),
            Function::Variable(id) => {
                evaluate(ast_nodes[&id].clone(), ast_nodes, memo, depth + 1, true)
            }
            _ => c.clone(),
        })
        .collect();
    match node.value {
        Function::Cons => {
            let left = usual(evaluated_children[0].clone(), ast_nodes, memo, depth + 1);
            let right = usual(evaluated_children[1].clone(), ast_nodes, memo, depth + 1);
            if USE_LIST && right.value == Function::Nil {
                return Rc::new(AstNode {
                    value: Function::List,
                    children: vec![left],
                });
            } else if right.value == Function::List {
                let mut children = vec![left];
                children.append(&mut right.children.clone());
                return Rc::new(AstNode {
                    value: Function::List,
                    children: children,
                });
            } else {
                return Rc::new(AstNode {
                    value: Function::Cons,
                    children: vec![left, right],
                });
            }
        }
        Function::Car => {
            let left = usual(evaluated_children[0].clone(), ast_nodes, memo, depth + 1);
            return left;
        }
        Function::Cdr => {
            let right = usual(evaluated_children[1].clone(), ast_nodes, memo, depth + 1);
            return right;
        }
        _ => node,
    }
}

#[allow(dead_code)]
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
    let s = ":1 = ap ap :0 nil ap ap cons 0 0";
    let mut memo = HashMap::new();
    let node = AstNode::parse_str(s);
    let node = evaluate(node.clone(), &mut ast_nodes, &mut memo, 0, true);
    let node = usual(node.clone(), &mut ast_nodes, &mut memo, 0);
    println!("{}", node);
    // let node = evaluate(ast_nodes[&1141].clone(), &mut ast_nodes, &mut memo, 0);
    // println!("{:#?}", node);
    // let node = evaluate(ast_nodes[&1109].clone(), &mut ast_nodes, &mut memo, 0);
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
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    assert!(node.value == Function::Add);
    assert!(node.children[0].value == Function::Ap);
    assert!(node.children[0].children.len() == 2);
    let node = AstNode::parse_str(":112 = ap ap add ap ap add 1 2 3");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    assert!(node.value == Function::Number(6));
    assert!(node.children.len() == 0);
}

#[test]
fn test_lasy_evaluation_cons() {
    let node1 = AstNode::parse_str(":111 = :111");
    let mut ast_nodes = HashMap::new();
    ast_nodes.insert(111, node1);

    let node = AstNode::parse_str(":112 = ap ap cons ap neg :111 nil");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::Cons);
    assert!(node.children[0].value == Function::Ap);
    assert!(node.children[0].children.len() == 2);

    let node = AstNode::parse_str(":112 = ap car ap ap cons ap neg 1 :111");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::Number(-1));

    let node = AstNode::parse_str(":112 = ap cdr ap ap cons ap neg :111 nil");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::Nil);
}

#[test]
fn test_lazy_true_false() {
    let node1 = AstNode::parse_str(":111 = :111");
    let mut ast_nodes = HashMap::new();
    ast_nodes.insert(111, node1);

    let node = AstNode::parse_str(":111 = ap ap t 0 1");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    assert!(node.value == Function::Number(0));

    let node = AstNode::parse_str(":111 = ap ap f 0 1");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    assert!(node.value == Function::Number(1));

    let node = AstNode::parse_str(":112 = ap ap t 1 :111");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    assert!(node.value == Function::Number(1));

    let node = AstNode::parse_str(":112 = ap ap f :111 1");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    assert!(node.value == Function::Number(1));
}

#[test]
fn test_cmp() {
    let node1 = AstNode::parse_str(":111 = :111");
    let mut ast_nodes = HashMap::new();
    ast_nodes.insert(111, node1);

    let node = AstNode::parse_str(":112 = ap ap lt 0 1");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::True);
    let node = AstNode::parse_str(":112 = ap ap lt 0 0");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::False);
    let node = AstNode::parse_str(":112 = ap ap eq 0 0");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::True);
    let node = AstNode::parse_str(":112 = ap ap eq 1 0");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::False);

    let node = AstNode::parse_str(":112 = ap ap eq nil nil");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::True);

    let node = AstNode::parse_str(":112 = ap ap eq ap ap cons 1 2 ap ap cons 1 2");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::True);

    let node = AstNode::parse_str(":112 = ap ap eq ap ap cons 1 2 ap ap cons 1 3");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::False);

    let node = AstNode::parse_str(":112 = ap ap eq i b");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::False);

    let node = AstNode::parse_str(":112 = ap ap eq i i");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::True);
}

#[test]
fn test_isnil() {
    let node = AstNode::parse_str(":112 = ap isnil nil");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    assert!(node.value == Function::True);
    let node = AstNode::parse_str(":112 = ap isnil ap ap cons 1 nil");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    assert!(node.value == Function::False);
    return;
}

#[test]
fn test_icombinator() {
    let node = AstNode::parse_str(":112 = ap i 1");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    assert!(node.value == Function::Number(1));

    let node = AstNode::parse_str(":112 = ap i i");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    assert!(node.value == Function::Icombinator);
    return;
}

#[test]
fn test_ccombinator() {
    let node = AstNode::parse_str(":112 = ap ap ap c add 1 2");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::Number(3));
    return;
}

#[test]
fn test_bcombinator() {
    let node = AstNode::parse_str(":112 = ap ap ap b neg neg 2");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::Number(2));
    let node = AstNode::parse_str(":112 = ap ap ap ap b add neg 2 3");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::Number(1));
    return;
}

#[test]
fn test_scombinator() {
    let node = AstNode::parse_str(":111 = ap ap ap s mul ap add 1 6");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    assert!(node.value == Function::Number(42));
}

#[test]
fn test_power2() {
    let node1 = AstNode::parse_str(
        ":111 = ap ap s ap ap c ap eq 0 1 ap ap b ap mul 2 ap ap b :111 ap add -1",
    );
    let mut ast_nodes = HashMap::new();
    ast_nodes.insert(111, node1);

    let ans1 = vec![
        ":112 = ap ap ap ap c ap eq 0 1 0 ap ap ap b ap mul 2 ap ap b :111 ap add -1 0",
        ":112 = ap ap ap ap eq 0 0 1 ap ap ap b ap mul 2 ap ap b :111 ap add -1 0",
        ":112 = ap ap t 1 ap ap ap b ap mul 2 ap ap b :111 ap add -1 0",
    ];
    let ans2 = vec![
        ":112 = ap ap ap s ap ap c ap eq 0 1 ap ap b ap mul 2 ap ap b :111 ap add -1 1",
        ":112 = ap ap ap ap c ap eq 0 1 1 ap ap ap b ap mul 2 ap ap b :111 ap add -1 1",
        ":112 = ap ap ap ap eq 0 1 1 ap ap ap b ap mul 2 ap ap b :111 ap add -1 1",
        ":112 = ap ap f 1 ap ap ap b ap mul 2 ap ap b :111 ap add -1 1",
        ":112 = ap ap ap b ap mul 2 ap ap b :111 ap add -1 1",
        ":112 = ap ap mul 2 ap ap ap b :111 ap add -1 1",
        ":112 = ap ap mul 2 ap :111 ap ap add -1 1",
        ":112 = ap ap mul 2 ap ap ap s ap ap c ap eq 0 1 ap ap b ap mul 2 ap ap b :111 ap add -1 ap ap add -1 1",
        ":112 = ap ap mul 2 ap ap ap ap c ap eq 0 1 ap ap add -1 1 ap ap ap b ap mul 2 ap ap b :111 ap add -1 ap ap add -1 1",
        ":112 = ap ap mul 2 ap ap ap ap eq 0 ap ap add -1 1 1 ap ap ap b ap mul 2 ap ap b :111 ap add -1 ap ap add -1 1",
        ":112 = ap ap mul 2 ap ap ap ap eq 0 0 1 ap ap ap b ap mul 2 ap ap b :111 ap add -1 ap ap add -1 1",
        ":112 = ap ap mul 2 ap ap t 1 ap ap ap b ap mul 2 ap ap b :111 ap add -1 ap ap add -1 1",
        ":112 = ap ap mul 2 1",
        ":112 = 2",
    ];
    let ans4 =
        vec![":112 = ap ap ap s ap ap c ap eq 0 1 ap ap b ap mul 2 ap ap b :111 ap add -1 2"];
    for &s in ans1.iter() {
        let node = AstNode::parse_str(s);
        let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
        // println!("{:#?}", node);
        assert!(node.value == Function::Number(1));
    }
    for &s in ans2.iter() {
        let node = AstNode::parse_str(s);
        let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
        // println!("{:#?}", node);
        assert!(node.value == Function::Number(2));
    }
    for &s in ans4.iter() {
        let node = AstNode::parse_str(s);
        let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
        // println!("{:#?}", node);
        assert!(node.value == Function::Number(4));
    }
}

#[test]
fn test_odd_even() {
    let node1 = AstNode::parse_str(":111 = ap ap cons 1 :112");
    let mut ast_nodes = HashMap::new();
    ast_nodes.insert(111, node1);
    let node2 = AstNode::parse_str(":112 = ap ap cons 2 :111");
    ast_nodes.insert(112, node2);
    let node = AstNode::parse_str(":113 = ap car :111");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::Number(1));

    let node = AstNode::parse_str(":113 = ap car ap cdr :111");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::Number(2));
    let node = AstNode::parse_str(":113 = ap car ap cdr ap cdr :111");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::Number(1));
    return;
}

#[test]
fn test_multi_function() {
    let mut ast_nodes = HashMap::new();
    let node = AstNode::parse_str(":111 = ap ap i ap :112 2 3");
    ast_nodes.insert(111, node);
    let node = AstNode::parse_str(":112 = :113");
    ast_nodes.insert(112, node);
    let node = AstNode::parse_str(":113 = t");
    ast_nodes.insert(113, node);
    let node = AstNode::parse_str(":114 = :111");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    assert!(node.value == Function::Number(2));
    return;
}

#[test]
fn test_cons_add() {
    let node = AstNode::parse_str(":113 = ap ap ap cons 1 1 add");
    let node = evaluate(node, &mut HashMap::new(), &mut HashMap::new(), 0, true);
    // println!("{:#?}", node);
    assert!(node.value == Function::Number(2));
}

#[test]
fn test_nil_bottom() {
    let mut ast_nodes = HashMap::new();
    let node = AstNode::parse_str(":111 = :111");
    ast_nodes.insert(111, node);

    let node = AstNode::parse_str(":113 = ap ap ap nil :111 42 :111");
    let node = evaluate(node, &mut ast_nodes, &mut HashMap::new(), 0, true);
    println!("{:#?}", node);
    assert!(node.value == Function::Number(42));
}

#[test]
fn test_modulate() {
    assert!(modulate_number(0) == "010");
    assert!(modulate_number(1) == "01100001");
    assert!(modulate_number(2) == "01100010");
    assert!(modulate_number(-1) == "10100001");
    assert!(modulate_number(256) == "011110000100000000");
    assert!(modulate(AstNode::make_number(2)) == "01100010");

    assert!(modulate(AstNode::make_nil()) == "00");
    assert!(modulate(AstNode::make_cons(AstNode::make_nil(), AstNode::make_nil())) == "110000");
    assert!(
        modulate(AstNode::make_cons(
            AstNode::make_number(0),
            AstNode::make_nil()
        )) == "1101000"
    );
    assert!(
        modulate(AstNode::make_cons(
            AstNode::make_number(1),
            AstNode::make_number(2)
        )) == "110110000101100010"
    );
    assert!(
        modulate(AstNode::make_cons(
            AstNode::make_number(1),
            AstNode::make_cons(AstNode::make_number(2), AstNode::make_nil(),)
        )) == "1101100001110110001000"
    );
}
