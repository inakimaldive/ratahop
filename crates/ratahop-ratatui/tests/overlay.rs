use ratahop_ratatui::{Alphabet, Hop, Input, Layout, Outcome, Target, rect};
use ratatui::{buffer::Buffer, layout::Rect, style::Modifier, widgets::Widget};
fn target(id: usize, area: Rect) -> Target<usize> {
    Target::new(id, rect(area), format!("Control {id}"))
}
#[test]
fn badges_have_expected_cells_and_respect_nonzero_origin() {
    let area = Rect::new(10, 5, 30, 10);
    let mut hop = Hop::default();
    hop.update(
        vec![
            target(1, Rect::new(12, 6, 10, 1)),
            target(2, Rect::new(12, 8, 10, 1)),
        ],
        area,
        None,
    );
    hop.start().unwrap();
    let mut buffer = Buffer::empty(area);
    (&hop).render(area, &mut buffer);
    assert_eq!(buffer[(12, 6)].symbol(), "a");
    assert_eq!(buffer[(12, 8)].symbol(), "s");
    assert_eq!(buffer[(11, 6)].symbol(), " ");
    assert_eq!(hop.handle(Input::Character('s')), Outcome::Selected(2));
}
#[test]
fn wide_glyph_crossing_badge_boundary_is_cleared() {
    let area = Rect::new(0, 0, 8, 3);
    let mut hop = Hop::default();
    hop.update(vec![target(0, Rect::new(1, 0, 1, 1))], area, None);
    hop.start().unwrap();
    let mut buffer = Buffer::empty(area);
    buffer.set_string(0, 0, "界hello", ratatui::style::Style::default());
    (&hop).render(area, &mut buffer);
    assert_eq!(buffer[(0, 0)].symbol(), " ");
    assert_eq!(buffer[(1, 0)].symbol(), "a");
    assert_eq!(buffer[(2, 0)].symbol(), "h");
}
#[cfg(feature = "crossterm")]
#[test]
fn crowded_picker_pages_and_still_selects_offscreen_targets() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let area = Rect::new(0, 0, 40, 5);
    let mut hop = Hop::default();
    hop.update(
        (0..12)
            .map(|id| target(id, Rect::new(0, 0, 1, 1)))
            .collect(),
        area,
        None,
    );
    hop.start().unwrap();
    assert_eq!(hop.layout(), Layout::Picker);
    let mut buffer = Buffer::empty(area);
    (&hop).render(area, &mut buffer);
    let text = |b: &Buffer| b.content.iter().map(|c| c.symbol()).collect::<String>();
    assert!(text(&buffer).contains("RataHop 1/4"));
    assert!(text(&buffer).contains("Control 0"));
    hop.handle_key(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE));
    (&hop).render(area, &mut buffer);
    assert!(text(&buffer).contains("RataHop 2/4"));
    assert!(text(&buffer).contains("Control 3"));
    let label = hop.session().hints()[11].label.clone();
    let mut outcome = Outcome::Pending;
    for c in label.chars() {
        outcome = hop.handle(Input::Character(c));
    }
    assert_eq!(outcome, Outcome::Selected(11));
}
#[test]
fn typed_prefix_is_underlined_and_invalid_input_is_visible() {
    let area = Rect::new(0, 0, 30, 10);
    let mut hop = Hop::new(Alphabet::new("ab").unwrap());
    hop.update(
        (0..8)
            .map(|id| target(id, Rect::new(0, id as u16, 10, 1)))
            .collect(),
        area,
        None,
    );
    hop.start().unwrap();
    hop.handle(Input::Character('a'));
    let mut buffer = Buffer::empty(area);
    (&hop).render(area, &mut buffer);
    let Layout::Badges(badges) = hop.layout() else {
        panic!()
    };
    assert!(
        buffer[(badges[0].area.x, badges[0].area.y)]
            .modifier
            .contains(Modifier::UNDERLINED)
    );
    hop.handle(Input::Character('z'));
    (&hop).render(area, &mut buffer);
    assert_eq!(
        buffer[(badges[0].area.x, badges[0].area.y)].bg,
        ratatui::style::Color::Red
    );
}
#[test]
fn tiny_views_clipping_and_resize_are_safe() {
    for width in 1..5 {
        for height in 1..5 {
            let area = Rect::new(5, 5, width, height);
            let mut hop = Hop::default();
            hop.update((0..20).map(|id| target(id, area)).collect(), area, None);
            hop.start().unwrap();
            (&hop).render(area, &mut Buffer::empty(area));
            assert!(hop.update(
                vec![target(0, area)],
                Rect::new(5, 5, width + 1, height),
                None
            ));
            assert!(!hop.is_active());
        }
    }
}
#[cfg(feature = "crossterm")]
#[test]
fn modified_keys_release_and_repeat_do_not_activate() {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    let area = Rect::new(0, 0, 10, 5);
    let mut hop = Hop::default();
    hop.update(vec![target(0, area)], area, None);
    hop.start().unwrap();
    for kind in [KeyEventKind::Release, KeyEventKind::Repeat] {
        let key = KeyEvent::new_with_kind(KeyCode::Char('a'), KeyModifiers::NONE, kind);
        assert!(hop.handle_key(key).consumed());
        assert!(hop.is_active());
    }
    assert_eq!(
        hop.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL)),
        Outcome::Pending
    );
    assert_eq!(
        hop.handle_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE)),
        Outcome::Selected(0)
    );
}

#[test]
fn explicit_anchors_and_collision_alternatives_stay_inside_controls() {
    let viewport = Rect::new(0, 0, 20, 5);
    let bounds = Rect::new(2, 1, 6, 1);
    let mut hop = Hop::default();
    hop.update(
        vec![target(1, bounds).anchor(4, 1), target(2, bounds)],
        viewport,
        None,
    );
    hop.start().unwrap();
    let Layout::Badges(badges) = hop.layout() else {
        panic!("two badges fit")
    };
    assert_eq!(badges[0].area.x, 4);
    assert_eq!(badges[1].area.x, 2);
    assert!(!badges[0].area.intersects(badges[1].area));
    for badge in badges {
        assert_eq!(badge.area.intersection(bounds), badge.area);
    }
}

#[test]
fn different_unsynchronized_render_viewport_does_not_draw_stale_hints() {
    let viewport = Rect::new(0, 0, 20, 5);
    let mut hop = Hop::default();
    hop.update(vec![target(0, viewport)], viewport, None);
    hop.start().unwrap();
    let smaller = Rect::new(0, 0, 10, 5);
    let mut buffer = Buffer::empty(smaller);
    (&hop).render(smaller, &mut buffer);
    assert!(buffer.content.iter().all(|cell| cell.symbol() == " "));
    assert!(hop.update(vec![target(0, smaller)], smaller, None));
    assert_eq!(hop.handle(Input::Character('a')), Outcome::Ignored);
}
