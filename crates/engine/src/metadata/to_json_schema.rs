use serde_json::{json, Value};
use wasmtime::component::Type;

pub trait ToJsonSchema {
    fn to_json_schema(&self) -> Value;
}

impl ToJsonSchema for Type {
    fn to_json_schema(&self) -> Value {
        match &self {
            Type::Bool => json!({ "type": "boolean" }),
            Type::Borrow(_) => todo!(),
            Type::Char | Type::String => json!({ "type": "string" }),
            Type::Enum(enum_) => {
                let values = enum_.names().map(|c| c.to_string()).collect::<Vec<_>>();
                json!({
                    "type": "string",
                    "enum": values
                })
            }
            Type::Flags(flags) => {
                let properties = flags
                    .names()
                    .map(|f| (f.to_string(), json!({ "type": "boolean" })))
                    .collect::<serde_json::Map<_, _>>();
                json!({
                    "type": "object",
                    "properties": properties
                })
            }
            Type::Float32 | Type::Float64 => json!({ "type": "number" }),
            Type::List(list) => {
                let items = ToJsonSchema::to_json_schema(&list.ty());
                json!({
                    "type": "array",
                    "items": items
                })
            }
            Type::Option(opt) => {
                let inner = ToJsonSchema::to_json_schema(&opt.ty());
                json!({
                    "anyOf": [
                        { "type": "null" },
                        inner
                    ]
                })
            }
            Type::Own(_) => todo!(),
            Type::Record(record) => {
                let properties = record
                    .fields()
                    .map(|f| (f.name.to_string(), ToJsonSchema::to_json_schema(&f.ty)))
                    .collect::<serde_json::Map<_, _>>();
                json!({
                    "type": "object",
                    "properties": properties
                })
            }
            Type::Result(result) => {
                let ok = result
                    .ok()
                    .map(|ok| ToJsonSchema::to_json_schema(&ok))
                    .unwrap_or_else(|| json!(null));
                let err = result
                    .err()
                    .map(|err| ToJsonSchema::to_json_schema(&err))
                    .unwrap_or_else(|| json!(null));
                json!({
                    "type": "object",
                    "properties": {
                        "Ok": ok,
                        "Err": err
                    }
                })
            }
            Type::S8 | Type::S16 | Type::S32 | Type::S64 => {
                json!({ "type": "integer", "format": "int" })
            }
            Type::Tuple(tuple) => {
                let items = tuple
                    .types()
                    .map(|t| ToJsonSchema::to_json_schema(&t))
                    .collect::<Vec<_>>();
                json!({
                    "type": "array",
                    "prefixItems": items
                })
            }
            Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
                json!({ "type": "integer", "format": "uint" })
            }
            Type::Variant(variant) => {
                let one_of = variant
                    .cases()
                    .map(|c| {
                        let ty = ToJsonSchema::to_json_schema(&c.ty.unwrap());
                        json!({
                            "title": c.name.to_string(),
                            "allOf": [ty]
                        })
                    })
                    .collect::<Vec<_>>();
                json!({
                    "oneOf": one_of
                })
            }
        }
    }
}
