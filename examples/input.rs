// cargo run --example input
use std::io;
use std::time::Duration;
use vexel::{Element, ElementCode, AppState, Style, Event, ElementData};
use vexel::render::Renderer;
use vexel::events::{poll_event, read_event, dispatch};
use crossterm::{cursor::Show, event::{EnableMouseCapture, DisableMouseCapture}, terminal::disable_raw_mode, ExecutableCommand};

fn main() -> io::Result<()> {
    let mut app = AppState::new();
    let mut renderer = Renderer::new();
    let mut stdout = io::stdout();

    let input_style = Style { x: 2, y: 1, width: 20, height: 3, ..Default::default() };
    let mut input = Element::input("in", "пиши сюда", input_style);
    input.code = Some(ElementCode {
        on_key: Some(Box::new(|event, app: &mut AppState| {
            match event {
                Event::Key(c) => {
                    if let ElementData::InputData { value, .. } = &mut app.elements[1].data {
                        value.push(c);
                    }
                }
                Event::Backspace => {
                    if let ElementData::InputData { value, .. } = &mut app.elements[1].data {
                        value.pop();
                    }
                }
                Event::Enter => {
                    let mut value = String::new();
                    if let ElementData::InputData { value: v, .. } = &mut app.elements[1].data {
                        value = std::mem::take(v);
                    }
                    if !value.is_empty() {
                        let n = app.elements[0].children.len() as u16;
                        let echo = Element::text(
                            "echo",
                            &value,
                            Style { x: 2, y: (4 + (n - 1) * 2) as i16, ..Default::default() },
                        );
                        app.push_sibling(echo);
                    }
                }
                _ => {}
            }
        })),
        on_render: None,
        on_click: None,
    });
    app.push_child(input);

    app.focused = app.cursor;//auto focus

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
