use serde::{Deserialize, Serialize};
use zvariant::Type;

use crate::attr::AttrList;

/// A string with optional text attributes.
///
/// Wraps a plain text string together with an [`AttrList`] for formatting
/// (underline, foreground colour, background colour, etc.) and a cursor
/// position.  Used throughout IBus for pre-edit text, candidate labels,
/// auxiliary text, and property labels.
///
/// # Example
///
/// ```
/// use libibus_rs::Text;
///
/// let t = Text::new("hello");
/// assert_eq!(t.len(), 5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Text {
    /// The plain-text content.
    pub text: String,
    /// Text attributes (underline, colour, etc.).
    pub attrs: AttrList,
    /// Cursor position within the text.
    pub cursor_pos: u32,
}

impl Text {
    /// Create a new text with no attributes.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            attrs: AttrList::new(),
            cursor_pos: text.len() as u32,
        }
    }

    /// Create a new text with attributes.
    pub fn with_attrs(text: &str, attrs: AttrList) -> Self {
        let cursor_pos = text.len() as u32;
        Self {
            text: text.to_owned(),
            attrs,
            cursor_pos,
        }
    }

    /// Create a text with attributes and a specific cursor position.
    pub fn with_cursor(text: &str, attrs: AttrList, cursor_pos: u32) -> Self {
        Self {
            text: text.to_owned(),
            attrs,
            cursor_pos,
        }
    }

    /// Return the byte length of the text.
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Whether the text is empty.
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Append a string to the text and move the cursor to the end.
    pub fn append(&mut self, other: &str) {
        self.text.push_str(other);
        self.cursor_pos = self.text.len() as u32;
    }

    /// Clear the text, attributes, and reset the cursor.
    pub fn clear(&mut self) {
        self.text.clear();
        self.attrs = AttrList::new();
        self.cursor_pos = 0;
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        Self::new(&text)
    }
}

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl From<&String> for Text {
    fn from(text: &String) -> Self {
        Self::new(text.as_str())
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

use crate::error::{Error, Result};
use crate::serializable::{IBusSerializable, unwrap_serializable, wrap_serializable};
use zvariant::Value;

impl IBusSerializable for Text {
    fn class_name() -> &'static str {
        "IBusText"
    }

    fn to_value(&self) -> Value<'static> {
        use zvariant::StructureBuilder;
        let mut builder = StructureBuilder::new();
        builder = builder.append_field(Value::from(self.text.clone()));
        builder = builder.append_field(Value::Value(Box::new(self.attrs.to_value())));
        let inner = Value::Structure(builder.build().unwrap());
        wrap_serializable(Self::class_name(), inner)
    }

    fn from_value(value: &Value<'_>) -> Result<Self> {
        let inner = unwrap_serializable(value, Self::class_name())?;
        if let Value::Structure(struct_) = inner {
            let fields = struct_.fields();
            if fields.len() >= 2 {
                let text_str: String = fields[0]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid text: {}", e)))?;
                let attrs_val = if let Value::Value(v) = &fields[1] {
                    v.as_ref()
                } else {
                    &fields[1]
                };
                let attrs = AttrList::from_value(attrs_val)?;
                let cursor_pos = text_str.len() as u32;
                return Ok(Text {
                    text: text_str,
                    attrs,
                    cursor_pos,
                });
            }
        }
        Err(Error::Connection("Invalid IBusText inner structure".into()))
    }
}
