//! Run with `cargo run -p ratahop-ratatui --example showcase -- form|table|split`.
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratahop_ratatui::{Hop, Outcome, Target, rect};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Clear, Paragraph},
};
use std::{io, time::Duration};
#[derive(Debug, Clone, PartialEq, Eq)]
enum Action {
    Edit,
    Save,
    Reset,
    Row(usize),
    Close,
}
struct App {
    hop: Hop<Action>,
    mode: String,
    name: String,
    status: String,
    scroll: usize,
    dialog: bool,
}
impl App {
    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let regions = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(area);
        frame.render_widget(
            Paragraph::new(format!(
                "RataHop / {} · f hints · j/k scroll · q quit",
                self.mode
            )),
            regions[0],
        );
        let mut targets = vec![];
        let mut button = |frame: &mut Frame, action: Action, bounds: Rect, title: &str| {
            frame.render_widget(
                Paragraph::new(title).style(Style::default().fg(Color::Cyan)),
                bounds,
            );
            targets.push(Target::new(action, rect(bounds), title));
        };
        if self.mode == "form" {
            let rows = Layout::vertical([
                Constraint::Length(3),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(0),
            ])
            .split(regions[1]);
            button(
                frame,
                Action::Edit,
                rows[0],
                &format!("Name: {} [edit]", self.name),
            );
            button(frame, Action::Save, rows[1], "Save profile");
            button(frame, Action::Reset, rows[2], "Reset profile");
        } else {
            let panes = Layout::horizontal([
                Constraint::Percentage(if self.mode == "split" { 30 } else { 0 }),
                Constraint::Min(1),
            ])
            .split(regions[1]);
            if self.mode == "split" {
                button(frame, Action::Edit, panes[0], "Settings [edit name]");
            }
            frame.render_widget(
                Block::bordered().title("100 rows · only visible rows get hints"),
                panes[1],
            );
            let inner = panes[1].inner(ratatui::layout::Margin::new(1, 1));
            for (row, id) in (self.scroll..100)
                .take(usize::from(inner.height))
                .enumerate()
            {
                button(
                    frame,
                    Action::Row(id),
                    Rect::new(inner.x, inner.y + row as u16, inner.width, 1),
                    &format!("Item {:03} · read details", id + 1),
                );
            }
        }
        frame.render_widget(Paragraph::new(self.status.as_str()), regions[2]);
        if self.dialog {
            targets.clear();
            let w = area.width.min(56);
            let h = area.height.min(9);
            let popup = Rect::new(
                area.x + (area.width - w) / 2,
                area.y + (area.height - h) / 2,
                w,
                h,
            );
            frame.render_widget(Clear, popup);
            frame.render_widget(
                Block::bordered().title("Edit name · Alt+f hints · Esc close"),
                popup,
            );
            let inner = popup.inner(ratatui::layout::Margin::new(1, 1));
            frame.render_widget(Paragraph::new(format!("{}▏", self.name)), inner);
            if inner.height >= 3 {
                let half = inner.width / 2;
                let y = inner.bottom() - 1;
                frame.render_widget(Paragraph::new("Save"), Rect::new(inner.x, y, half, 1));
                frame.render_widget(
                    Paragraph::new("Close"),
                    Rect::new(inner.x + half, y, inner.width - half, 1),
                );
                targets.push(Target::new(
                    Action::Save,
                    rect(Rect::new(inner.x, y, half, 1)),
                    "Save profile",
                ));
                targets.push(Target::new(
                    Action::Close,
                    rect(Rect::new(inner.x + half, y, inner.width - half, 1)),
                    "Close dialog",
                ));
            }
        }
        self.hop.update(targets, area, None);
        frame.render_widget(&self.hop, area);
    }
    fn act(&mut self, action: Action) {
        match action {
            Action::Edit => self.dialog = true,
            Action::Save => {
                self.status = format!("Saved profile: {}", self.name);
                self.dialog = false;
            }
            Action::Reset => {
                self.name = "Terminal reader".into();
                self.status = "Reset profile".into();
            }
            Action::Close => self.dialog = false,
            Action::Row(id) => self.status = format!("Selected stable row ID {id}"),
        }
    }
}
fn main() -> io::Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "form".into());
    if !["form", "table", "split"].contains(&mode.as_str()) {
        return Err(io::Error::other("choose form, table, or split"));
    }
    let mut app = App {
        hop: Hop::default(),
        mode,
        name: "Terminal reader".into(),
        status: "Type a visible hint to activate its control. PgUp/PgDn browse crowded pickers."
            .into(),
        scroll: 0,
        dialog: false,
    };
    let mut terminal = ratatui::init();
    let result = (|| -> io::Result<()> {
        loop {
            terminal.draw(|frame| app.draw(frame))?;
            if !event::poll(Duration::from_millis(100))? {
                continue;
            }
            match event::read()? {
                Event::Resize(_, _) => app.hop.cancel(),
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }
                    let outcome = app.hop.handle_key(key);
                    if let Outcome::Selected(action) = &outcome {
                        app.act(action.clone());
                    }
                    if outcome.consumed() {
                        continue;
                    }
                    if key.code == KeyCode::Char('f')
                        && (key.modifiers == KeyModifiers::ALT
                            || (!app.dialog && key.modifiers.is_empty()))
                    {
                        if let Err(error) = app.hop.start() {
                            app.status = error.to_string();
                        }
                    } else if app.dialog {
                        match key.code {
                            KeyCode::Esc => app.dialog = false,
                            KeyCode::Backspace => {
                                app.name.pop();
                            }
                            KeyCode::Char(c)
                                if key.modifiers.is_empty()
                                    || key.modifiers == KeyModifiers::SHIFT =>
                            {
                                app.name.push(c)
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('j') | KeyCode::Down => {
                                app.scroll = (app.scroll + 1).min(99)
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                app.scroll = app.scroll.saturating_sub(1)
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    })();
    ratatui::restore();
    result
}
