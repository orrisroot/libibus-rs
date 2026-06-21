use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use zvariant::Type;

pub mod keysym;

bitflags! {
    /// Bitmask of keyboard modifier states.
    ///
    /// Represents the modifier flags in an IBus key event.
    ///
    /// # Examples
    ///
    /// ```
    /// use libibus_rs::ModifierType;
    ///
    /// let m = ModifierType::CONTROL | ModifierType::SHIFT;
    /// assert!(m.contains(ModifierType::CONTROL));
    /// assert!(m.contains(ModifierType::SHIFT));
    /// assert!(!m.contains(ModifierType::MOD1));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ModifierType: u32 {
        /// Shift key
        const SHIFT  = 1 << 0;
        /// Caps Lock
        const LOCK   = 1 << 1;
        /// Control key
        const CONTROL = 1 << 2;
        /// Alt key (Mod1)
        const MOD1   = 1 << 3;
        /// Num Lock (Mod2)
        const MOD2   = 1 << 4;
        /// Mod3
        const MOD3   = 1 << 5;
        /// Super / Windows key (Mod4)
        const MOD4   = 1 << 6;
        /// Mod5
        const MOD5   = 1 << 7;
        /// Button 1
        const BUTTON1 = 1 << 8;
        /// Button 2
        const BUTTON2 = 1 << 9;
        /// Button 3
        const BUTTON3 = 1 << 10;
        /// Button 4
        const BUTTON4 = 1 << 11;
        /// Button 5
        const BUTTON5 = 1 << 12;
        /// Event was handled
        const HANDLED = 1 << 24;
        /// Forward the event
        const FORWARD = 1 << 25;
        /// Key was released (not pressed)
        const RELEASE = 1 << 30;
    }
}

impl ModifierType {
    /// Whether the Shift modifier is set.
    pub fn is_shift(self) -> bool {
        self.contains(Self::SHIFT)
    }

    /// Whether the Control modifier is set.
    pub fn is_control(self) -> bool {
        self.contains(Self::CONTROL)
    }

    /// Whether the Alt (Mod1) modifier is set.
    pub fn is_alt(self) -> bool {
        self.contains(Self::MOD1)
    }

    /// Whether the Super (Mod4) modifier is set.
    pub fn is_super(self) -> bool {
        self.contains(Self::MOD4)
    }

    /// Whether the RELEASE flag is set (key released rather than pressed).
    pub fn is_release(self) -> bool {
        self.contains(Self::RELEASE)
    }

    /// Whether the HANDLED flag is set.
    pub fn is_handled(self) -> bool {
        self.contains(Self::HANDLED)
    }
}

/// An IBus key event.
///
/// Represents a key press or release with its keysym, hardware keycode,
/// and modifier state.
///
/// # Example
///
/// ```
/// use libibus_rs::KeyEvent;
///
/// let ev = KeyEvent::new(0x0061, 0x26, 0); // 'a' key press
/// assert!(!ev.modifiers().is_shift());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct KeyEvent {
    /// The X11 keysym value.
    pub keyval: u32,
    /// Hardware keycode.
    pub keycode: u32,
    /// Bitmask of [`ModifierType`] flags.
    pub state: u32,
}

impl KeyEvent {
    /// Create a new key event.
    pub const fn new(keyval: u32, keycode: u32, state: u32) -> Self {
        Self {
            keyval,
            keycode,
            state,
        }
    }

    /// Return the modifier flags as a [`ModifierType`].
    pub fn modifiers(&self) -> ModifierType {
        ModifierType::from_bits_truncate(self.state)
    }

    /// Set the modifier flags from a [`ModifierType`].
    pub fn set_modifiers(&mut self, modifiers: ModifierType) {
        self.state = modifiers.bits();
    }

    /// Whether this key is a modifier key (Shift, Control, Alt, etc.).
    pub fn is_modifier(&self) -> bool {
        (self.keyval >= 0xffe1 && self.keyval <= 0xffff)
            || self.keyval == keysym::Shift_L
            || self.keyval == keysym::Shift_R
            || self.keyval == keysym::Control_L
            || self.keyval == keysym::Control_R
            || self.keyval == keysym::Meta_L
            || self.keyval == keysym::Meta_R
            || self.keyval == keysym::Alt_L
            || self.keyval == keysym::Alt_R
            || self.keyval == keysym::Super_L
            || self.keyval == keysym::Super_R
            || self.keyval == keysym::Hyper_L
            || self.keyval == keysym::Hyper_R
            || self.keyval == keysym::ISO_Level3_Shift
            || self.keyval == keysym::ISO_Next_Group
    }
}

impl From<(u32, u32, u32)> for KeyEvent {
    fn from((keyval, keycode, state): (u32, u32, u32)) -> Self {
        Self::new(keyval, keycode, state)
    }
}
