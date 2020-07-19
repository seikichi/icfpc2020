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
    state: Rc<AstNode>,

    cells: Vec<Vec<u32>>,
    flag: i64,
    ymin: i64,
    xmin: i64,
    ymax: i64,
    xmax: i64,
    y: i64,
    x: i64,
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
            state: AstNode::make_nil(),
            cells: vec![vec![]],
            flag: 0,
            ymin: 0,
            xmin: 0,
            ymax: 0,
            xmax: 0,
            y: 0,
            x: 0,
        }
    }

    pub fn interact(&mut self, y: u32, x: u32) {
        let y = y as i64 + self.ymin;
        let x = x as i64 + self.xmin;
        self.y = y;
        self.x = x;

        let node = self
            .evaluator
            .evaluate(self.state.clone(), AstNode::make_vector(x as i64, y as i64));
        self.flag = node.get_list_item(0).get_number();
        self.state = node.get_list_item(1);
        let data = node.get_list_item(2);
        self.update_cells(data.clone());
    }

    pub fn debug(&self) -> String {
        //format!("{:#?}", self.current.get_list_item(2))
        // format!("{:?}", self.parse_data())
        // format!("flag={:#?}, state={:#?}", self.flag, self.state)
        format!(
            "flag={}, y={:#?}, x={:#?}, ymin={}, xmin={}, ymax={}, xmax={}",
            self.flag, self.y, self.x, self.ymin, self.xmin, self.ymax, self.xmax
        )
        // "NO DEBUG INFO".to_string()
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

    fn update_cells(&mut self, data: Rc<AstNode>) {
        let points_lists = self.parse_data(data);

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
                self.cells[(y - ymin) as usize][(x - xmin) as usize] |= 1 << i;
            }
        }
    }

    fn parse_data(&self, data: Rc<AstNode>) -> Vec<Vec<Point>> {
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
