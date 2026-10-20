use ratahop::{Alphabet, HintError, Input, Outcome, Rect, Session, Target};
use std::collections::HashSet;
fn targets(count: usize) -> Vec<Target<usize>> {
    (0..count)
        .map(|i| Target::new(i, Rect::new(1, 1, 10, 1), format!("Control {i}")))
        .collect()
}
#[test]
fn label_properties_across_alphabets_and_boundaries() {
    for alphabet in ["ab", "asdfghjkl", "0123456789", "AZ!?"] {
        let alphabet = Alphabet::new(alphabet).unwrap();
        for count in (0..=150).chain([255, 256, 257, 999, 1000]) {
            let labels = alphabet.labels(count);
            assert_eq!(labels.len(), count);
            assert_eq!(labels, alphabet.labels(count));
            assert_eq!(labels.iter().collect::<HashSet<_>>().len(), count);
            let mut sorted = labels.clone();
            sorted.sort();
            assert!(sorted.windows(2).all(|w| !w[1].starts_with(&w[0])));
            for (id, label) in labels.iter().enumerate() {
                // Feed the assigned label through the public state machine.
                if count > 150 && id % 97 != 0 {
                    continue;
                }
                let mut session = Session::new(alphabet.clone());
                session
                    .start(&targets(count), Rect::new(0, 0, 80, 24), None)
                    .unwrap();
                for (i, c) in label.chars().enumerate() {
                    let out = session.handle(Input::Character(c));
                    if i + 1 == label.len() {
                        assert_eq!(out, Outcome::Selected(id));
                    } else {
                        assert_eq!(out, Outcome::Pending);
                    }
                }
                assert!(!session.is_active());
            }
        }
    }
}
#[test]
fn validation_and_failed_restart_leave_no_active_session() {
    for value in ["", "a", "aa", "ab ", "a\n", "éx", "界a"] {
        assert!(Alphabet::new(value).is_err(), "{value:?}");
    }
    let mut session = Session::default();
    session
        .start(&targets(2), Rect::new(0, 0, 80, 24), None)
        .unwrap();
    assert_eq!(
        session.start(&targets(0), Rect::new(0, 0, 80, 24), None),
        Err(HintError::NoTargets)
    );
    assert!(!session.is_active());
    let duplicate = vec![targets(1).remove(0), targets(1).remove(0)];
    assert_eq!(
        session.start(&duplicate, Rect::new(0, 0, 80, 24), None),
        Err(HintError::DuplicateId)
    );
    assert_eq!(session.handle(Input::Character('a')), Outcome::Ignored);
}
#[test]
fn scopes_clipping_and_changes_invalidate_snapshots() {
    let viewport = Rect::new(10, 10, 20, 10);
    let mut entries = vec![
        Target::new(0, Rect::new(8, 9, 5, 3), "Partial").scope(1),
        Target::new(0, Rect::new(10, 10, 5, 1), "Other scope").scope(2),
        Target::new(2, Rect::new(80, 80, 1, 1), "Outside").scope(1),
    ];
    let mut session = Session::default();
    session.start(&entries, viewport, Some(1)).unwrap();
    assert_eq!(session.hints().len(), 1);
    assert_eq!(session.hints()[0].target.area, Rect::new(10, 10, 3, 2));
    assert!(!session.refresh(&entries, viewport, Some(1)));
    entries[0].title = "Changed action".into();
    assert!(session.refresh(&entries, viewport, Some(1)));
    assert!(!session.is_active());
    session.start(&entries, viewport, Some(1)).unwrap();
    assert!(session.refresh(&entries, viewport, Some(2)));
    session.start(&entries, viewport, Some(1)).unwrap();
    assert!(session.refresh(&entries, Rect::new(10, 10, 21, 10), Some(1)));
    session.start(&entries, viewport, Some(1)).unwrap();
    entries.remove(0);
    assert!(session.refresh(&entries, viewport, Some(1)));
}
#[test]
fn prefix_invalid_input_backspace_and_cancellation() {
    let mut session = Session::new(Alphabet::new("ab").unwrap());
    session
        .start(&targets(8), Rect::new(0, 0, 80, 24), None)
        .unwrap();
    assert_eq!(session.handle(Input::Character('a')), Outcome::Pending);
    assert_eq!(session.prefix(), "a");
    assert_eq!(session.handle(Input::Character('z')), Outcome::Invalid);
    assert!(session.invalid_input());
    assert_eq!(session.prefix(), "a");
    assert_eq!(session.handle(Input::Backspace), Outcome::Pending);
    assert_eq!(session.prefix(), "");
    assert!(!session.invalid_input());
    assert_eq!(session.handle(Input::Other), Outcome::Pending);
    assert_eq!(session.handle(Input::Escape), Outcome::Cancelled);
    assert_eq!(session.handle(Input::Other), Outcome::Ignored);
    assert!(!Outcome::<usize>::Ignored.consumed());
    assert!(Outcome::<usize>::Invalid.consumed());
}
#[test]
fn extreme_rectangles_are_saturating() {
    let r = Rect::new(u16::MAX - 1, u16::MAX - 1, 10, 10);
    assert_eq!(r.right(), u16::MAX);
    assert_eq!(r.intersection(Rect::new(0, 0, 80, 24)).width, 0);
}
