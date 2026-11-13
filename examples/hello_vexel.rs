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

    let style_button = Style { width: 20, height: 3, ..Default::default() };
    let code_button = ElementCode {
        on_click: Some(Box::new(|_event, app: &mut AppState| {
            app.delete_element(1);
            app.cursor = 0;
            app.push_child(Element::text("msg", "Hello Vexel!", Style::default()));
        })),
        on_key: None,
        on_render: None,
    };
    let mut button = Element::button("hello", "hello", style_button);
    button.code = Some(code_button);
    app.push_child(button);

    // события → диспетчер → кадр
    stdout.execute(EnableMouseCapture)?;// без этого мышь не рабоатет
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
