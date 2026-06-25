use serde::{Deserialize, Serialize};
use zvariant::{Signature, Type};

use crate::text::Text;

/// Visual type of an IBus property.
///
/// Serialized as a `u32` on the wire, matching the IBus protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum PropType {
    /// Normal push-button property.
    Normal = 0,
    /// Toggle (checkable) property.
    Toggle = 1,
    /// Radio-button property.
    Radio = 2,
    /// Sub-menu property.
    Menu = 3,
    /// Visual separator.
    Separator = 4,
}

impl PropType {
    pub const fn to_u32(self) -> u32 {
        self as u32
    }

    pub const fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Normal),
            1 => Some(Self::Toggle),
            2 => Some(Self::Radio),
            3 => Some(Self::Menu),
            4 => Some(Self::Separator),
            _ => None,
        }
    }
}

impl From<PropType> for u32 {
    fn from(t: PropType) -> Self {
        t as u32
    }
}

impl From<u32> for PropType {
    fn from(v: u32) -> Self {
        Self::from_u32(v).unwrap_or(Self::Normal)
    }
}

/// Check state for toggle/radio properties.
///
/// Serialized as a `u32` on the wire, matching the IBus protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum PropState {
    /// Unchecked state.
    Unchecked = 0,
    /// Checked state.
    Checked = 1,
    /// Inconsistent (mixed) state.
    Inconsistent = 2,
}

impl PropState {
    pub const fn to_u32(self) -> u32 {
        self as u32
    }

    pub const fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Unchecked),
            1 => Some(Self::Checked),
            2 => Some(Self::Inconsistent),
            _ => None,
        }
    }
}

impl From<PropState> for u32 {
    fn from(s: PropState) -> Self {
        s as u32
    }
}

impl From<u32> for PropState {
    fn from(v: u32) -> Self {
        Self::from_u32(v).unwrap_or(Self::Unchecked)
    }
}

/// A single item in an IBus engine's property (menu) panel.
///
/// Properties allow engines to expose buttons, toggles, and sub-menus in
/// the IBus panel UI.
///
/// # Example
///
/// ```
/// use libibus_rs::Prop;
///
/// let mut prop = Prop::new("mode", "Input Mode");
/// prop.set_tooltip("Switch input mode");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Prop {
    /// Unique key for identifying this property.
    pub key: String,
    /// Visual type (see [`PropType`]).
    pub prop_type: u32,
    /// Display label.
    pub label: Text,
    /// Icon name.
    pub icon: String,
    /// Tooltip text.
    pub tooltip: Text,
    /// Whether the property is sensitive (interactive).
    pub sensitive: bool,
    /// Whether the property is visible.
    pub visible: bool,
    /// Check state (see [`PropState`]).
    pub state: u32,
    /// Sub-properties (for menu-type properties).
    ///
    /// Note: This is a `Box<PropList>` (not `PropList`) to break the recursive
    /// `zvariant::Type` derive cycle between `Prop` and `PropList`.
    pub sub_props: Box<PropList>,
    /// Symbol text representation of the property (e.g. used for displaying key cap symbols in the panel).
    pub symbol: Text,
}

impl Prop {
    /// Create a new normal-type property.
    pub fn new(key: &str, label: &str) -> Self {
        Self {
            key: key.to_owned(),
            prop_type: PropType::Normal as u32,
            label: Text::new(label),
            icon: String::new(),
            tooltip: Text::new(""),
            sensitive: true,
            visible: true,
            state: PropState::Unchecked as u32,
            sub_props: Box::new(PropList::new()),
            symbol: Text::new(""),
        }
    }

    /// Create a separator.
    pub fn separator() -> Self {
        Self {
            key: String::new(),
            prop_type: PropType::Separator as u32,
            label: Text::new(""),
            icon: String::new(),
            tooltip: Text::new(""),
            sensitive: false,
            visible: true,
            state: PropState::Unchecked as u32,
            sub_props: Box::new(PropList::new()),
            symbol: Text::new(""),
        }
    }

    /// Create a toggle (checkable) property.
    pub fn toggle(key: &str, label: &str) -> Self {
        let mut p = Self::new(key, label);
        p.prop_type = PropType::Toggle as u32;
        p
    }

    /// Create a radio property.
    pub fn radio(key: &str, label: &str) -> Self {
        let mut p = Self::new(key, label);
        p.prop_type = PropType::Radio as u32;
        p
    }

    /// Set the label.
    pub fn set_label(&mut self, label: &str) -> &mut Self {
        self.label = Text::new(label);
        self
    }

    /// Set the icon name.
    pub fn set_icon(&mut self, icon: &str) -> &mut Self {
        self.icon = icon.to_owned();
        self
    }

    /// Set the tooltip text.
    pub fn set_tooltip(&mut self, tooltip: &str) -> &mut Self {
        self.tooltip = Text::new(tooltip);
        self
    }

    /// Set sensitivity (whether the user can interact with it).
    pub fn set_sensitive(&mut self, sensitive: bool) -> &mut Self {
        self.sensitive = sensitive;
        self
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) -> &mut Self {
        self.visible = visible;
        self
    }

    /// Set the check state.
    pub fn set_state(&mut self, state: PropState) -> &mut Self {
        self.state = state as u32;
        self
    }

    /// Set whether the property is checked (convenience for toggle properties).
    pub fn set_checked(&mut self, checked: bool) -> &mut Self {
        self.state = if checked {
            PropState::Checked as u32
        } else {
            PropState::Unchecked as u32
        };
        self
    }

    /// Attach sub-properties (for menu-type properties).
    pub fn set_sub_props(&mut self, sub_props: PropList) -> &mut Self {
        *self.sub_props = sub_props;
        self
    }

    /// Set the symbol text.
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Self {
        self.symbol = Text::new(symbol);
        self
    }

    /// Whether this is a toggle-type property.
    pub fn is_toggle(&self) -> bool {
        self.prop_type == PropType::Toggle as u32
    }

    /// Whether this is a separator.
    pub fn is_separator(&self) -> bool {
        self.prop_type == PropType::Separator as u32
    }

    /// Whether this property is in a checked state.
    pub fn is_checked(&self) -> bool {
        self.state == PropState::Checked as u32
    }

    /// Whether this property has sub-properties.
    pub fn has_sub_props(&self) -> bool {
        !self.sub_props.is_empty()
    }
}

/// A list of [`Prop`] items.
///
/// Used for the engine property panel and for sub-menu contents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PropList {
    /// The contained properties.
    pub props: Vec<Prop>,
}

// Manual Type impl to break the Prop ↔ Box<PropList> ↔ PropList recursive cycle.
// PropList in IBus is serialized as an array of IBusProperty structures (v = variant).
static PROP_LIST_SIG: Signature = Signature::static_array(&Signature::Variant);

impl Type for PropList {
    const SIGNATURE: &'static Signature = &PROP_LIST_SIG;
}

impl PropList {
    /// Create an empty property list.
    pub fn new() -> Self {
        Self { props: Vec::new() }
    }

    /// Append a property.
    pub fn append(&mut self, prop: Prop) {
        self.props.push(prop);
    }

    /// Number of properties in this list.
    pub fn len(&self) -> usize {
        self.props.len()
    }

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.props.is_empty()
    }

    /// Update a property's state/label/icon recursively.
    pub fn update_property(&mut self, prop: &Prop) {
        for p in &mut self.props {
            if p.key == prop.key {
                p.state = prop.state;
                p.label = prop.label.clone();
                p.icon.clone_from(&prop.icon);
                p.tooltip = prop.tooltip.clone();
                p.sensitive = prop.sensitive;
                p.visible = prop.visible;
                return;
            }
            if p.has_sub_props() {
                p.sub_props.update_property(prop);
            }
        }
    }

    /// Look up a property by key (recursive).
    pub fn get(&self, key: &str) -> Option<&Prop> {
        for p in &self.props {
            if p.key == key {
                return Some(p);
            }
            if p.has_sub_props()
                && let Some(found) = p.sub_props.get(key)
            {
                return Some(found);
            }
        }
        None
    }
}

impl From<Vec<Prop>> for PropList {
    fn from(props: Vec<Prop>) -> Self {
        Self { props }
    }
}

use crate::error::{Error, Result};
use crate::serializable::{
    IBusSerializable, unwrap_serializable, variant_signature, wrap_serializable,
};
use zvariant::Value;

impl IBusSerializable for Prop {
    fn class_name() -> &'static str {
        "IBusProperty"
    }

    fn to_value(&self) -> Value<'static> {
        let label_val = Value::Value(Box::new(self.label.to_value()));
        let tooltip_val = Value::Value(Box::new(self.tooltip.to_value()));
        let sub_props_val = Value::Value(Box::new(self.sub_props.to_value()));
        let symbol_val = Value::Value(Box::new(self.symbol.to_value()));

        let inner = Value::from((
            self.key.clone(),
            self.prop_type,
            label_val,
            self.icon.clone(),
            tooltip_val,
            self.sensitive,
            self.visible,
            self.state,
            sub_props_val,
            symbol_val,
        ));
        wrap_serializable(Self::class_name(), inner)
    }

    fn from_value(value: &Value<'_>) -> Result<Self> {
        let inner = unwrap_serializable(value, Self::class_name())?;
        if let Value::Structure(struct_) = inner {
            let fields = struct_.fields();
            if fields.len() >= 9 {
                let key = fields[0]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid key: {}", e)))?;
                let prop_type = fields[1]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid prop_type: {}", e)))?;

                let label_val = if let Value::Value(v) = &fields[2] {
                    v.as_ref()
                } else {
                    &fields[2]
                };
                let label = Text::from_value(label_val)?;

                let icon = fields[3]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid icon: {}", e)))?;

                let tooltip_val = if let Value::Value(v) = &fields[4] {
                    v.as_ref()
                } else {
                    &fields[4]
                };
                let tooltip = Text::from_value(tooltip_val)?;

                let sensitive = fields[5]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid sensitive: {}", e)))?;
                let visible = fields[6]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid visible: {}", e)))?;
                let state = fields[7]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid state: {}", e)))?;

                let sub_props_val = if let Value::Value(v) = &fields[8] {
                    v.as_ref()
                } else {
                    &fields[8]
                };
                let sub_props = Box::new(PropList::from_value(sub_props_val)?);

                let symbol = if fields.len() >= 10 {
                    let symbol_val = if let Value::Value(v) = &fields[9] {
                        v.as_ref()
                    } else {
                        &fields[9]
                    };
                    Text::from_value(symbol_val)?
                } else {
                    Text::new("")
                };

                return Ok(Prop {
                    key,
                    prop_type,
                    label,
                    icon,
                    tooltip,
                    sensitive,
                    visible,
                    state,
                    sub_props,
                    symbol,
                });
            }
        }
        Err(Error::Connection(
            "Invalid IBusProperty inner structure".into(),
        ))
    }
}

impl IBusSerializable for PropList {
    fn class_name() -> &'static str {
        "IBusPropList"
    }

    fn to_value(&self) -> Value<'static> {
        let sig = variant_signature();
        let mut array = zvariant::Array::new(sig);
        for prop in &self.props {
            array
                .append(Value::Value(Box::new(prop.to_value())))
                .unwrap();
        }
        let inner = Value::Array(array);
        wrap_serializable(Self::class_name(), inner)
    }

    fn from_value(value: &Value<'_>) -> Result<Self> {
        let inner = unwrap_serializable(value, Self::class_name())?;
        if let Value::Array(arr) = inner {
            let mut props = Vec::new();
            for val in arr.as_ref() {
                props.push(Prop::from_value(val)?);
            }
            return Ok(PropList { props });
        }
        Err(Error::Connection(
            "Invalid IBusPropList inner structure".into(),
        ))
    }
}
