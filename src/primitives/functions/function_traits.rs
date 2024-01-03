use dyn_clone::DynClone;
use std::fmt::Debug;

use pest::iterators::Pair;
use crate::{
    parser::{
        parser::Rule,
        pre_parsed_problem::PreExp,
        transformer::{TransformError, TransformerContext},
    },
    primitives::primitive::Primitive,
    utils::CompilationError,
};
use erased_serde::serialize_trait_object;
pub trait FunctionCall: Debug + DynClone + erased_serde::Serialize {
    fn from_parameters(
        pars: Vec<PreExp>,
        origin_rule: &Pair<Rule>,
    ) -> Result<Self, CompilationError>
    where
        Self: Sized;
    fn call(&self, context: &TransformerContext) -> Result<Primitive, TransformError>;
    fn to_string(&self) -> String;
}

serialize_trait_object!(FunctionCall);