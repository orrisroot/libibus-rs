use serde::{Deserialize, Serialize};
use zvariant::Type;

/// Attribute type for text formatting.
///
/// These correspond to the `IBusAttrType` enum in the IBus protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum AttrType {
    /// Underline style.
    Underline = 0,
    /// Foreground colour (ABGR).
    Foreground = 1,
    /// Background colour (ABGR).
    Background = 2,
    /// Font style (CSS-like string).
    FontStyle = 3,
    /// Font weight.
    FontWeight = 4,
    /// Rise (vertical offset in pixels).
    Rise = 5,
    /// Strikethrough.
    Strikethrough = 6,
    /// Font scale factor (percent × 100, e.g. 200 = 2×).
    Scale = 7,
    /// Text alignment.
    Align = 8,
}

impl AttrType {
    /// Convert a `u32` to an `AttrType`, returning `None` for unknown values.
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Underline),
            1 => Some(Self::Foreground),
            2 => Some(Self::Background),
            3 => Some(Self::FontStyle),
            4 => Some(Self::FontWeight),
            5 => Some(Self::Rise),
            6 => Some(Self::Strikethrough),
            7 => Some(Self::Scale),
            8 => Some(Self::Align),
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
