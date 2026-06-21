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
}

impl LookupOrientation {
    pub const fn to_u32(self) -> u32 {
        self as u32
    }

    pub const fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(Self::Horizontal),
            1 => Some(Self::Vertical),
            _ => None,
        }
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
    pub candidates: Vec<Text>,
    /// Labels shown beside each candidate (may be empty to use 1-based indices).
    pub labels: Vec<Text>,
    /// Absolute cursor position in the full candidate list.
    pub cursor_pos: u32,
    /// Whether the cursor highlight is visible.
    pub cursor_visible: bool,
    /// Whether to wrap around at the start/end of the list.
    pub round: bool,
    /// Layout orientation (see `ORIENTATION_*` constants).
    pub orientation: u32,
    /// Number of candidates per page.
    pub page_size: u32,
    /// Cursor position within the current page.
    pub cursor_pos_in_page: u32,
}

impl Default for LookupTable {
    fn default() -> Self {
        Self {
            candidates: Vec::new(),
            labels: Vec::new(),
            cursor_pos: 0,
            cursor_visible: true,
            round: false,
            orientation: LookupOrientation::Vertical as u32,
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
    }

    /// Set the absolute cursor position and update the page-relative position.
    pub fn set_cursor_pos(&mut self, pos: u32) {
        self.cursor_pos = pos;
        self.update_cursor_in_page();
    }

    /// Return the absolute cursor position.
    pub fn cursor_pos(&self) -> u32 {
        self.cursor_pos
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
        self.orientation = orientation as u32;
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
            let num_pages = (self.candidates.len() as u32 + self.page_size - 1) / self.page_size;
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
