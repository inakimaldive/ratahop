use ratahop::{Input, Outcome, Rect, Session, Target};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session = Session::default();
    session.start(
        &[
            Target::new("save", Rect::new(1, 1, 8, 1), "Save"),
            Target::new("close", Rect::new(10, 1, 8, 1), "Close"),
        ],
        Rect::new(0, 0, 80, 24),
        None,
    )?;
    for hint in session.hints() {
        println!("{}: {}", hint.label, hint.target.title);
    }
    if let Outcome::Selected(id) = session.handle(Input::Character('a')) {
        println!("Selected: {id}");
    }
    Ok(())
}
