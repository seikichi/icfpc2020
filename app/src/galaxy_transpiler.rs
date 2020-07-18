mod galaxy_interpreter;

use galaxy_interpreter::{load, Function, Statement};

fn parse_cells(cells: &[Function]) -> (String, &[Function]) {
    match cells[0] {
        Function::Number(n) => (n.to_string(), &cells[1..]),
        Function::Neg => ("_neg".to_string(), &cells[1..]),
        Function::Ap => {
            let (fun, rest1) = parse_cells(&cells[1..]);
            let (arg, rest2) = parse_cells(&rest1);
            (format!("((force {}) {})", fun, arg), rest2)
        }
        _ => (String::from(""), &[]),
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
    let statements = load();
    println!("{:?}", statements);
}

#[test]
fn test_transpile_statement() {
    let cases = vec![
        (vec![Function::Number(42)], "42"),
        (
            vec![Function::Ap, Function::Neg, Function::Number(42)],
            "((force _neg) 42)",
        ),
    ];

    for (cells, expected) in cases {
        let actual = transpile_statement(&Statement { id: 0, cells });
        assert_eq!(actual, expected);
    }
}
