// шаблон приложения на vexel
// cargo run --example app
mod components;

use std::io;
use std::time::Duration;
use vexel::{AppState, Event};
use vexel::render::Renderer;
use vexel::events::{poll_event, read_event, dispatch};
use crossterm::{cursor::Show, event::{EnableMouseCapture, DisableMouseCapture}, terminal::disable_raw_mode, ExecutableCommand};

fn main() -> io::Result<()> {
    let mut app = AppState::new();
    let mut renderer = Renderer::new();
    let mut stdout = io::stdout();

    app.push_child(components::output());
    app.push_sibling(components::input_box());

    stdout.execute(EnableMouseCapture)?;
    loop {
        if poll_event(Duration::from_millis(16)) {
            if let Some(ev) = read_event() {
                match ev {
                    Event::Escape => break,
                    _ => dispatch(ev, &mut app, &renderer),
                }
            }
        }
        renderer.render(&app)?;
    }

    disable_raw_mode()?;
    stdout.execute(Show)?;
    stdout.execute(DisableMouseCapture)?;
    Ok(())
}
