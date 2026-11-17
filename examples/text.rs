// тест Style: color / background / border / атрибуты
// cargo run --example text

use std::io;
use std::time::Duration;
use vexel::{AppState, Event, Element, Style, Color, NamedColor, Rgb};
use vexel::render::Renderer;
use vexel::events::{poll_event, read_event, dispatch};
use crossterm::{cursor::Show, event::{EnableMouseCapture, DisableMouseCapture}, terminal::disable_raw_mode, ExecutableCommand};

fn main() -> io::Result<()> {
    let mut app = AppState::new();
    let mut renderer = Renderer::new();
    let mut stdout = io::stdout();

    // ─── 1. Style.color: каждая строка другим цветом ───
    let colors = [
        ("Red", Color::Named(NamedColor::Red)),
        ("Green", Color::Named(NamedColor::Green)),
        ("Yellow", Color::Named(NamedColor::Yellow)),
        ("Blue", Color::Named(NamedColor::Blue)),
        ("Cyan", Color::Named(NamedColor::Cyan)),
        ("BrightWhite", Color::Named(NamedColor::BrightWhite)),
        ("Rgb(255,128,0)", Color::Rgb(Rgb { r: 255, g: 128, b: 0 })),
    ];
    for (i, (name, color)) in colors.iter().enumerate() {
        app.push_child(Element::text("color_test", name, Style {
            x: 1, y: i as i16,
            width: 20, height: 1,
            color: *color,
            ..Default::default()
        }));
    }

    // ─── 2. Style.background: заливка всей ширины ───
    let bgs = [
        ("blue bg", Color::Named(NamedColor::Blue)),
        ("green bg", Color::Named(NamedColor::Green)),
        ("Rgb(60,60,70) bg", Color::Rgb(Rgb { r: 60, g: 60, b: 70 })),
    ];
    for (i, (name, bg)) in bgs.iter().enumerate() {
        app.push_child(Element::text("bg_test", name, Style {
            x: 25, y: i as i16,
            width: 20, height: 1,
            background: Some(*bg),
            ..Default::default()
        }));
    }

    // ─── 3. color + background вместе ───
    app.push_child(Element::text("both_test", "white on red", Style {
        x: 25, y: 4,
        width: 20, height: 1,
        color: Color::Named(NamedColor::White),
        background: Some(Color::Named(NamedColor::Red)),
        ..Default::default()
    }));

    // ─── 4. Style.border: box с рамкой ───
    app.push_child(Element::container("border_test", Style {
        x: 1, y: 9,
        width: 20, height: 5,
        color: Color::Named(NamedColor::Cyan),
        border: Some(vexel::Border { width: 1, color: Color::Named(NamedColor::Red) }),
        ..Default::default()
    }));

    // ─── 5. border + background: рамка на залитом фоне ───
    app.push_child(Element::container("fill_test", Style {
        x: 25, y: 9,
        width: 20, height: 5,
        color: Color::Named(NamedColor::White),
        background: Some(Color::Rgb(Rgb { r: 40, g: 40, b: 50 })),
        border: Some(vexel::Border { width: 1, color: Color::Named(NamedColor::Yellow) }),
        ..Default::default()
    }));

    // ─── 6. атрибуты через разметку ───
    app.push_child(Element::text("attr_test", "**bold** *italic* __underline__", Style {
        x: 1, y: 16,
        width: 44, height: 1,
        color: Color::Named(NamedColor::White),
        ..Default::default()
    }));

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
