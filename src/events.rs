use std::time::Duration;
use crossterm::event::{self, Event as CTEvent, KeyCode, MouseEventKind, MouseButton};

use crate::{AppState, Event};


pub fn poll_event(timeout: Duration) -> bool {
    event::poll(timeout).unwrap_or(false)
}

pub fn read_event() -> Option<Event> {
    match event::read().ok()? {
        // --- клавиатура ---
        CTEvent::Key(key) => match key.code {
            KeyCode::Char(c) => Some(Event::Key(c)),
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
    let cb = app.elements[target]
        .code
        .as_mut()                    // Option<ElementCode> -> Option<&mut ElementCode>
        .and_then(|c| c.on_key.as_mut()); // есть ли on_key?
    if let Some(cb) = cb {
        // нельзя: cb(event, app) пока app занят borrow'ом выше
        // решение: забрать колбэк на время (mem::take), вернуть после
        let taken = std::mem::take(cb);
        taken(event, app);
        // вернуть обратно не выйдет: FnOnce потребил колбэк.
        // поэтому у on_key должен быть другой путь - см. вопросы ниже
    }
}

fn call_on_click(app: &mut AppState, target: usize, event: Event) {
    //TODO 
    let _ = (target, event);
}
