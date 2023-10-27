use rooc::{
    consts::{Comparison, OptimizationType},
    linear_problem::{Constraint, LinearProblem},
    parser::parse,
    simplex::{IntoCanonicalTableau, Tableau}, transformer::{transform},
};
use term_table::{row::Row, table_cell::TableCell, Table};

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

    let problem = "
    max sum(i in 0..len(C), j in 0..len(b)){  X_ij * C[i]}
    s.t.
        len(C) * sum(i in 0..len(C)){ C[i] * X_ij } <= b[j] for j in 0..len(C)
    where
        C = [15, 30]
        b = [20, 25]
    "
    .to_string();
    let parsed = parse(&problem);
    match parsed {
        Ok(parsed) => {
            println!("{:#?}", parsed);
            let transformed = transform(&parsed);
            println!("\n\n");
            match transformed {
                Ok(transformed) => println!("{}", transformed.to_string()),
                Err(e) => println!("Error: {:#?}", e),
            }
        }
        Err(e) => {
            println!("Error: {}", e);
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
