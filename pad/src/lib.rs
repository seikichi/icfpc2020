mod utils;

use core::{AstNode, GalaxyEvaluator};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {}

#[wasm_bindgen]
pub fn evaluate_galaxy() -> String {
    let mut evaluator = GalaxyEvaluator::new();
    let node = evaluator.evaluate(AstNode::make_nil(), AstNode::make_vector(0, 0));
    format!("{:#?}", node)
}

#[wasm_bindgen]
pub struct GalaxyEvaluatorProxy {
    evaluator: GalaxyEvaluator,
}

#[wasm_bindgen]
impl GalaxyEvaluatorProxy {
    pub fn new() -> Self {
        let mut evaluator = GalaxyEvaluator::new();
        Self { evaluator }
    }

    pub fn interact(&mut self) -> String {
        let node = self
            .evaluator
            .evaluate(AstNode::make_nil(), AstNode::make_vector(0, 0));
        format!("HAHAHA: {:#?}", node)
    }
}
