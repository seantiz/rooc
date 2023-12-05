use std::collections::HashMap;

use crate::{
    math_enums::{Comparison, Op, OptimizationType},
    primitives::primitive::Primitive,
    utils::{InputSpan, Spanned},
};

use super::{
    parser::PreProblem,
    pre_parsed_problem::{ArrayAccess, PreCondition, PreExp, PreObjective},
    recursive_set_resolver::recursive_set_resolver,
};

#[derive(Debug, Clone)]
pub enum Exp {
    Number(f64),
    Variable(String),
    Mod(Box<Exp>),
    Min(Vec<Exp>),
    Max(Vec<Exp>),
    BinOp(Op, Box<Exp>, Box<Exp>),
    Neg(Box<Exp>),
}

impl Exp {
    pub fn make_binop(op: Op, lhs: Exp, rhs: Exp) -> Box<Self> {
        Exp::BinOp(op, lhs.to_box(), rhs.to_box()).to_box()
    }

    pub fn to_box(self) -> Box<Exp> {
        Box::new(self)
    }
    pub fn from_pre_exp(
        pre_exp: &PreExp,
        context: &mut TransformerContext,
    ) -> Result<Self, TransformError> {
        pre_exp.into_exp(context)
    }

    pub fn simplify(&self) -> Exp {
        todo!("implement the simplify function by using e-graphs egg")
    }

    pub fn flatten(self) -> Exp {
        match self {
            Exp::BinOp(op, lhs, rhs) => match (op, *lhs, *rhs) {
                //(a +- b)c = ac +- bc
                (Op::Mul, Exp::BinOp(inner_op @ (Op::Add | Op::Sub), lhs, rhs), c) => Exp::BinOp(
                    inner_op,
                    Exp::make_binop(Op::Mul, *lhs, c.clone()),
                    Exp::make_binop(Op::Mul, *rhs, c),
                )
                .flatten(),
                //c(a +- b) = ac +- bc
                (Op::Mul, c, Exp::BinOp(inner_op @ (Op::Add | Op::Sub), lhs, rhs)) => Exp::BinOp(
                    inner_op,
                    Exp::make_binop(Op::Mul, c.clone(), *lhs),
                    Exp::make_binop(Op::Mul, c, *rhs),
                )
                .flatten(),
                //-(a)b = -ab
                (Op::Mul, Exp::Neg(lhs), c) => {
                    Exp::Neg(Exp::make_binop(Op::Mul, *lhs, c).flatten().to_box())
                }
                //a(-b) = -ab
                (Op::Mul, c, Exp::Neg(rhs)) => {
                    Exp::Neg(Exp::make_binop(Op::Mul, c, *rhs).flatten().to_box())
                }
                //(a +- b)/c = a/c +- b/c
                (Op::Div, Exp::BinOp(inner_op @ (Op::Add | Op::Sub), lhs, rhs), c) => Exp::BinOp(
                    inner_op,
                    Exp::make_binop(Op::Div, *lhs, c.clone()),
                    Exp::make_binop(Op::Div, *rhs, c),
                ),

                (op, lhs, rhs) => Exp::BinOp(op, lhs.flatten().to_box(), rhs.flatten().to_box()),
            },
            _ => self,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Exp::Number(value) => value.to_string(),
            Exp::Variable(name) => name.clone(),
            Exp::Mod(exp) => format!("|{}|", exp.to_string()),
            Exp::Min(exps) => format!(
                "min{{ {} }}",
                exps.iter()
                    .map(|exp| exp.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Exp::Max(exps) => format!(
                "max{{ {} }}",
                exps.iter()
                    .map(|exp| exp.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Exp::BinOp(operator, lhs, rhs) => {
                //TODO: add parenthesis when needed
                format!(
                    "{} {} {}",
                    lhs.to_string(),
                    operator.to_string(),
                    rhs.to_string()
                )
            }
            Exp::Neg(exp) => {
                if exp.is_leaf() {
                    format!("-{}", exp.to_string())
                } else {
                    format!("-({})", exp.to_string())
                }
            }
        }
    }
    pub fn is_leaf(&self) -> bool {
        match self {
            Exp::BinOp(_, _, _) => false,
            Exp::Neg(_) => false,
            _ => true,
        }
    }
}

#[derive(Debug)]
pub struct Objective {
    objective_type: OptimizationType,
    rhs: Exp,
}

impl Objective {
    pub fn new(objective_type: OptimizationType, rhs: Exp) -> Self {
        Self {
            objective_type,
            rhs,
        }
    }
    pub fn to_string(&self) -> String {
        format!(
            "{} {}",
            self.objective_type.to_string(),
            self.rhs.to_string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    lhs: Exp,
    condition_type: Comparison,
    rhs: Exp,
}

impl Condition {
    pub fn new(lhs: Exp, condition_type: Comparison, rhs: Exp) -> Self {
        Self {
            lhs: lhs,
            condition_type,
            rhs: rhs,
        }
    }
    pub fn to_string(&self) -> String {
        format!(
            "{} {} {}",
            self.lhs.to_string(),
            self.condition_type.to_string(),
            self.rhs.to_string()
        )
    }
}

#[derive(Debug)]
pub struct Problem {
    objective: Objective,
    conditions: Vec<Condition>,
}

impl Problem {
    pub fn new(objective: Objective, conditions: Vec<Condition>) -> Self {
        Self {
            objective,
            conditions,
        }
    }
    pub fn to_string(&self) -> String {
        let conditions = self
            .conditions
            .iter()
            .map(|condition| condition.to_string())
            .collect::<Vec<_>>()
            .join("\n\t");
        format!("{}\ns.t\n\t{}", self.objective.to_string(), conditions)
    }
}

#[derive(Debug, Clone)]
pub enum TransformError {
    MissingVariable(String),
    AlreadyExistingVariable(String),
    OutOfBounds(String),
    WrongArgument(String),
    SpannedError(Spanned<Box<TransformError>>, Option<String>),
    Other(String),
}
impl TransformError {
    pub fn to_string(&self) -> String {
        match self {
            TransformError::MissingVariable(name) => format!("[Missing variable] {}", name),
            TransformError::AlreadyExistingVariable(name) => {
                format!("Variable {} was already declared", name)
            }
            TransformError::OutOfBounds(name) => format!("[Out of bounds] {}", name),
            TransformError::WrongArgument(name) => format!("[Wrong argument] {}", name),
            TransformError::Other(name) => name.clone(),
            TransformError::SpannedError(error, _) => return error.to_string(),
        }
    }
    pub fn get_traced_error(&self) -> String {
        let error = self.to_string();
        let trace = self.get_trace();
        let trace = trace
            .iter()
            .map(|(span, origin)| {
                let origin = if let Some(o) = origin {
                    format!(" ({})", o)
                } else {
                    "".to_string()
                };
                format!("\tat {}:{}{}", span.start_line, span.start_column, origin)
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("{}\n{}", error, trace)
    }
    pub fn to_spanned_error(self, span: &InputSpan) -> TransformError {
        TransformError::SpannedError(Spanned::new(Box::new(self), span.clone()), None)
    }
    pub fn get_trace(&self) -> Vec<(InputSpan, Option<String>)> {
        match self {
            TransformError::SpannedError(e, s) => {
                let mut trace = vec![(e.get_span().clone(), s.clone())];
                let mut last_error = e;
                while let TransformError::SpannedError(ref e, ref r) = **last_error.get_span_value()
                {
                    let span = e.get_span().clone();
                    //don't add if the last span is the same as the current one
                    if let Some((last_span, _)) = trace.last() {
                        if last_span == &span {
                            last_error = e;
                            continue;
                        }
                    }
                    trace.push((span, r.clone()));
                    last_error = e;
                }
                trace.reverse();
                trace
            }
            _ => Vec::new(),
        }
    }
    pub fn get_trace_from_source(&self, source: &str) -> Result<String, ()> {
        let trace = self.get_trace();
        let trace = trace
            .into_iter()
            .map(|(span, _)| {
                let text = span.get_span_text(source)?;
                Ok(format!(
                    "at {}:{} {}",
                    span.start_line, span.start_column, text,
                ))
            })
            .collect::<Result<Vec<_>, ()>>()?;
        let join = trace.join("\n\t");
        Ok(format!("{}\n\t{}", self.to_string(), join))
    }
}

#[derive(Debug)]
pub struct Frame {
    pub variables: HashMap<String, Primitive>,
}
impl Frame {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
    pub fn from_primitives(constants: HashMap<String, Primitive>) -> Self {
        Self {
            variables: constants,
        }
    }

    pub fn get_value(&self, name: &str) -> Option<&Primitive> {
        self.variables.get(name)
    }
    pub fn declare_variable(&mut self, name: &str, value: Primitive) -> Result<(), TransformError> {
        if self.has_variable(name) {
            return Err(TransformError::AlreadyExistingVariable(name.to_string()));
        }
        self.variables.insert(name.to_string(), value);
        Ok(())
    }
    pub fn update_variable(&mut self, name: &str, value: Primitive) -> Result<(), TransformError> {
        if !self.has_variable(name) {
            return Err(TransformError::MissingVariable(name.to_string()));
        }
        self.variables.insert(name.to_string(), value);
        Ok(())
    }
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }
    pub fn drop_variable(&mut self, name: &str) -> Result<Primitive, TransformError> {
        if !self.variables.contains_key(name) {
            return Err(TransformError::MissingVariable(name.to_string()));
        }
        let value = self.variables.remove(name).unwrap();
        Ok(value)
    }
}

#[derive(Debug)]
pub struct TransformerContext {
    frames: Vec<Frame>,
}
impl TransformerContext {
    pub fn new(primitives: HashMap<String, Primitive>) -> Self {
        let frame = Frame::from_primitives(primitives);
        Self {
            frames: vec![frame],
        }
    }

    pub fn flatten_variable_name(
        &self,
        compound_name: &Vec<String>,
    ) -> Result<String, TransformError> {
        let flattened = compound_name
            .iter()
            .map(|name| match self.get_value(name) {
                Some(value) => match value {
                    Primitive::Number(value) => Ok(value.to_string()),
                    Primitive::String(value) => Ok(value.clone()),
                    Primitive::GraphNode(v) => Ok(v.get_name().clone()),
                    _ => Err(TransformError::WrongArgument(format!(
                        "Expected a number or string for variable flattening, got {}, check the definition of {}",
                        value.get_type().to_string(),
                        name
                    ))),
                },
                None => Err(TransformError::MissingVariable(name.to_string())),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(flattened.join("_"))
    }

    pub fn add_populated_scope(&mut self, frame: Frame) {
        self.frames.push(frame);
    }
    pub fn replace_last_frame(&mut self, frame: Frame) {
        self.frames.pop();
        self.frames.push(frame);
    }
    pub fn add_scope(&mut self) {
        let frame = Frame::new();
        self.frames.push(frame);
    }
    pub fn pop_scope(&mut self) -> Result<Frame, TransformError> {
        if self.frames.len() <= 1 {
            return Err(TransformError::Other("Missing frame to pop".to_string()));
        }
        Ok(self.frames.pop().unwrap())
    }
    pub fn get_value(&self, name: &str) -> Option<&Primitive> {
        for frame in self.frames.iter().rev() {
            match frame.get_value(name) {
                Some(value) => return Some(value),
                None => continue,
            }
        }
        None
    }
    pub fn exists_variable(&self, name: &str, strict: bool) -> bool {
        if strict {
            for frame in self.frames.iter().rev() {
                if frame.has_variable(name) {
                    return true;
                }
            }
        } else {
            return match self.frames.last() {
                Some(frame) => frame.has_variable(name),
                None => false,
            };
        }
        false
    }
    pub fn declare_variable(
        &mut self,
        name: &str,
        value: Primitive,
        strict: bool,
    ) -> Result<(), TransformError> {
        if name == "_" {
            return Ok(());
        }
        if strict {
            if self.get_value(name).is_some() {
                return Err(TransformError::AlreadyExistingVariable(name.to_string()));
            }
        }
        let frame = self.frames.last_mut().unwrap();
        frame.declare_variable(name, value)
    }
    pub fn update_variable(&mut self, name: &str, value: Primitive) -> Result<(), TransformError> {
        if name == "_" {
            return Ok(());
        }
        for frame in self.frames.iter_mut().rev() {
            if frame.has_variable(name) {
                return frame.update_variable(name, value);
            }
        }
        Err(TransformError::MissingVariable(name.to_string()))
    }
    pub fn remove_variable(&mut self, name: &str) -> Result<Primitive, TransformError> {
        if name == "_" {
            return Ok(Primitive::Undefined);
        }
        for frame in self.frames.iter_mut().rev() {
            if frame.has_variable(name) {
                return frame.drop_variable(name);
            }
        }
        Err(TransformError::MissingVariable(name.to_string()))
    }

    pub fn flatten_compound_variable(
        &self,
        name: &String,
        indexes: &Vec<String>,
    ) -> Result<String, TransformError> {
        let names: String = self.flatten_variable_name(indexes)?;
        let name = format!("{}_{}", name, names);
        Ok(name)
    }

    pub fn get_numerical_constant(&self, name: &str) -> Result<f64, TransformError> {
        match self.get_value(name) {
            Some(v) => v.as_number(),
            None => Err(TransformError::MissingVariable(name.to_string())),
        }
    }
    pub fn get_array_value(&self, array_access: &ArrayAccess) -> Result<Primitive, TransformError> {
        match self.get_value(&array_access.name) {
            Some(a) => {
                let accesses = array_access
                    .accesses
                    .iter()
                    .map(|access| access.as_usize(self))
                    .collect::<Result<Vec<_>, TransformError>>()?;
                let value = a.as_iterator()?.read(accesses)?;
                Ok(value)
            }
            None => Err(TransformError::MissingVariable(
                array_access.name.to_string(),
            )),
        }
    }
}

pub fn transform_parsed_problem(pre_problem: &PreProblem) -> Result<Problem, TransformError> {
    let constants = pre_problem
        .constants
        .iter()
        .map(|c| (c.name.clone(), c.value.clone()))
        .collect::<Vec<_>>();
    let constants = HashMap::from_iter(constants);
    let mut context = TransformerContext::new(constants);
    transform_problem(pre_problem, &mut context)
}

/*
this function gets a set, defined by a number of variables with a certain name, and an iterator,
it should return a vector of vectors, where each vector is a set of values for the variables
ex:
checks that the iterator has at least the same number of elements as the set, and then returns the values in the iterator
    in:  set {i, j} and iterator [[0, 0], [1, 1]]
    out: [[0, 0], [1, 1]]
    in:  set {i} and iterator [[0, 0], [1, 1]]
    out: [[0], [1]]
    in:  set {i, j, k} and iterator [[0, 0], [1, 1]]
    out: error!
*/

pub type PrimitiveSet = Vec<Primitive>;

#[derive(Debug, Clone)]
pub enum VariableType {
    Single(Spanned<String>),
    Tuple(Vec<Spanned<String>>),
}

pub fn transform_condition(
    condition: &PreCondition,
    context: &mut TransformerContext,
) -> Result<Condition, TransformError> {
    let lhs = condition.lhs.into_exp(context)?;
    let rhs = condition.rhs.into_exp(context)?;
    Ok(Condition::new(lhs, condition.condition_type.clone(), rhs))
}

pub fn transform_condition_with_iteration(
    condition: &PreCondition,
    context: &mut TransformerContext,
) -> Result<Vec<Condition>, TransformError> {
    if condition.iteration.is_empty() {
        return Ok(vec![transform_condition(condition, context)?]);
    }
    let mut results: Vec<Condition> = Vec::new();
    recursive_set_resolver(&condition.iteration, context, &mut results, 0, &|c| {
        transform_condition(condition, c)
    })
    .map_err(|e| e.to_spanned_error(&condition.span))?;
    Ok(results)
}

pub fn transform_objective(
    objective: &PreObjective,
    context: &mut TransformerContext,
) -> Result<Objective, TransformError> {
    let rhs = objective.rhs.into_exp(context)?;
    Ok(Objective::new(objective.objective_type.clone(), rhs))
}

pub fn transform_problem(
    problem: &PreProblem,
    context: &mut TransformerContext,
) -> Result<Problem, TransformError> {
    let objective = transform_objective(&problem.objective, context)?;
    let mut conditions: Vec<Condition> = Vec::new();
    for condition in problem.conditions.iter() {
        let transformed = transform_condition_with_iteration(condition, context)?;
        for condition in transformed {
            conditions.push(condition);
        }
    }
    Ok(Problem::new(objective, conditions))
}