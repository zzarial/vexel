use vexel::{Element, ElementCode, AppState, Style, Event};
use vexel::render::Renderer;
use vexel::events::{poll_event, read_event, dispatch};
use std::time::Duration;
use std::io;
use crossterm::{terminal::{disable_raw_mode}, cursor::Show, event::{EnableMouseCapture, DisableMouseCapture}, ExecutableCommand};

fn main() -> io::Result<()> {
    let mut app = AppState::new();
    let mut renderer = Renderer::new();
    let mut stdout = io::stdout();
    
    //let element ...
    //app.push_child(element);

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

    // вернуть терминал в норму
    disable_raw_mode()?;
    stdout.execute(Show)?;
    stdout.execute(DisableMouseCapture)?;
    Ok(())
}
