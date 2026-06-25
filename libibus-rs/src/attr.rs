use serde::{Deserialize, Serialize};
use zvariant::Type;

/// Attribute type for text formatting.
///
/// These correspond to the `IBusAttrType` enum in the IBus protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum AttrType {
    /// Underline style.
    Underline = 1,
    /// Foreground colour (ABGR).
    Foreground = 2,
    /// Background colour (ABGR).
    Background = 3,
    /// Font style (CSS-like string).
    FontStyle = 4,
    /// Font weight.
    FontWeight = 5,
    /// Rise (vertical offset in pixels).
    Rise = 6,
    /// Strikethrough.
    Strikethrough = 7,
    /// Font scale factor (percent × 100, e.g. 200 = 2×).
    Scale = 8,
    /// Text alignment.
    Align = 9,
}

impl AttrType {
    /// Convert a `u32` to an `AttrType`, returning `None` for unknown values.
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            1 => Some(Self::Underline),
            2 => Some(Self::Foreground),
            3 => Some(Self::Background),
            4 => Some(Self::FontStyle),
            5 => Some(Self::FontWeight),
            6 => Some(Self::Rise),
            7 => Some(Self::Strikethrough),
            8 => Some(Self::Scale),
            9 => Some(Self::Align),
            _ => None,
        }
    }

    /// Convert to the protocol integer.
    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

/// A single text attribute (underline, foreground colour, etc.).
///
/// Attributes have a type, a value, and a start/end range within the text.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Attr {
    /// Attribute type (see [`AttrType`]).
    pub attr_type: u32,
    /// Attribute value (interpretation depends on type).
    pub value: u32,
    /// Start byte index (inclusive).
    pub start_index: u32,
    /// End byte index (exclusive).
    pub end_index: u32,
}

impl Attr {
    /// Create a new attribute.
    pub fn new(attr_type: AttrType, value: u32, start_index: u32, end_index: u32) -> Self {
        Self {
            attr_type: attr_type as u32,
            value,
            start_index,
            end_index,
        }
    }

    /// Create an underline attribute.
    pub fn underline(style: u32, start_index: u32, end_index: u32) -> Self {
        Self::new(AttrType::Underline, style, start_index, end_index)
    }

    /// Create a foreground colour attribute.
    ///
    /// `color` is an ABGR value (e.g. `0x00ff0000` for red).
    pub fn foreground(color: u32, start_index: u32, end_index: u32) -> Self {
        Self::new(AttrType::Foreground, color, start_index, end_index)
    }

    /// Create a background colour attribute.
    ///
    /// `color` is an ABGR value.
    pub fn background(color: u32, start_index: u32, end_index: u32) -> Self {
        Self::new(AttrType::Background, color, start_index, end_index)
    }

    /// Return the [`AttrType`] if it is recognised, or `None`.
    pub fn attr_type(&self) -> Option<AttrType> {
        AttrType::from_u32(self.attr_type)
    }
}

/// A list of [`Attr`] instances applying to a text.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
pub struct AttrList {
    /// The contained attributes.
    pub attrs: Vec<Attr>,
}

impl AttrList {
    /// Create an empty attribute list.
    pub fn new() -> Self {
        Self { attrs: Vec::new() }
    }

    /// Append an attribute.
    pub fn append(&mut self, attr: Attr) {
        self.attrs.push(attr);
    }

    /// Number of attributes.
    pub fn len(&self) -> usize {
        self.attrs.len()
    }

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.attrs.is_empty()
    }
}

impl From<Vec<Attr>> for AttrList {
    fn from(attrs: Vec<Attr>) -> Self {
        Self { attrs }
    }
}

use crate::error::{Error, Result};
use crate::serializable::{
    IBusSerializable, unwrap_serializable, variant_signature, wrap_serializable,
};
use zvariant::Value;

impl IBusSerializable for Attr {
    fn class_name() -> &'static str {
        "IBusAttribute"
    }

    fn to_value(&self) -> Value<'static> {
        let inner = Value::from((self.attr_type, self.value, self.start_index, self.end_index));
        wrap_serializable(Self::class_name(), inner)
    }

    fn from_value(value: &Value<'_>) -> Result<Self> {
        let inner = unwrap_serializable(value, Self::class_name())?;
        if let Value::Structure(struct_) = inner {
            let fields = struct_.fields();
            if fields.len() >= 4 {
                let attr_type = fields[0]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid attr_type: {}", e)))?;
                let value_num = fields[1]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid value: {}", e)))?;
                let start_index = fields[2]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid start_index: {}", e)))?;
                let end_index = fields[3]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid end_index: {}", e)))?;
                return Ok(Attr {
                    attr_type,
                    value: value_num,
                    start_index,
                    end_index,
                });
            }
        }
        Err(Error::Connection(
            "Invalid IBusAttribute inner structure".into(),
        ))
    }
}

impl IBusSerializable for AttrList {
    fn class_name() -> &'static str {
        "IBusAttrList"
    }

    fn to_value(&self) -> Value<'static> {
        let sig = variant_signature();
        let mut array = zvariant::Array::new(sig);
        for attr in &self.attrs {
            array
                .append(Value::Value(Box::new(attr.to_value())))
                .unwrap();
        }
        let inner = Value::Array(array);
        wrap_serializable(Self::class_name(), inner)
    }

    fn from_value(value: &Value<'_>) -> Result<Self> {
        let inner = unwrap_serializable(value, Self::class_name())?;
        if let Value::Array(arr) = inner {
            let mut attrs = Vec::new();
            for val in arr.as_ref() {
                attrs.push(Attr::from_value(val)?);
            }
            return Ok(AttrList { attrs });
        }
        Err(Error::Connection(
            "Invalid IBusAttrList inner structure".into(),
        ))
    }
}
