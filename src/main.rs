use term_table::{row::Row, Table, table_cell::TableCell};

use rooc::{
    math::math_enums::{Comparison, OptimizationType},
    solvers::{
        linear_problem::{Constraint, LinearProblem},
        simplex::{IntoCanonicalTableau, Tableau},
    }, RoocParser,
};

#[allow(unused)]
fn main() {
    let mut tableau = Tableau::new(
        vec![-3.0, -4.0, -7.0, 0.0, 0.0],
        vec![vec![1.0, 3.0, 4.0, 1.0, 0.0], vec![2.0, 1.0, 3.0, 0.0, 1.0]],
        vec![1.0, 2.0],
        vec![3, 4],
        0.0,
        0.0,
        create_variable_names(5),
    );
    let mut tableau = Tableau::new(
        vec![3.0, 4.0, 6.0],
        vec![vec![0.0, 1.0, 1.0], vec![1.0, -1.0, 0.0]],
        vec![0.0, 1.0],
        vec![2, 0],
        0.0,
        0.0,
        create_variable_names(3),
    );
    let mut tableau = Tableau::new(
        vec![-3.0, -4.0, -7.0, 0.0, 0.0],
        vec![vec![1.0, 3.0, 4.0, 1.0, 0.0], vec![2.0, 1.0, 3.0, 0.0, 1.0]],
        vec![1.0, 2.0],
        vec![3, 4],
        0.0,
        0.0,
        create_variable_names(5),
    );
    let mut tableau = Tableau::new(
        vec![-4.0, -0.25, -0.25, -0.25, 0.0, 0.0, 0.0],
        vec![
            vec![2.0, 5.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            vec![3.0, 0.0, 10.0, 0.0, 0.0, 1.0, 0.0],
            vec![12.0, 0.0, 0.0, 25.0, 0.0, 0.0, 1.0],
        ],
        vec![75.0, 250.0, 500.0],
        vec![4, 5, 6],
        0.0,
        0.0,
        create_variable_names(7),
    );

    let linear_problem = LinearProblem::new(
        vec![3.0, 4.0, 6.0],
        OptimizationType::Min,
        0.0,
        vec![
            Constraint::new(vec![1.0, 3.0, 4.0], Comparison::Equal, 1.0),
            Constraint::new(vec![2.0, 1.0, 3.0], Comparison::Equal, 2.0),
        ],
        create_variable_names(3),
    );
    let standard_problem = linear_problem.into_standard_form().unwrap();
    let mut tableau = standard_problem.into_canonical().unwrap();

    let result = tableau.solve(1000);
    match result {
        Ok(optimal_tableau) => {
            let pretty = tableau.to_fractional_tableau();
            let table = pretty.pretty_table();
            let mut cli_table = Table::new();
            let values = optimal_tableau.get_variables_values().clone();
            let mut header = Row::new(values.iter().map(TableCell::new));
            header.cells.push(TableCell::new(
                optimal_tableau.get_tableau().get_current_value(),
            ));
            cli_table.add_row(header);
            let empty: Vec<TableCell> = Vec::new();
            cli_table.add_row(Row::new(empty));
            table.iter().for_each(|row| {
                cli_table.add_row(Row::new(row.iter().map(TableCell::new)));
            });
            println!("{}", cli_table.render());
            println!("Optimal value: {}", optimal_tableau.get_optimal_value());
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    let source = "
    min 1
    s.t.
        sum(i in 0..5) { i } <= 1 for j in enumerate(A)
        where 
            G = Graph {
                A -> [B],
                B
            }
            A = [1,2]

    "
    .trim()
    .to_string();
    let parser = RoocParser::new(source);
    let parsed = parser.parse();
    match parsed {
        Ok(parsed) => {
            println!("{:?}", parsed.create_type_checker());
        }
        Err(e) => {
            println!("{}", e.to_error_string());
        }
    }
}

fn create_variable_names(n: usize) -> Vec<String> {
    let mut variables = Vec::new();
    for i in 0..n {
        variables.push(format!("x{}", i));
    }
    variables
}
