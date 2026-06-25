use crate::error::{Error, Result};
use std::sync::OnceLock;
use zvariant::{Dict, Signature, StructureBuilder, Value};

/// Trait for objects that can be serialized into the IBusSerializable GVariant format.
pub trait IBusSerializable: Sized {
    /// Class name corresponding to the C implementation (e.g. "IBusText").
    fn class_name() -> &'static str;
    /// Serialize the object into a GVariant Value.
    fn to_value(&self) -> Value<'static>;
    /// Deserialize the object from a GVariant Value.
    fn from_value(value: &Value<'_>) -> Result<Self>;
}

/// Get the cached Signature for "v" (Variant).
pub fn variant_signature() -> &'static Signature {
    static SIGNATURE: OnceLock<Signature> = OnceLock::new();
    SIGNATURE.get_or_init(|| Signature::try_from("v").unwrap())
}

pub fn wrap_serializable<'a>(class_name: &'a str, inner: Value<'a>) -> Value<'a> {
    let dict = Dict::new(&Signature::Str, &Signature::Variant);
    let dict_val = Value::Dict(dict);

    let mut fields = vec![Value::new(class_name), dict_val];
    match inner {
        Value::Structure(struct_) => {
            for field in struct_.into_fields() {
                fields.push(field);
            }
        }
        other => {
            fields.push(other);
        }
    }
    let mut builder = StructureBuilder::new();
    for field in fields {
        builder = builder.append_field(field);
    }
    Value::Structure(builder.build().unwrap())
}

pub fn unwrap_serializable<'a>(value: &'a Value<'a>, expected_class: &str) -> Result<Value<'a>> {
    let mut current = value;
    while let Value::Value(inner) = current {
        current = inner.as_ref();
    }
    if let Value::Structure(struct_) = current {
        let fields = struct_.fields();
        if fields.len() >= 2
            && let Value::Str(class_name) = &fields[0]
        {
            if class_name.as_str() == expected_class {
                if fields.len() == 3 {
                    return Ok(fields[2].clone());
                } else {
                    let mut builder = StructureBuilder::new();
                    for field in &fields[2..] {
                        builder = builder.append_field(field.clone());
                    }
                    return Ok(Value::Structure(builder.build().unwrap()));
                }
            } else {
                return Err(Error::Connection(format!(
                    "Expected class {}, got {}",
                    expected_class,
                    class_name.as_str()
                )));
            }
        }
    }
    Err(Error::Connection(format!(
        "Invalid IBusSerializable value structure (expected {})",
        expected_class
    )))
}
