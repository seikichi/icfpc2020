mod utils;

use std::collections::HashMap;
use std::rc::Rc;

use galaxy_evalua
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {}

const GALAXY_TEXT: &str = include_str!("../../resource/galaxy.txt");

#[wasm_bindgen]
pub fn evaluate_galaxy() -> String {
    let statements: HashMap<_, _> = GALAXY_TEXT
        .split("\n")
        .map(|s| {
            let statement = Statement::new(s);
            (statement.id, statement)
        })
        .collect();

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
    return format!("{:#?}", node);
}
