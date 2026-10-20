//! Framework-independent keyboard hints for terminal controls.
//!
//! Hosts register visible targets, start a [`Session`], route normalized input,
//! and dispatch the returned target ID. Terminal setup and actions stay with the host.
use std::{collections::VecDeque, error::Error, fmt};

/// Terminal-cell coordinates. Right and bottom edges are exclusive.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}
impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    pub fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }
    pub fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }
    pub fn intersection(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self::new(
            x,
            y,
            self.right().min(other.right()).saturating_sub(x),
            self.bottom().min(other.bottom()).saturating_sub(y),
        )
    }
    pub fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// A host-declared control. IDs must be unique within the active scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target<Id> {
    pub id: Id,
    pub area: Rect,
    /// Human-readable text used by the crowded-layout picker.
    pub title: String,
    pub scope: u64,
    /// Preferred badge position, in terminal cells.
    pub anchor: Option<(u16, u16)>,
}
impl<Id> Target<Id> {
    pub fn new(id: Id, area: Rect, title: impl Into<String>) -> Self {
        Self {
            id,
            area,
            title: title.into(),
            scope: 0,
            anchor: None,
        }
    }
    pub fn scope(mut self, scope: u64) -> Self {
        self.scope = scope;
        self
    }
    pub fn anchor(mut self, x: u16, y: u16) -> Self {
        self.anchor = Some((x, y));
        self
    }
}

/// A validated alphabet. ASCII graphic characters ensure one-cell hint labels.
#[derive(Debug, Clone)]
pub struct Alphabet(Vec<char>);
impl Default for Alphabet {
    fn default() -> Self {
        Self("asdfghjkl".chars().collect())
    }
}
impl Alphabet {
    pub fn new(value: &str) -> Result<Self, HintError> {
        let chars: Vec<_> = value.chars().collect();
        if chars.len() < 2
            || chars.iter().any(|c| !c.is_ascii_graphic())
            || chars
                .iter()
                .enumerate()
                .any(|(i, c)| chars[..i].contains(c))
        {
            return Err(HintError::InvalidAlphabet);
        }
        Ok(Self(chars))
    }
    /// Deterministic, prefix-free labels. Short leaves precede longer ones.
    pub fn labels(&self, count: usize) -> Vec<String> {
        if count == 0 {
            return vec![];
        }
        let mut leaves: VecDeque<String> = self.0.iter().map(char::to_string).collect();
        while leaves.len() < count {
            let prefix = leaves.pop_front().expect("validated alphabet");
            for c in &self.0 {
                leaves.push_back(format!("{prefix}{c}"));
            }
        }
        leaves.into_iter().take(count).collect()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintError {
    InvalidAlphabet,
    DuplicateId,
    NoTargets,
}
impl fmt::Display for HintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidAlphabet => {
                "hint alphabet must contain at least two distinct printable ASCII characters"
            }
            Self::DuplicateId => "target IDs must be unique in the active scope",
            Self::NoTargets => "no visible targets in the active scope",
        })
    }
}
impl Error for HintError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Character(char),
    Backspace,
    Escape,
    Other,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome<Id> {
    Ignored,
    Pending,
    Invalid,
    Cancelled,
    Selected(Id),
}
impl<Id> Outcome<Id> {
    pub fn consumed(&self) -> bool {
        !matches!(self, Self::Ignored)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint<Id> {
    pub target: Target<Id>,
    pub label: String,
}

/// An immutable target snapshot plus a mutable typed prefix.
///
/// Call [`Self::refresh`] before routing input whenever layout or membership may
/// have changed. Any snapshot change cancels the session rather than relabelling.
#[derive(Debug, Clone)]
pub struct Session<Id> {
    alphabet: Alphabet,
    hints: Vec<Hint<Id>>,
    viewport: Rect,
    scope: Option<u64>,
    prefix: String,
    active: bool,
    invalid: bool,
}
impl<Id> Default for Session<Id> {
    fn default() -> Self {
        Self::new(Alphabet::default())
    }
}
impl<Id> Session<Id> {
    pub fn new(alphabet: Alphabet) -> Self {
        Self {
            alphabet,
            hints: vec![],
            viewport: Rect::default(),
            scope: None,
            prefix: String::new(),
            active: false,
            invalid: false,
        }
    }
    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
    pub fn invalid_input(&self) -> bool {
        self.invalid
    }
    pub fn hints(&self) -> &[Hint<Id>] {
        &self.hints
    }
    pub fn viewport(&self) -> Rect {
        self.viewport
    }
    pub fn matches(&self) -> impl Iterator<Item = &Hint<Id>> {
        self.hints
            .iter()
            .filter(|h| h.label.starts_with(&self.prefix))
    }
    pub fn cancel(&mut self) {
        self.active = false;
        self.prefix.clear();
        self.invalid = false;
    }
}
impl<Id: Clone + Eq> Session<Id> {
    fn visible(targets: &[Target<Id>], viewport: Rect, scope: Option<u64>) -> Vec<Target<Id>> {
        targets
            .iter()
            .filter(|t| scope.is_none_or(|s| t.scope == s))
            .filter_map(|t| {
                let area = t.area.intersection(viewport);
                (!area.is_empty()).then(|| {
                    let mut t = t.clone();
                    t.area = area;
                    t
                })
            })
            .collect()
    }
    pub fn start(
        &mut self,
        targets: &[Target<Id>],
        viewport: Rect,
        scope: Option<u64>,
    ) -> Result<(), HintError> {
        self.cancel();
        self.hints.clear();
        let targets = Self::visible(targets, viewport, scope);
        if targets.is_empty() {
            return Err(HintError::NoTargets);
        }
        if targets
            .iter()
            .enumerate()
            .any(|(i, t)| targets[..i].iter().any(|p| p.id == t.id))
        {
            return Err(HintError::DuplicateId);
        }
        let labels = self.alphabet.labels(targets.len());
        self.hints = targets
            .into_iter()
            .zip(labels)
            .map(|(target, label)| Hint { target, label })
            .collect();
        self.viewport = viewport;
        self.scope = scope;
        self.active = true;
        Ok(())
    }
    /// Returns true when an active session was invalidated.
    pub fn refresh(&mut self, targets: &[Target<Id>], viewport: Rect, scope: Option<u64>) -> bool {
        if !self.active {
            return false;
        }
        let visible = Self::visible(targets, viewport, scope);
        let changed = self.viewport != viewport
            || self.scope != scope
            || visible.len() != self.hints.len()
            || visible.iter().zip(&self.hints).any(|(t, h)| t != &h.target);
        if changed {
            self.cancel();
        }
        changed
    }
    pub fn handle(&mut self, input: Input) -> Outcome<Id> {
        if !self.active {
            return Outcome::Ignored;
        }
        self.invalid = false;
        match input {
            Input::Escape => {
                self.cancel();
                Outcome::Cancelled
            }
            Input::Backspace => {
                self.prefix.pop();
                Outcome::Pending
            }
            Input::Character(c) => {
                let mut next = self.prefix.clone();
                next.push(c);
                if let Some(h) = self.hints.iter().find(|h| h.label == next) {
                    let id = h.target.id.clone();
                    self.cancel();
                    return Outcome::Selected(id);
                }
                if self.hints.iter().any(|h| h.label.starts_with(&next)) {
                    self.prefix = next;
                    Outcome::Pending
                } else {
                    self.invalid = true;
                    Outcome::Invalid
                }
            }
            Input::Other => Outcome::Pending,
        }
    }
}
