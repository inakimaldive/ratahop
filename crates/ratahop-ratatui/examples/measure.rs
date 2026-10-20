//! Small reproducible timing probe; not a statistical benchmark suite.
use ratahop_ratatui::{Alphabet, Hop, Target, rect};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use std::{hint::black_box, time::Instant};
fn main() {
    for count in [10, 100, 1000] {
        let viewport = Rect::new(0, 0, 150, 54);
        let targets: Vec<_> = (0..count)
            .map(|id| Target::new(id, rect(Rect::new(0, 0, 10, 1)), format!("Target {id}")))
            .collect();
        let start = Instant::now();
        for _ in 0..100 {
            black_box(Alphabet::default().labels(count));
        }
        let allocation = start.elapsed() / 100;
        let mut hop = Hop::default();
        hop.update(targets, viewport, None);
        let start = Instant::now();
        for _ in 0..100 {
            hop.start().unwrap();
            black_box(hop.session());
        }
        let session = start.elapsed() / 100;
        let mut buffer = Buffer::empty(viewport);
        let start = Instant::now();
        for _ in 0..100 {
            (&hop).render(viewport, &mut buffer);
            black_box(&buffer);
        }
        println!(
            "{count} targets: labels {allocation:?}, start {session:?}, crowded render {:?}",
            start.elapsed() / 100
        );
    }
}
