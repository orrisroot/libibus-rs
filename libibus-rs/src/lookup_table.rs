use serde::{Deserialize, Serialize};
use zvariant::Type;

use crate::text::Text;

/// Orientation of the lookup table in the IBus panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum LookupOrientation {
    /// Horizontal layout.
    Horizontal = 0,
    /// Vertical layout.
    Vertical = 1,
    /// System default orientation.
    System = 2,
}

impl LookupOrientation {
    pub const fn to_u32(self) -> u32 {
        self as u32
    }

    pub const fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::Horizontal),
            1 => Some(Self::Vertical),
            2 => Some(Self::System),
            _ => None,
        }
    }

    pub const fn from_u32(v: u32) -> Option<Self> {
        Self::from_i32(v as i32)
    }
}

impl From<LookupOrientation> for u32 {
    fn from(o: LookupOrientation) -> Self {
        o as u32
    }
}

impl From<u32> for LookupOrientation {
    fn from(v: u32) -> Self {
        Self::from_u32(v).unwrap_or(Self::Vertical)
    }
}

/// A set of conversion candidates displayed in the IBus panel.
///
/// Used by input method engines to show Japanese/Chinese/etc. conversion
/// candidates.  The panel renders the candidates in a window, supports
/// paging, cursor movement, and selection by index.
///
/// # Example
///
/// ```
/// use libibus_rs::{LookupTable, Text};
///
/// let mut table = LookupTable::new();
/// table.append_candidate(Text::new("候補1"));
/// table.append_candidate(Text::new("候補2"));
/// table.set_cursor_pos(0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LookupTable {
    /// List of candidate strings.
    candidates: Vec<Text>,
    /// Labels shown beside each candidate (may be empty to use 1-based indices).
    labels: Vec<Text>,
    /// Absolute cursor position in the full candidate list.
    cursor_pos: u32,
    /// Whether the cursor highlight is visible.
    cursor_visible: bool,
    /// Whether to wrap around at the start/end of the list.
    round: bool,
    /// Layout orientation (0=Horizontal, 1=Vertical, 2=System).
    orientation: i32,
    /// Number of candidates per page.
    page_size: u32,
    /// Cursor position within the current page.
    cursor_pos_in_page: u32,
}

impl Default for LookupTable {
    fn default() -> Self {
        Self {
            candidates: Vec::new(),
            labels: Vec::new(),
            cursor_pos: 0,
            cursor_visible: true,
            round: false,
            orientation: LookupOrientation::System as i32,
            page_size: 5,
            cursor_pos_in_page: 0,
        }
    }
}

impl LookupTable {
    /// Create an empty lookup table.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all candidates and reset the table state.
    pub fn clear(&mut self) {
        self.candidates.clear();
        self.labels.clear();
        self.cursor_pos = 0;
        self.cursor_visible = true;
        self.round = false;
        self.page_size = 5;
        self.cursor_pos_in_page = 0;
    }

    /// Append a candidate at the end of the list.
    pub fn append_candidate(&mut self, candidate: Text) {
        self.candidates.push(candidate);
    }

    /// Replace all candidates.
    pub fn set_candidates(&mut self, candidates: Vec<Text>) {
        self.candidates = candidates;
        if self.cursor_pos >= self.candidates.len() as u32 {
            self.cursor_pos = 0;
        }
        self.update_cursor_in_page();
    }

    /// Set the absolute cursor position and update the page-relative position.
    pub fn set_cursor_pos(&mut self, pos: u32) {
        if pos < self.candidates.len() as u32 {
            self.cursor_pos = pos;
        } else if !self.candidates.is_empty() {
            self.cursor_pos = self.candidates.len() as u32 - 1;
        } else {
            self.cursor_pos = 0;
        }
        self.update_cursor_in_page();
    }

    /// Return the absolute cursor position.
    pub fn cursor_pos(&self) -> u32 {
        self.cursor_pos
    }

    /// Return a reference to all candidates.
    pub fn candidates(&self) -> &[Text] {
        &self.candidates
    }

    /// Return a reference to all labels.
    pub fn labels(&self) -> &[Text] {
        &self.labels
    }

    /// Return whether the cursor highlight is visible.
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Return whether wrapping at list boundaries is enabled.
    pub fn round(&self) -> bool {
        self.round
    }

    /// Return the orientation of the lookup table.
    pub fn orientation(&self) -> LookupOrientation {
        LookupOrientation::from_i32(self.orientation).unwrap_or(LookupOrientation::Vertical)
    }

    /// Return the cursor position within the current page.
    pub fn cursor_pos_in_page(&self) -> u32 {
        self.cursor_pos_in_page
    }

    fn update_cursor_in_page(&mut self) {
        if self.page_size > 0 {
            self.cursor_pos_in_page = self.cursor_pos % self.page_size;
        }
    }

    /// Set the number of candidates per page.
    pub fn set_page_size(&mut self, size: u32) {
        self.page_size = size;
        self.update_cursor_in_page();
    }

    /// Return the page size.
    pub fn page_size(&self) -> u32 {
        self.page_size
    }

    /// Set the panel layout orientation.
    pub fn set_orientation(&mut self, orientation: LookupOrientation) {
        self.orientation = orientation as i32;
    }

    /// Enable or disable wrapping at list boundaries.
    pub fn set_round(&mut self, round: bool) {
        self.round = round;
    }

    /// Show or hide the cursor highlight.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
    }

    /// Replace all candidate labels.
    pub fn set_labels(&mut self, labels: Vec<Text>) {
        self.labels = labels;
    }

    /// Move the cursor to the previous page.
    ///
    /// Returns `false` if the cursor was already on the first page and
    /// [`round`](Self::round) is `false`.
    pub fn page_up(&mut self) -> bool {
        if self.cursor_pos >= self.page_size {
            self.cursor_pos -= self.page_size;
            self.update_cursor_in_page();
            true
        } else if self.round && self.page_size > 0 {
            let num_pages = (self.candidates.len() as u32).div_ceil(self.page_size);
            if num_pages > 1 {
                let last_page_start = (num_pages - 1) * self.page_size;
                if last_page_start > 0 {
                    self.cursor_pos = last_page_start - 1;
                } else {
                    self.cursor_pos = 0;
                }
                self.update_cursor_in_page();
            }
            true
        } else {
            false
        }
    }

    /// Move the cursor to the next page.
    ///
    /// Returns `false` if the cursor was already on the last page and
    /// [`round`](Self::round) is `false`.
    pub fn page_down(&mut self) -> bool {
        if self.page_size == 0 {
            return false;
        }
        let new_pos = self.cursor_pos + self.page_size;
        if new_pos < self.candidates.len() as u32 {
            self.cursor_pos = new_pos;
            self.update_cursor_in_page();
            true
        } else if self.round {
            self.cursor_pos = 0;
            self.update_cursor_in_page();
            true
        } else {
            false
        }
    }

    /// Move the cursor up by one candidate.
    ///
    /// Returns `false` if the cursor was already at position 0 and
    /// [`round`](Self::round) is `false`.
    pub fn cursor_up(&mut self) -> bool {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            self.update_cursor_in_page();
            true
        } else if self.round && !self.candidates.is_empty() {
            self.cursor_pos = self.candidates.len() as u32 - 1;
            self.update_cursor_in_page();
            true
        } else {
            false
        }
    }

    /// Move the cursor down by one candidate.
    ///
    /// Returns `false` if the cursor was already at the last candidate and
    /// [`round`](Self::round) is `false`.
    pub fn cursor_down(&mut self) -> bool {
        if self.cursor_pos + 1 < self.candidates.len() as u32 {
            self.cursor_pos += 1;
            self.update_cursor_in_page();
            true
        } else if self.round && !self.candidates.is_empty() {
            self.cursor_pos = 0;
            self.update_cursor_in_page();
            true
        } else {
            false
        }
    }

    /// Return a slice of candidates on the current page.
    pub fn current_page(&self) -> &[Text] {
        let start = self.cursor_pos - self.cursor_pos_in_page;
        let start = start as usize;
        let end = std::cmp::min(start + self.page_size as usize, self.candidates.len());
        &self.candidates[start..end]
    }

    /// Set the cursor to a specific position within the current page.
    pub fn set_cursor_pos_in_page(&mut self, pos: u32) {
        if self.page_size > 0 && pos < self.page_size && pos < self.candidates.len() as u32 {
            self.cursor_pos = (self.cursor_pos / self.page_size) * self.page_size + pos;
            self.cursor_pos_in_page = pos;
        }
    }

    /// Return the total number of candidates.
    pub fn number_of_candidates(&self) -> u32 {
        self.candidates.len() as u32
    }

    /// Whether there are no candidates.
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    /// Return a reference to the candidate at the current cursor position.
    pub fn get_current_candidate(&self) -> Option<&Text> {
        self.candidates.get(self.cursor_pos as usize)
    }
}

use crate::error::{Error, Result};
use crate::serializable::{
    IBusSerializable, unwrap_serializable, variant_signature, wrap_serializable,
};
use zvariant::Value;

impl IBusSerializable for LookupTable {
    fn class_name() -> &'static str {
        "IBusLookupTable"
    }

    fn to_value(&self) -> Value<'static> {
        let sig = variant_signature();

        let mut cand_array = zvariant::Array::new(sig);
        for cand in &self.candidates {
            cand_array
                .append(Value::Value(Box::new(cand.to_value())))
                .unwrap();
        }
        let cands = Value::Array(cand_array);

        let mut label_array = zvariant::Array::new(sig);
        for label in &self.labels {
            label_array
                .append(Value::Value(Box::new(label.to_value())))
                .unwrap();
        }
        let labels = Value::Array(label_array);

        use zvariant::StructureBuilder;
        let mut builder = StructureBuilder::new();
        builder = builder.append_field(Value::from(self.page_size));
        builder = builder.append_field(Value::from(self.cursor_pos));
        builder = builder.append_field(Value::from(self.cursor_visible));
        builder = builder.append_field(Value::from(self.round));
        builder = builder.append_field(Value::from(self.orientation));
        builder = builder.append_field(cands);
        builder = builder.append_field(labels);
        let inner = Value::Structure(builder.build().unwrap());
        wrap_serializable(Self::class_name(), inner)
    }

    fn from_value(value: &Value<'_>) -> Result<Self> {
        let inner = unwrap_serializable(value, Self::class_name())?;
        if let Value::Structure(struct_) = inner {
            let fields = struct_.fields();
            if fields.len() >= 7 {
                let page_size = fields[0]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid page_size: {}", e)))?;
                let cursor_pos = fields[1]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid cursor_pos: {}", e)))?;
                let cursor_visible = fields[2]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid cursor_visible: {}", e)))?;
                let round = fields[3]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid round: {}", e)))?;
                let orientation: i32 = fields[4]
                    .clone()
                    .try_into()
                    .map_err(|e| Error::Connection(format!("Invalid orientation: {}", e)))?;

                let cands_val = if let Value::Value(v) = &fields[5] {
                    v.as_ref()
                } else {
                    &fields[5]
                };
                let candidates = if let Value::Array(arr) = cands_val {
                    let mut c = Vec::new();
                    for v in arr.as_ref() {
                        c.push(Text::from_value(v)?);
                    }
                    c
                } else {
                    return Err(Error::Connection("Invalid candidates array".into()));
                };

                let labels_val = if let Value::Value(v) = &fields[6] {
                    v.as_ref()
                } else {
                    &fields[6]
                };
                let labels = if let Value::Array(arr) = labels_val {
                    let mut l = Vec::new();
                    for v in arr.as_ref() {
                        l.push(Text::from_value(v)?);
                    }
                    l
                } else {
                    return Err(Error::Connection("Invalid labels array".into()));
                };

                let cursor_pos_in_page = if page_size > 0 {
                    cursor_pos % page_size
                } else {
                    0
                };

                return Ok(LookupTable {
                    candidates,
                    labels,
                    cursor_pos,
                    cursor_visible,
                    round,
                    orientation,
                    page_size,
                    cursor_pos_in_page,
                });
            }
        }
        Err(Error::Connection(
            "Invalid IBusLookupTable inner structure".into(),
        ))
    }
}
