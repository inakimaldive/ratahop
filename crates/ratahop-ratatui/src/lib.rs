//! Ratatui hint overlays without owning your event loop or widgets.
//!
//! Call [`Hop::update`] after laying out visible controls, [`Hop::start`] from
//! your activation shortcut, and render `&hop` last. Forward active input first.
//!
//! ```
//! use ratahop_ratatui::{rect, Hop, Target, Input, Outcome};
//! use ratatui::layout::Rect;
//! let mut hop = Hop::default();
//! let viewport = Rect::new(0, 0, 80, 24);
//! hop.update(vec![Target::new("save", rect(Rect::new(2, 3, 10, 1)), "Save")], viewport, None);
//! hop.start()?; // Your app chooses the activation shortcut.
//! assert_eq!(hop.handle(Input::Character('a')), Outcome::Selected("save"));
//! // In your draw callback: frame.render_widget(&hop, frame.area());
//! # Ok::<(), ratahop_ratatui::HintError>(())
//! ```
pub use ratahop::{Alphabet, HintError, Input, Outcome, Session, Target};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Clear, Widget},
};
use unicode_width::UnicodeWidthStr;

pub fn rect(area: Rect) -> ratahop::Rect {
    ratahop::Rect::new(area.x, area.y, area.width, area.height)
}

/// Styles can be replaced with monochrome or application-specific themes.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub badge: Style,
    pub typed: Style,
    pub invalid: Style,
    pub picker: Style,
}
impl Default for Theme {
    fn default() -> Self {
        Self {
            badge: Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            typed: Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::UNDERLINED),
            invalid: Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            picker: Style::default().fg(Color::White).bg(Color::Black),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Badge {
    pub hint_index: usize,
    pub area: Rect,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layout {
    Badges(Vec<Badge>),
    Picker,
}

/// Stateful adapter. Target snapshots must be updated before handling input.
pub struct Hop<Id> {
    session: Session<Id>,
    targets: Vec<Target<Id>>,
    viewport: Rect,
    scope: Option<u64>,
    page: usize,
    pub theme: Theme,
}
impl<Id> Default for Hop<Id> {
    fn default() -> Self {
        Self::new(Alphabet::default())
    }
}
impl<Id> Hop<Id> {
    pub fn new(alphabet: Alphabet) -> Self {
        Self {
            session: Session::new(alphabet),
            targets: vec![],
            viewport: Rect::default(),
            scope: None,
            page: 0,
            theme: Theme::default(),
        }
    }
    pub fn session(&self) -> &Session<Id> {
        &self.session
    }
    pub fn is_active(&self) -> bool {
        self.session.is_active()
    }
    pub fn cancel(&mut self) {
        self.session.cancel();
        self.page = 0;
    }
    /// Browse the crowded-layout picker with any event backend.
    pub fn next_page(&mut self) {
        let rows = usize::from(self.viewport.height.saturating_sub(2).max(1));
        let max_page = self.session.matches().count().saturating_sub(1) / rows;
        self.page = self.page.saturating_add(1).min(max_page);
    }
    pub fn previous_page(&mut self) {
        self.page = self.page.saturating_sub(1);
    }
    /// Place labels within their control rectangle. Fall back to a paged picker
    /// if any label cannot fit without covering another label.
    pub fn layout(&self) -> Layout {
        let mut badges: Vec<Badge> = vec![];
        for (hint_index, hint) in self
            .session
            .hints()
            .iter()
            .enumerate()
            .filter(|(_, h)| h.label.starts_with(self.session.prefix()))
        {
            let area = hint.target.area;
            let width = u16::try_from(hint.label.len()).unwrap_or(u16::MAX);
            if width > area.width {
                return Layout::Picker;
            }
            let right = area.right().saturating_sub(width);
            let mut candidates = vec![];
            if let Some((x, y)) = hint.target.anchor
                && x >= area.x
                && x <= right
                && y >= area.y
                && y < area.bottom()
            {
                candidates.push((x, y));
            }
            // Deterministic top-left, top-right, bottom-left, bottom-right anchors.
            candidates.extend([
                (area.x, area.y),
                (right, area.y),
                (area.x, area.bottom() - 1),
                (right, area.bottom() - 1),
            ]);
            let position = candidates
                .into_iter()
                .map(|(x, y)| Rect::new(x, y, width, 1))
                .find(|r| badges.iter().all(|b| !r.intersects(b.area)));
            match position {
                Some(area) => badges.push(Badge { hint_index, area }),
                None => return Layout::Picker,
            }
        }
        Layout::Badges(badges)
    }
}
impl<Id: Clone + Eq> Hop<Id> {
    /// Synchronize the latest layout. Returns true if an active session was
    /// cancelled because geometry, membership, order, title, or scope changed.
    pub fn update(&mut self, targets: Vec<Target<Id>>, viewport: Rect, scope: Option<u64>) -> bool {
        let cancelled = self.session.refresh(&targets, rect(viewport), scope);
        self.targets = targets;
        self.viewport = viewport;
        self.scope = scope;
        if cancelled {
            self.page = 0;
        }
        cancelled
    }
    pub fn start(&mut self) -> Result<(), HintError> {
        self.page = 0;
        self.session
            .start(&self.targets, rect(self.viewport), self.scope)
    }
    pub fn handle(&mut self, input: Input) -> Outcome<Id> {
        let result = self.session.handle(input);
        self.page = 0;
        result
    }
    /// Crossterm releases are ignored; repeats are consumed but cannot select a
    /// second control. Modified characters are not interpreted as hint letters.
    #[cfg(feature = "crossterm")]
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Outcome<Id> {
        use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
        if !self.is_active() {
            return Outcome::Ignored;
        }
        if key.kind != KeyEventKind::Press {
            return Outcome::Pending;
        }
        if key.modifiers.intersects(
            KeyModifiers::CONTROL
                | KeyModifiers::ALT
                | KeyModifiers::SUPER
                | KeyModifiers::HYPER
                | KeyModifiers::META,
        ) {
            return self.handle(Input::Other);
        }
        match key.code {
            KeyCode::PageDown | KeyCode::Down => {
                self.next_page();
                Outcome::Pending
            }
            KeyCode::PageUp | KeyCode::Up => {
                self.previous_page();
                Outcome::Pending
            }
            KeyCode::Char(c) => self.handle(Input::Character(c)),
            KeyCode::Esc => self.handle(Input::Escape),
            KeyCode::Backspace => self.handle(Input::Backspace),
            _ => self.handle(Input::Other),
        }
    }
}

/// Clear entire wide glyphs touched by a badge, including the leading cell of
/// a glyph that begins before the badge. All coordinates are buffer-relative.
fn clear_badge(buf: &mut Buffer, area: Rect) {
    let row = area.y;
    let mut touched = vec![];
    for x in buf.area.x..buf.area.right() {
        let width = buf[(x, row)]
            .symbol()
            .width()
            .max(1)
            .min(usize::from(u16::MAX)) as u16;
        let end = x.saturating_add(width).min(buf.area.right());
        if x < area.right() && end > area.x {
            touched.extend(x..end);
        }
    }
    for x in touched {
        buf[(x, row)].reset();
    }
}
fn paint_label(
    buf: &mut Buffer,
    area: Rect,
    label: &str,
    prefix_len: usize,
    theme: Theme,
    invalid: bool,
) {
    clear_badge(buf, area);
    for (i, c) in label.chars().enumerate().take(usize::from(area.width)) {
        let style = if invalid {
            theme.invalid
        } else if i < prefix_len {
            theme.typed
        } else {
            theme.badge
        };
        buf[(area.x + i as u16, area.y)]
            .set_char(c)
            .set_style(style);
    }
}
impl<Id> Widget for &Hop<Id> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.is_active() {
            return;
        }
        let area = area.intersection(buf.area);
        let viewport = self.viewport.intersection(area);
        if viewport.is_empty() {
            return;
        }
        // Rendering a different viewport without synchronizing must not show
        // stale coordinates. Host should call update to cancel the session.
        if viewport != self.viewport {
            return;
        }
        let prefix_len = self.session.prefix().len();
        match self.layout() {
            Layout::Badges(badges) => {
                for badge in badges {
                    let label = &self.session.hints()[badge.hint_index].label;
                    paint_label(
                        buf,
                        badge.area,
                        label,
                        prefix_len,
                        self.theme,
                        self.session.invalid_input(),
                    );
                }
            }
            Layout::Picker => {
                Clear.render(viewport, buf);
                Block::new().style(self.theme.picker).render(viewport, buf);
                let matches: Vec<_> = self.session.matches().collect();
                let rows = usize::from(viewport.height.saturating_sub(2).max(1));
                let pages = matches.len().div_ceil(rows).max(1);
                let page = self.page.min(pages - 1);
                let status = if self.session.invalid_input() {
                    "Invalid key · "
                } else {
                    ""
                };
                buf.set_stringn(
                    viewport.x,
                    viewport.y,
                    format!(
                        "{status}RataHop {}/{} · {} targets",
                        page + 1,
                        pages,
                        matches.len()
                    ),
                    viewport.width as usize,
                    self.theme.picker.add_modifier(Modifier::BOLD),
                );
                if viewport.height < 2 {
                    return;
                }
                for (row, hint) in matches.iter().skip(page * rows).take(rows).enumerate() {
                    let y = viewport.y + 1 + row as u16;
                    if y >= viewport.bottom() - 1 {
                        break;
                    }
                    let title: String = hint
                        .target
                        .title
                        .chars()
                        .filter(|c| !c.is_control())
                        .collect();
                    let label_width = u16::try_from(hint.label.len())
                        .unwrap_or(u16::MAX)
                        .min(viewport.width);
                    paint_label(
                        buf,
                        Rect::new(viewport.x, y, label_width, 1),
                        &hint.label,
                        prefix_len,
                        self.theme,
                        self.session.invalid_input(),
                    );
                    let text_x = viewport.x.saturating_add(label_width).saturating_add(1);
                    if text_x < viewport.right() {
                        buf.set_stringn(
                            text_x,
                            y,
                            title,
                            usize::from(viewport.right() - text_x),
                            self.theme.picker,
                        );
                    }
                }
                buf.set_stringn(
                    viewport.x,
                    viewport.bottom() - 1,
                    "Type label · PgUp/PgDn · Esc",
                    viewport.width as usize,
                    self.theme.picker,
                );
            }
        }
    }
}
