use core::{AstNode, GalaxyEvaluator};
use std::thread;

#[allow(dead_code)]
fn main() {
    let stack_size = 1024 * 1024 * 1024;
    let handler = thread::Builder::new()
        .name("interpreter".to_owned())
        .stack_size(stack_size)
        .spawn(move || {
            let mut evaluator = GalaxyEvaluator::new();
            let node = evaluator.evaluate(AstNode::make_nil(), AstNode::make_vector(0, 0));
            println!("{}", node);
        })
        .unwrap();
    handler.join().unwrap();
}
