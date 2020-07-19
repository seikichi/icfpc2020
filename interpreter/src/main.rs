use core::{AstNode, GalaxyEvaluator, demodulate};
use std::thread;
use std::env;

#[allow(dead_code)]
fn main() {
    let stack_size = 1024 * 1024 * 1024;
    let handler = thread::Builder::new()
        .name("interpreter".to_owned())
        .stack_size(stack_size)
        .spawn(move || {
            let args: Vec<String> = env::args().collect();

            let state = if args.len() >= 2 {
                demodulate(&args[1])
            } else {
                AstNode::make_nil()
            };

            let vector = if args.len() >= 3 {
                demodulate(&args[2])
            } else {
                AstNode::make_vector(0, 0)
            };

            let mut evaluator = GalaxyEvaluator::new();
            let node = evaluator.evaluate(state, vector);
            println!("{}", node);
        })
        .unwrap();
    handler.join().unwrap();
}
