use core::panic;
use std::{fmt::{format, Debug}, collections::HashMap};

use crate::{
    bail_wrong_argument, match_or_bail,
    parser::{PreArrayAccess, CompoundVariable},
    transformer::{ TransformError, TransformerContext},
};
use pest::Span;

#[derive(Debug, Clone)]
pub struct GraphEdge {
    from: String,
    to: String,
    weight: Option<f64>,
}
impl GraphEdge {
    pub fn new(from: String, to: String, weight: Option<f64>) -> Self {
        Self { from, to, weight }
    }
    pub fn to_string(&self) -> String {
        match self.weight {
            Some(w) => format!("{}:{}", self.to, w),
            None => self.to.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    name: String,
    edges: HashMap<String, GraphEdge>,
}
impl GraphNode {
    pub fn new(name: String, edges: Vec<GraphEdge>) -> Self {
        let edges = edges
            .into_iter()
            .map(|edge| (edge.to.clone(), edge))
            .collect::<HashMap<String, GraphEdge>>();
        Self { name, edges }
    }
    pub fn to_string(&self) -> String {
        let edges = self
            .edges
            .iter()
            .map(|(_, edge)| edge.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}: {{{}}}", self.name, edges)
    }
}

#[derive(Debug, Clone)]
pub struct Graph {
    vertices: Vec<GraphNode>,
}
impl Graph {
    pub fn new(vertices: Vec<GraphNode>) -> Self {
        Self { vertices }
    }
    pub fn edges(&self) -> Vec<GraphEdge> {
        self.vertices
            .iter()
            .map(|node| {
                node.edges
                    .values()
                    .map(|edge| edge.clone())
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
    }
    pub fn vertices(&self) -> &Vec<GraphNode> {
        &self.vertices
    }
    pub fn neighbour_of(&self, node_name: &str) -> Result<Vec<&GraphEdge>, TransformError> {
        let node = self
            .vertices
            .iter()
            .find(|n: &&GraphNode| n.name == node_name);
        match node {
            Some(node) => Ok(node.edges.values().collect()),
            None => {
                return Err(TransformError::NotFound(format!(
                    "node {} not found in graph",
                    node_name
                )))
            }
        }
    }

    pub fn to_string(&self) -> String {
        let nodes = self
            .vertices
            .iter()
            .map(|node| node.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        format!("[{}]", nodes)
    }
}


pub trait FunctionCall:Debug {
    fn from_parameters(pars: Vec<Parameter>, span: &Span) -> Result<Self, CompilationError>
    where
        Self: Sized;
    fn call<'a>(&self, context: &'a TransformerContext) -> Result<Primitive<'a>, TransformError>;
    fn to_string(&self) -> String;
}


#[derive(Debug)]
pub enum IteratorKind<'a> {
    Numbers(&'a Vec<f64>),
    Strings(&'a Vec<&'a str>),
    Edges(Vec<GraphEdge>),
    Nodes(&'a Vec<&'a GraphNode>),
}

#[derive(Debug)]
pub enum Primitive<'a> {
    Number(f64),
    String(String),
    NumberArray(&'a Vec<f64>),
    NumberMatrix(&'a Vec<Vec<f64>>),
    Iterator(IteratorKind<'a>),
    Graph(&'a Graph),
    GraphEdge(&'a GraphEdge),
    GraphNode(&'a GraphNode),
    Tuple(Vec<Primitive<'a>>),
}
#[derive(Debug)]
pub enum Parameter {
    Number(f64),
    String(String),
    Variable(String),
    CompoundVariable(CompoundVariable),
    ArrayAccess(PreArrayAccess),
    FunctionCall(Box<dyn FunctionCall>),
}

impl Parameter {
    pub fn as_primitive<'a>(&self, context: &'a TransformerContext) -> Result<Primitive<'a>, TransformError> {
        match self {
            Parameter::Number(n) => Ok(Primitive::Number(*n)),
            Parameter::String(s) => Ok(Primitive::String(s.clone())),
            Parameter::Variable(s) => {
                let value = context.get_primitive(s)?;
                Ok(value)
            }
            Parameter::CompoundVariable(c) => {
                let name = context.flatten_compound_variable(&c.name, &c.indexes)?;
                let value = context.get_primitive(&name)?;
                Ok(value)
            }
            Parameter::FunctionCall(f) => {
                let value = f.call(context)?;
                Ok(value)
            }
            Parameter::ArrayAccess(a)
        }
    }
    //TODO make this a macro
    pub fn as_number<'a>(&self, context: &'a TransformerContext) -> Result<f64, TransformError> {
        match self {
            Parameter::Number(n) => Ok(*n),
            Parameter::Variable(s) => {
                let value = context.get_primitive(s)?;
                match_or_bail!("number", Primitive::Number(n) => Ok(n) ; (value, self))
            }
            Parameter::CompoundVariable(c) => {
                let name = context.flatten_compound_variable(&c.name, &c.indexes)?;
                let value = context.get_primitive(&name)?;
                match_or_bail!("number", Primitive::Number(n) => Ok(n) ; (value, self))
            }
            Parameter::FunctionCall(f) => {
                let value = f.call(context)?;
                match_or_bail!("number", Primitive::Number(n) => Ok(n) ; (value, self))
            }
            _ => bail_wrong_argument!("number", self),
        }
    }
    pub fn as_graph<'a>(
        &self,
        context: &'a TransformerContext,
    ) -> Result<&'a Graph, TransformError> {
        match self {
            Parameter::Variable(s) => {
                let value = context.get_primitive(s)?;
                match_or_bail!("graph", Primitive::Graph(g) => Ok(g) ; (value, self))
            }
            Parameter::CompoundVariable(c) => {
                let name = context.flatten_compound_variable(&c.name, &c.indexes)?;
                let value = context.get_primitive(&name)?;
                match_or_bail!("graph", Primitive::Graph(g) => Ok(g) ; (value, self))
            }
            Parameter::FunctionCall(f) => {
                let value = f.call(context)?;
                match_or_bail!("graph", Primitive::Graph(g) => Ok(g) ; (value, self))
            }
            _ => bail_wrong_argument!("graph", self),
        }
    }
    pub fn as_number_array<'a>(
        &self,
        context: &'a TransformerContext,
    ) -> Result<&'a Vec<f64>, TransformError> {
        match self {
            Parameter::Variable(s) => {
                let value = context.get_primitive(s)?;
                match_or_bail!("array1d", Primitive::NumberArray(a) => Ok(a) ; (value, self))
            }
            Parameter::CompoundVariable(c) => {
                let name = context.flatten_compound_variable(&c.name, &c.indexes)?;

                let value = context.get_primitive(&name)?;
                match_or_bail!("array1d", Primitive::NumberArray(a) => Ok(a) ; (value, self))
            }
            Parameter::FunctionCall(f) => {
                let value = f.call(context)?;
                match_or_bail!("array1d", Primitive::NumberArray(a) => Ok(a) ; (value, self))
            }
            _ => bail_wrong_argument!("array1d", self),
        }
    }
    pub fn as_number_matrix<'a>(
        &self,
        context: &'a TransformerContext,
    ) -> Result<&'a Vec<Vec<f64>>, TransformError> {
        match self {
            Parameter::Variable(s) => {
                let value = context.get_primitive(s)?;
                match_or_bail!("array2d", Primitive::NumberMatrix(a) => Ok(a) ; (value, self))
            }
            Parameter::CompoundVariable(c) => {
                let name = context.flatten_compound_variable(&c.name, &c.indexes)?;
                let value = context.get_primitive(&name)?;
                match_or_bail!("array2d", Primitive::NumberMatrix(a) => Ok(a) ; (value, self))
            }
            Parameter::FunctionCall(f) => {
                let value = f.call(context)?;
                match_or_bail!("array2d", Primitive::NumberMatrix(a) => Ok(a) ; (value, self))
            }
            _ => bail_wrong_argument!("array2d", self),
        }
    }
    pub fn as_iterator<'a>(
        &self,
        context: &'a TransformerContext,
    ) -> Result<IteratorKind<'a>, TransformError> {
        match self {
            Parameter::Variable(s) => {
                let value = context.get_primitive(s)?;
                match_or_bail!("array", Primitive::Iterator(a) => Ok(a) ; (value, self))
            }
            Parameter::CompoundVariable(c) => {
                let name = context.flatten_compound_variable(&c.name, &c.indexes)?;

                let value = context.get_primitive(&name)?;
                match_or_bail!("array", Primitive::Iterator(a) => Ok(a) ; (value, self))
            }
            Parameter::FunctionCall(f) => {
                let value = f.call(context)?;
                match_or_bail!("array", Primitive::Iterator(a) => Ok(a) ; (value, self))
            }
            _ => bail_wrong_argument!("array", self),
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Parameter::Number(n) => n.to_string(),
            Parameter::String(s) => s.to_string(),
            Parameter::Variable(s) => s.to_string(),
            Parameter::CompoundVariable(c) => c.to_string(),
            Parameter::ArrayAccess(a) => a.to_string(),
            Parameter::FunctionCall(f) => f.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Comparison {
    LowerOrEqual,
    UpperOrEqual,
    Equal,
}
impl Comparison {
    pub fn to_string(&self) -> String {
        match self {
            Comparison::LowerOrEqual => "<=".to_string(),
            Comparison::UpperOrEqual => ">=".to_string(),
            Comparison::Equal => "=".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}
impl Op {
    pub fn precedence(&self) -> u8 {
        match self {
            Op::Add => 1,
            Op::Sub => 1,
            Op::Mul => 2,
            Op::Div => 2,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Op::Add => "+".to_string(),
            Op::Sub => "-".to_string(),
            Op::Mul => "*".to_string(),
            Op::Div => "/".to_string(),
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum OptimizationType {
    Min,
    Max,
}
impl OptimizationType {
    pub fn to_string(&self) -> String {
        match self {
            OptimizationType::Min => "min".to_string(),
            OptimizationType::Max => "max".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConstantValue {
    Number(f64),
    OneDimArray(Vec<f64>),
    TwoDimArray(Vec<Vec<f64>>),
    Graph(Graph),
    String(String),
}
impl ConstantValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::OneDimArray(v) => format!("{:?}", v),
            Self::TwoDimArray(v) => {
                let result = v.iter().map(|row| format!("{:?}", row)).collect::<Vec<_>>();
                format!("[\n{}\n]", result.join(",\n"))
            }
            Self::Graph(g) => {
                format!("Graph {{\n{}\n}}", g.to_string())
            }
            Self::String(s) => format!("\"{}\"", s),
        }
    }
}

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    pub value: ConstantValue,
}
impl Constant {
    pub fn new(name: String, value: ConstantValue) -> Self {
        Self { name, value }
    }
    pub fn to_string(&self) -> String {
        format!("{} = {}", self.name, self.value.to_string())
    }
}

pub struct CompilationError {
    kind: ParseError,
    start_line: usize,
    start: usize,
    end_line: usize,
    end: usize,
    text: String,
}
impl CompilationError {
    pub fn new(
        kind: ParseError,
        start_line: usize,
        start: usize,
        end_line: usize,
        end: usize,
        text: String,
    ) -> Self {
        Self {
            kind,
            start_line,
            start,
            end_line,
            end,
            text,
        }
    }
    pub fn from_span(kind: ParseError, span: &Span, exclude_string: bool) -> Self {
        let (start_line, start) = span.start_pos().line_col();
        let (end_line, end) = span.end_pos().line_col();
        let text = if exclude_string { "" } else { span.as_str() }.to_string();
        Self::new(kind, start_line, start, end_line, end, text)
    }
    pub fn to_string(&self) -> String {
        format!(
            "Error at line {}:{} to {}:{}\n\t{} {}",
            self.start_line,
            self.start,
            self.end_line,
            self.end,
            self.kind.to_string(),
            self.text
        )
    }
    pub fn to_error_string(&self) -> String {
        format!("{} {}", self.kind.to_string(), self.text)
    }
}
impl std::fmt::Debug for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    MissingToken(String),
    SemanticError(String),
    WrongNumberOfArguments(usize, Vec<String>),
}
impl ParseError {
    pub fn to_string(&self) -> String {
        match self {
            Self::UnexpectedToken(s) => format!("Unexpected token: \"{}\"", s),
            Self::MissingToken(s) => format!("Missing token: \"{}\"", s),
            Self::SemanticError(s) => format!("Semantic error: \"{}\"", s),
            Self::WrongNumberOfArguments(got, expected) => format!(
                "Wrong number of arguments: got {}, expected {}: ({})",
                got,
                expected.len(),
                expected.join(", ")
            ),
        }
    }

}
