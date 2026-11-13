use std::time::Duration;
use crossterm::event::{self, Event as CTEvent, KeyCode, MouseEventKind, MouseButton};

use crate::{AppState, Event};


pub fn poll_event(timeout: Duration) -> bool {
    event::poll(timeout).unwrap_or(false)
}
//переводит crossterm event type в lib/Event 
pub fn read_event() -> Option<Event> {
    match event::read().ok()? {
        // --- клавиатура ---
        CTEvent::Key(key) => match key.code {
            KeyCode::Char(c) => Some(Event::Key(c)),
            //TODO остальные символы
            KeyCode::Enter => Some(Event::Enter),
            KeyCode::Tab => Some(Event::Tab),
            KeyCode::Esc => Some(Event::Escape),
            _ => None,
        },
        // --- мышь ---
        CTEvent::Mouse(m) => match m.kind {
            MouseEventKind::Up(MouseButton::Left) => {
                Some(Event::Click { x: m.column, y: m.row })
            }
            _ => None, // Down/Drag/Moved/Scroll затычка
        },
        _ => None, // Focus/Resize/Paste затычка
    }
}

//диспетчер: берёт событие и решает, какому элементу его отдать
pub fn dispatch(event: Event, app: &mut AppState, r: &crate::render::Renderer) {
    match event {
        // событие идет в focus el
        Event::Key(_) | Event::Enter | Event::Tab | Event::Escape => {
            let target = app.focused;
            call_on_key(app, target, event);
        }
        // найти элемент клика, сфокусирвоать, вызвать on_click
        Event::Click { x, y } => {
            if let Some(target) = r.hit_test(x, y) {
                app.focused = target;  
                call_on_click(app, target, event);
            }
        }
    }
}

//вызов кулбеков
fn call_on_key(app: &mut AppState, target: usize, event: Event) {
    let mut cb: Option<Box<dyn Fn(Event, &mut AppState)>> = None;// есть ли code и on_key?
    if let Some(code) = app.elements[target].code.as_mut() {//  колбэк лежит внтури app, а вызывать его надо С app.
        cb = code.on_key.take(); //  держать ссылку и мутировать нельзя. take на время. 
    }

    if let Some(cb) = cb {
        cb(event, app);
        // вернуть колбэк обратно
        if let Some(code) = app.elements[target].code.as_mut() {
            code.on_key = Some(cb);
        }
    }
}

fn call_on_click(app: &mut AppState, target: usize, event: Event) {
    let mut cb: Option<Box<dyn Fn(Event, &mut AppState)>> = None;

    if let Some(code) = app.elements[target].code.as_mut() {
        cb = code.on_click.take();
    }

    if let Some(cb) = cb {
        cb(event, app);

        if let Some(code) = app.elements[target].code.as_mut() {
            code.on_click = Some(cb);
        }
    }
}
