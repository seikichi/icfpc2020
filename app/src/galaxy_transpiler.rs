mod galaxy_interpreter;

use std::thread;
use galaxy_interpreter::{load, Function, Statement};

fn parse_cells(cells: &[Function]) -> (String, &[Function]) {
    match cells[0] {
        Function::Number(n) => (n.to_string(), &cells[1..]),
        Function::Neg => ("_neg".to_string(), &cells[1..]),
        Function::Equal => ("_eq".to_string(), &cells[1..]),
        Function::Cons => ("_cons".to_string(), &cells[1..]),
        Function::Nil => ("_nil".to_string(), &cells[1..]),
        Function::Add => ("_add".to_string(), &cells[1..]),
        Function::Mul => ("_mul".to_string(), &cells[1..]),
        Function::Div => ("_div".to_string(), &cells[1..]),
        Function::Car => ("_car".to_string(), &cells[1..]),
        Function::Cdr => ("_cdr".to_string(), &cells[1..]),
        Function::Isnil => ("_isnil".to_string(), &cells[1..]),
        Function::Lt => ("_lt".to_string(), &cells[1..]),
        Function::Bcombinator => ("_b".to_string(), &cells[1..]),
        Function::Ccombinator => ("_c".to_string(), &cells[1..]),
        Function::Scombinator => ("_s".to_string(), &cells[1..]),
        Function::Icombinator => ("_i".to_string(), &cells[1..]),
        Function::True => ("_t".to_string(), &cells[1..]),
        Function::False => ("_f".to_string(), &cells[1..]),
        Function::Variable(i) => (format!("(delay z{})", i), &cells[1..]),
        Function::Ap => {
            let (fun, rest1) = parse_cells(&cells[1..]);
            let (arg, rest2) = parse_cells(&rest1);
            (format!("((force {}) {})", fun, arg), rest2)
        }
    }
}

fn transpile_statement(s: &Statement) -> String {
    let (code, rest) = parse_cells(&s.cells);
    if !rest.is_empty() {
        panic!(
            "rest is not empty: input = {:?}, code = {:?}, rest = {:?}",
            s, code, rest
        )
    }
    code
}

fn main() {
    let stack_size = 1024 * 1024 * 1024;
    let handler = thread::Builder::new().name("transpiler".to_owned()).stack_size(stack_size).spawn(move || {
        let statements = load();
        for (id, statement) in statements {
            println!("statement length = {}", statement.cells.len());
            println!("(define z{} {})", id, transpile_statement(&statement));
        }
    }).unwrap();
    handler.join().unwrap();
}

#[test]
fn test_transpile_statement() {
    let cases = vec![
        (vec![Function::Number(42)], "42"),
        (
            vec![Function::Ap, Function::Neg, Function::Number(42)],
            "((force _neg) 42)",
        ),
        (
            vec![
                Function::Ap,
                Function::Ap,
                Function::Add,
                Function::Number(1),
                Function::Number(1),
            ],
            "((force ((force _add) 1)) 1)",
        ),
        (
            vec![
                Function::Ap,
                Function::Ap,
                Function::Cons,
                Function::Number(1),
                Function::Ap,
                Function::Ap,
                Function::Cons,
                Function::Number(2),
                Function::Nil,
            ],
            "((force ((force _cons) 1)) ((force ((force _cons) 2)) _nil))",
        ),
        (vec![Function::Variable(42)], "(delay z42)"),
    ];

    for (cells, expected) in cases {
        let actual = transpile_statement(&Statement { id: 0, cells });
        assert_eq!(actual, expected);
    }
}
