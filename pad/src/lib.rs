mod utils;

use core::{AstNode, Function, GalaxyEvaluator};
use std::rc::Rc;
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
    current: Rc<AstNode>,

    cells: Vec<Vec<u32>>,
    ymin: i64,
    xmin: i64,
    ymax: i64,
    xmax: i64,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Point {
    x: i64,
    y: i64,
}

#[wasm_bindgen]
impl GalaxyEvaluatorProxy {
    pub fn new() -> Self {
        let evaluator = GalaxyEvaluator::new();
        let current = Rc::new(AstNode {
            value: Function::Number(42),
            children: vec![],
        });
        Self {
            evaluator,
            current,
            cells: vec![vec![]],
            ymin: 0,
            xmin: 0,
            ymax: 0,
            xmax: 0,
        }
    }

    pub fn interact(&mut self) {
        let node = self
            .evaluator
            .evaluate(AstNode::make_nil(), AstNode::make_vector(0, 0));
        self.current = node;
        self.update_cells();
    }

    pub fn debug(&self) -> String {
        //format!("{:#?}", self.current.get_list_item(2))
        // format!("{:?}", self.parse_data())
        // format!("{:#?}", self.cells[6])
    }

    pub fn width(&self) -> u32 {
        self.cells[0].len() as u32
    }

    pub fn height(&self) -> u32 {
        self.cells.len() as u32
    }

    pub fn color(&self, y: u32, x: u32) -> u32 {
        self.cells[y as usize][x as usize] as u32
    }

    fn update_cells(&mut self) {
        let points_lists = self.parse_data();

        let mut ymin = 1 << 60;
        let mut xmin = 1 << 60;
        let mut ymax = -(1 << 60);
        let mut xmax = -(1 << 60);

        for points in &points_lists {
            for &Point { x, y } in points {
                ymin = std::cmp::min(ymin, y);
                xmin = std::cmp::min(xmin, x);
                ymax = std::cmp::max(ymax, y);
                xmax = std::cmp::max(xmax, x);
            }
        }

        let width = xmax - xmin + 1;
        let height = ymax - ymin + 1;

        self.ymin = ymin;
        self.xmin = xmin;
        self.ymax = ymax;
        self.xmax = xmax;
        self.cells = vec![vec![0; width as usize]; height as usize];

        for (i, points) in points_lists.iter().enumerate() {
            for &Point { x, y } in points {
                self.cells[(y - ymin) as usize][(x - xmin) as usize] = (i + 1) as u32;
            }
        }
    }

    fn parse_data(&self) -> Vec<Vec<Point>> {
        let data = self.current.get_list_item(2);

        let mut list = vec![];
        let mut cell = data;

        while cell.value == Function::Cons {
            list.push(Self::parse_point_list(cell.children[0].clone()));
            cell = cell.children[1].clone();
        }

        list
    }

    fn parse_point_list(cell: Rc<AstNode>) -> Vec<Point> {
        let mut list = vec![];
        let mut cell = cell;

        while cell.value == Function::Cons {
            let x = cell.children[0].children[0].get_number();
            let y = cell.children[0].children[1].get_number();
            list.push(Point { x, y });

            cell = cell.children[1].clone();
        }

        list
    }
}
