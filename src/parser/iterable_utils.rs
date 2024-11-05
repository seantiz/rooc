use crate::primitives::{IterableKind, Primitive, PrimitiveKind};

/// Flattens an array of primitives into a single primitive iterable.
/// The resulting iterable type is determined by the type of the first element.
///
/// # Arguments
/// * `values` - Vector of primitives to flatten
///
/// # Returns
/// A single Primitive containing an IterableKind with the flattened values
pub fn flatten_primitive_array_values(values: Vec<Primitive>) -> Result<Primitive, String> {
    let first = values.first();
    if first.is_none() {
        return Ok(Primitive::Iterable(IterableKind::Anys(vec![])));
    }
    let first_kind = first.unwrap().get_type();
    //TODO try to make this return a Mixed Primitive if the types are different, instead of failing
    match first_kind {
        PrimitiveKind::Any => Ok(Primitive::Iterable(IterableKind::Numbers(vec![]))), //can never happen
        PrimitiveKind::Boolean => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Boolean(b) => Ok(b),
                    _ => Err(format!("Expected Boolean but got {}", v.type_string())),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Booleans(values)))
        }
        PrimitiveKind::Number => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Number(n) => Ok(n),
                    _ => Err(format!("Expected Number, got \"{}\"", v.type_string(),)),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Numbers(values)))
        }
        PrimitiveKind::Integer => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Integer(i) => Ok(i),
                    _ => Err(format!("Expected Integer but got {}", v.type_string())),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Integers(values)))
        }
        PrimitiveKind::PositiveInteger => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::PositiveInteger(i) => Ok(i),
                    _ => Err(format!(
                        "Expected PositiveInteger but got {}",
                        v.type_string()
                    )),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::PositiveIntegers(values)))
        }
        PrimitiveKind::String => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::String(s) => Ok(s),
                    _ => Err(format!("Expected String but got {}", v.type_string())),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Strings(values)))
        }
        PrimitiveKind::GraphEdge => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::GraphEdge(e) => Ok(e),
                    _ => Err(format!("Expected GraphEdge but got {}", v.type_string())),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Edges(values)))
        }
        PrimitiveKind::GraphNode => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::GraphNode(n) => Ok(n),
                    _ => Err(format!("Expected GraphNode but got {}", v.type_string())),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Nodes(values)))
        }
        PrimitiveKind::Tuple(_) => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Tuple(t) => Ok(t),
                    _ => Err(format!("Expected Tuple but got {}", v.type_string())),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Tuples(values)))
        }
        PrimitiveKind::Iterable(_) => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Iterable(i) => Ok(i),
                    _ => Err(format!("Expected Iterable but got {}", v.type_string())),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Iterables(values)))
        }
        PrimitiveKind::Undefined => Ok(Primitive::Iterable(IterableKind::Numbers(vec![]))),
        PrimitiveKind::Graph => {
            let values = values
                .into_iter()
                .map(|v| match v {
                    Primitive::Graph(g) => Ok(g),
                    _ => Err(format!("Expected Graph but got {}", v.type_string())),
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(Primitive::Iterable(IterableKind::Graphs(values)))
        }
    }
}
