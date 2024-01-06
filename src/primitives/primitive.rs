use core::fmt;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    bail_wrong_argument, match_or_bail,
    math::operators::{BinOp, UnOp},
    parser::transformer::TransformError,
    primitives::primitive_traits::ApplyOp,
    wrong_argument,
};

use super::{
    graph::{Graph, GraphEdge, GraphNode},
    iterable::IterableKind,
    tuple::Tuple,
};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "value")]
pub enum Primitive {
    Number(f64),
    String(String),
    //TODO instead of making these, make a recursive IterableKind
    Iterable(IterableKind),
    Graph(Graph),
    GraphEdge(GraphEdge),
    GraphNode(GraphNode),
    Tuple(Tuple),
    Boolean(bool),
    Undefined,
}
#[wasm_bindgen(typescript_custom_section)]
const IPrimitive: &'static str = r#"
export type SerializedPrimitive = 
    | { kind: 'Number', value: number }
    | { kind: 'String', value: string }
    | { kind: 'Iterable', value: SerializedIterable }
    | { kind: 'Graph', value: SerializedGraph }
    | { kind: 'GraphEdge', value: SerializedGraphEdge }
    | { kind: 'GraphNode', value: SerializedGraphNode }
    | { kind: 'Tuple', value: SerializedTuple }
    | { kind: 'Boolean', value: boolean }
    | { kind: 'Undefined' }
"#;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PrimitiveKind {
    Number,
    String,
    Iterable(Box<PrimitiveKind>),
    Graph,
    GraphEdge,
    GraphNode,
    Tuple(Vec<PrimitiveKind>),
    Boolean,
    Undefined,
    Any,
}
#[wasm_bindgen(typescript_custom_section)]
const IPrimitiveKind: &'static str = r#"
export type SerializedPrimitiveKind = 
    | { kind: 'Number' }
    | { kind: 'String' }
    | { kind: 'Iterable', value: SerializedPrimitiveKind[] }
    | { kind: 'Graph' }
    | { kind: 'GraphEdge' }
    | { kind: 'GraphNode' }
    | { kind: 'Tuple', value: SerializedPrimitiveKind[] }
    | { kind: 'Boolean' }
    | { kind: 'Undefined' }
    | { kind: 'Any' }
"#;
impl PrimitiveKind {
    pub fn from_primitive(p: &Primitive) -> Self {
        match p {
            Primitive::Number(_) => PrimitiveKind::Number,
            Primitive::String(_) => PrimitiveKind::String,
            Primitive::Iterable(p) => p.get_type(),
            Primitive::Graph(_) => PrimitiveKind::Graph,
            Primitive::GraphEdge(_) => PrimitiveKind::GraphEdge,
            Primitive::GraphNode(_) => PrimitiveKind::GraphNode,
            Primitive::Tuple(t) => t.get_type(),
            Primitive::Boolean(_) => PrimitiveKind::Boolean,
            Primitive::Undefined => PrimitiveKind::Undefined,
        }
    }
    pub fn can_apply_binary_op(&self, op: BinOp, to: PrimitiveKind) -> bool {
        match self {
            PrimitiveKind::Any => false,
            PrimitiveKind::Undefined => false,
            PrimitiveKind::Number => f64::can_apply_binary_op(op, to),
            PrimitiveKind::Boolean => bool::can_apply_binary_op(op, to),
            PrimitiveKind::Graph => Graph::can_apply_binary_op(op, to),
            PrimitiveKind::GraphEdge => GraphEdge::can_apply_binary_op(op, to),
            PrimitiveKind::GraphNode => GraphNode::can_apply_binary_op(op, to),
            PrimitiveKind::Tuple(_) => Tuple::can_apply_binary_op(op, to),
            PrimitiveKind::Iterable(_) => IterableKind::can_apply_binary_op(op, to),
            PrimitiveKind::String => String::can_apply_binary_op(op, to),
        }
    }
    pub fn can_apply_unary_op(&self, op: UnOp) -> bool {
        match self {
            PrimitiveKind::Any => false,
            PrimitiveKind::Undefined => false,
            PrimitiveKind::Number => f64::can_apply_unary_op(op),
            PrimitiveKind::Boolean => bool::can_apply_unary_op(op),
            PrimitiveKind::Graph => Graph::can_apply_unary_op(op),
            PrimitiveKind::GraphEdge => GraphEdge::can_apply_unary_op(op),
            PrimitiveKind::GraphNode => GraphNode::can_apply_unary_op(op),
            PrimitiveKind::Tuple(_) => Tuple::can_apply_unary_op(op),
            PrimitiveKind::Iterable(_) => IterableKind::can_apply_unary_op(op),
            PrimitiveKind::String => String::can_apply_unary_op(op),
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            PrimitiveKind::Number => "Number".to_string(),
            PrimitiveKind::String => "String".to_string(),
            PrimitiveKind::Iterable(i) => format!("{}[]", i.to_string()),
            PrimitiveKind::Graph => "Graph".to_string(),
            PrimitiveKind::GraphEdge => "GraphEdge".to_string(),
            PrimitiveKind::GraphNode => "GraphNode".to_string(),
            PrimitiveKind::Tuple(t) => format!(
                "({})",
                t.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            PrimitiveKind::Boolean => "Boolean".to_string(),
            PrimitiveKind::Undefined => "Undefined".to_string(),
            PrimitiveKind::Any => "Any".to_string(),
        }
    }
}

impl Primitive {
    pub fn get_type(&self) -> PrimitiveKind {
        PrimitiveKind::from_primitive(self)
    }
    pub fn get_type_string(&self) -> String {
        self.get_type().to_string()
    }
    pub fn as_number(&self) -> Result<f64, TransformError> {
        match_or_bail!("Number", Primitive::Number(n) => Ok(*n) ; (self, self))
    }
    pub fn as_integer(&self) -> Result<i64, TransformError> {
        let n = self.as_number()?;
        if n.fract() != 0.0 {
            bail_wrong_argument!("Integer", self)
        } else {
            Ok(n as i64)
        }
    }
    pub fn as_usize(&self) -> Result<usize, TransformError> {
        let n = self.as_number()?;
        if n.fract() != 0.0 {
            bail_wrong_argument!("Integer", self)
        } else if n < 0.0 {
            bail_wrong_argument!("Positive integer", self)
        } else {
            Ok(n as usize)
        }
    }
    pub fn as_graph(&self) -> Result<&Graph, TransformError> {
        match_or_bail!("Graph", 
            Primitive::Graph(g) => Ok(g)
          ; (self, self))
    }
    pub fn as_iterator(&self) -> Result<&IterableKind, TransformError> {
        match_or_bail!("Iterable", Primitive::Iterable(i) => Ok(i) ; (self, self))
    }
    pub fn as_tuple(&self) -> Result<&Vec<Primitive>, TransformError> {
        match_or_bail!("Tuple", Primitive::Tuple(t) => Ok(t.get_primitives()) ; (self, self))
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Primitive::Number(n) => n.to_string(),
            Primitive::String(s) => s.to_string(),
            Primitive::Iterable(i) => match i {
                IterableKind::Numbers(v) => format!("{:?}", v),
                IterableKind::Strings(v) => format!("{:?}", v),
                IterableKind::Edges(v) => format!("{:?}", v),
                IterableKind::Nodes(v) => format!("{:?}", v),
                IterableKind::Tuple(v) => format!("{:?}", v),
                IterableKind::Booleans(v) => format!("{:?}", v),
                IterableKind::Graphs(v) => format!("{:?}", v),
                IterableKind::Iterable(v) => {
                    let result = v
                        .iter()
                        .map(|i| i.to_string_depth(1))
                        .collect::<Vec<_>>()
                        .join(",\n");
                    format!("[\n{}\n]", result)
                }
            },
            Primitive::Graph(g) => g.to_string(),
            Primitive::GraphEdge(e) => e.to_string(),
            Primitive::GraphNode(n) => n.to_string(),
            Primitive::Tuple(v) => format!("{:?}", v),
            Primitive::Boolean(b) => b.to_string(),
            Primitive::Undefined => "undefined".to_string(),
        };
        f.write_str(&s)
    }
}
