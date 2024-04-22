use std::fmt::Display;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    math::math_enums::{Comparison, OptimizationType},
    transformers::standardizer::to_standard_form,
};
use crate::transformers::standard_linear_model::{format_var, StandardLinearModel};

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct LinearConstraint {
    coefficients: Vec<f64>,
    rhs: f64,
    constraint_type: Comparison,
}

impl LinearConstraint {
    pub fn new(coefficients: Vec<f64>, constraint_type: Comparison, rhs: f64) -> LinearConstraint {
        LinearConstraint {
            coefficients,
            rhs,
            constraint_type,
        }
    }
    pub fn get_coefficients(&self) -> &Vec<f64> {
        &self.coefficients
    }
    pub fn get_rhs(&self) -> f64 {
        self.rhs
    }
    pub fn get_constraint_type(&self) -> &Comparison {
        &self.constraint_type
    }
    pub fn into_parts(self) -> (Vec<f64>, Comparison, f64) {
        (self.coefficients, self.constraint_type, self.rhs)
    }
    pub fn ensure_size(&mut self, size: usize) {
        self.coefficients.resize(size, 0.0);
    }
}

#[wasm_bindgen]
impl LinearConstraint {
    pub fn wasm_get_coefficients(&self) -> Vec<f64> {
        self.coefficients.clone()
    }
    pub fn wasm_get_rhs(&self) -> f64 {
        self.rhs
    }
    pub fn wasm_get_constraint_type(&self) -> Comparison {
        self.constraint_type
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct LinearModel {
    variables: Vec<String>,
    objective_offset: f64,
    optimization_type: OptimizationType,
    objective: Vec<f64>,
    constraints: Vec<LinearConstraint>,
}

impl LinearModel {
    pub fn new(
        objective: Vec<f64>,
        optimization_type: OptimizationType,
        objective_offset: f64,
        constraints: Vec<LinearConstraint>,
        variables: Vec<String>,
    ) -> LinearModel {
        LinearModel {
            objective,
            constraints,
            optimization_type,
            variables,
            objective_offset,
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        Vec<f64>,
        OptimizationType,
        f64,
        Vec<LinearConstraint>,
        Vec<String>,
    ) {
        (
            self.objective,
            self.optimization_type,
            self.objective_offset,
            self.constraints,
            self.variables,
        )
    }
    pub fn get_optimization_type(&self) -> &OptimizationType {
        &self.optimization_type
    }
    pub fn into_standard_form(self) -> Result<StandardLinearModel, ()> {
        to_standard_form(self)
    }
    pub fn get_objective(&self) -> &Vec<f64> {
        &self.objective
    }
    pub fn get_constraints(&self) -> &Vec<LinearConstraint> {
        &self.constraints
    }
    pub fn get_variables(&self) -> &Vec<String> {
        &self.variables
    }
    pub fn get_objective_offset(&self) -> f64 {
        self.objective_offset
    }
}

impl Display for LinearModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let constraints = self.constraints.iter().map(|c| {
            let mut is_first = true;
            let coefficients = c
                .coefficients
                .iter()
                .enumerate()
                .flat_map(|(i, c)| {
                    if *c == 0.0 {
                        None
                    } else {
                        let var = format_var(&self.variables[i], *c, is_first);
                        is_first = false;
                        Some(var)
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");
            format!("    {} {} {}", coefficients, c.constraint_type, c.rhs)
        });

        let constraints = constraints.collect::<Vec<String>>().join("\n");
        let mut is_first = true;
        let objective = self
            .objective
            .iter()
            .enumerate()
            .flat_map(|(i, c)| {
                if *c == 0.0 {
                    None
                } else {
                    let var = format_var(&self.variables[i], *c, is_first);
                    is_first = false;
                    Some(var)
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
        let offset = if self.objective_offset == 0.0 {
            "".to_string()
        } else if self.objective_offset < 0.0 {
            format!(" - {}", self.objective_offset.abs())
        } else {
            format!(" + {}", self.objective_offset)
        };
        let objective = format!("{}{}", objective, self.objective_offset);
        write!(
            f,
            "{} {}\ns.t.\n{}",
            self.optimization_type, objective, constraints
        )
    }
}

#[wasm_bindgen]
impl LinearModel {
    pub fn wasm_get_objective(&self) -> Vec<f64> {
        self.objective.clone()
    }
    pub fn wasm_get_constraints(&self) -> Vec<LinearConstraint> {
        self.constraints.clone()
    }
    pub fn wasm_get_variables(&self) -> Vec<String> {
        self.variables.clone()
    }
    pub fn wasm_get_objective_offset(&self) -> f64 {
        self.objective_offset
    }
    pub fn wasm_get_optimization_type(&self) -> OptimizationType {
        self.optimization_type.clone()
    }

    pub fn wasm_to_string(&self) -> String {
        format!("{}", self)
    }
}