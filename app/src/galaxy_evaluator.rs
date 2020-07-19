mod galaxy_interpreter;

use galaxy_interpreter::{evaluate, load, usual, AstNode, Function};
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;

pub struct GalaxyEvaluator {
    ast_nodes: HashMap<i64, Rc<AstNode>>,
    memo: HashMap<Rc<AstNode>, Rc<AstNode>>,
}

impl GalaxyEvaluator {
    pub fn new() -> Self {
        let statements = load();
        let mut ast_nodes = HashMap::<i64, Rc<AstNode>>::new();
        for statement in statements.values() {
            let (node, index) = AstNode::parse_cells(&statement.cells, 0);
            assert!(index == statement.cells.len() - 1);
            ast_nodes.insert(statement.id, node);
        }
        let memo = HashMap::new();
        GalaxyEvaluator {
            ast_nodes: ast_nodes,
            memo: memo,
        }
    }
    pub fn evaluate(&mut self, state: Rc<AstNode>, vect: Rc<AstNode>) -> Rc<AstNode> {
        assert!(vect.value == Function::Cons);
        assert!(vect.children.len() == 2);
        let node = Rc::new(AstNode {
            value: Function::Ap,
            children: vec![
                Rc::new(AstNode {
                    value: Function::Ap,
                    children: vec![AstNode::make_leaf(Function::Variable(0)), state.clone()],
                }),
                vect.clone(),
            ],
        });
        let node = evaluate(node, &mut self.ast_nodes, &mut self.memo, 0, true);
        let node = usual(node.clone(), &mut self.ast_nodes, &mut self.memo, 0);
        return node;
    }
}

fn main() {
    let stack_size = 1024 * 1024 * 1024;
    let handler = thread::Builder::new()
        .name("transpiler".to_owned())
        .stack_size(stack_size)
        .spawn(move || {
            let mut evaluator = GalaxyEvaluator::new();
            let node = evaluator.evaluate(AstNode::make_nil(), AstNode::make_vector(0, 0));
            println!("{}", node);
        })
        .unwrap();
    handler.join().unwrap();
}

// #[test]
// fn can_evaluate_initial_state() {
//     let stack_size = 1024 * 1024 * 1024;
//     let handler = thread::Builder::new()
//         .name("transpiler".to_owned())
//         .stack_size(stack_size)
//         .spawn(move || {
//             let mut evaluator = GalaxyEvaluator::new();
//             let node = evaluator.evaluate(AstNode::make_nil(), AstNode::make_vector(0, 0));
//             println!("{}", node);
//         })
//         .unwrap();
//     handler.join().unwrap();
// }
