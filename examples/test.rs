// cargo run --example test
//
// Большой тест всей библиотеки vexel: один экран = все возможности сразу.
// Правила при пополнении:
// 1. у каждого элемента уникальный id — ищем по id, а не по индексу,
//    поэтому блоки можно переставлять местами
// 2. absolute — координаты от угла экрана, relative — от контента родителя
//    (контент = внутри рамки и padding)
// 3. новый блок = отдельная build_* функция + комментарий, что именно проверяем
// 4. элемент кладётся туда, где стоит app.cursor: перед вложением ставим cursor на родителя

use std::cell::Cell;
use std::io;
use std::time::Duration;

use crossterm::{
    cursor::Show,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::disable_raw_mode,
    ExecutableCommand,
};

use vexel::events::{dispatch, poll_event, read_event};
use vexel::render::Renderer;
use vexel::{
    clone_element, AppState, Border, Color, Element, ElementCode, ElementData, Event, NamedColor,
    Position, Rgb, Style, TextStyle,
};

// ───────────── хелперы для работы с состоянием ─────────────

// индекс элемента по id. None = такого элемента нет
fn find_id(app: &AppState, id: &str) -> Option<usize> {
    app.elements.iter().position(|el| el.id == id)
}

// показать/скрыть элемент вместе со всеми вложенными
fn set_visible_tree(app: &mut AppState, index: usize, visible: bool) {
    let Some(el) = app.elements.get_mut(index) else { return };
    el.style.visible = visible;
    let children = el.children.clone();
    for child in children {
        set_visible_tree(app, child, visible);
    }
}

// delete_element = visible=false + снятие фокуса. детей гасим руками
fn hide_tree(app: &mut AppState, index: usize) {
    let children = app.elements[index].children.clone();
    app.delete_element(index);
    for child in children {
        hide_tree(app, child);
    }
}

// заменить содержимое текста. update_element принимает элемент целиком
fn set_text(app: &mut AppState, id: &str, text: &str) {
    let Some(index) = find_id(app, id) else { return };
    let mut el = clone_element(&app.elements[index]);
    el.data = ElementData::TextData(text.to_string());
    app.update_element(el);
}

// верхняя строка: последнее событие. по ней видно, что события доходят
fn set_status(app: &mut AppState, text: &str) {
    set_text(app, "status", &format!("event: {text}"));
}

// общий стиль кнопок: рамка, чтобы было видно, где кончается зона клика
fn button_style(x: i16, y: i16, width: u16) -> Style {
    Style {
        x,
        y,
        width,
        height: 3,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightBlack) }),
        ..Default::default()
    }
}

// ───────────── блоки экрана ─────────────

fn build_title(app: &mut AppState) {
    app.cursor = 0;
    app.push_child(Element::text(
        "title",
        "VEXEL BIG TEST | Esc: закрыть модалку / выход | клик по инпуту, потом печатай",
        Style {
            x: 1,
            y: 0,
            width: 118,
            height: 1,
            color: Color::Named(NamedColor::BrightWhite),
            text_style: TextStyle::Bold,
            ..Default::default()
        },
    ));
}

fn build_status(app: &mut AppState) {
    app.cursor = 0;
    app.push_child(Element::text(
        "status",
        "event: -",
        Style {
            x: 1,
            y: 1,
            width: 118,
            height: 1,
            color: Color::Named(NamedColor::BrightBlack),
            ..Default::default()
        },
    ));
}

fn build_colors(app: &mut AppState) {
    // контейнер с рамкой + все 16 NamedColor
    app.cursor = 0;
    app.push_child(Element::container("box_colors", Style {
        x: 1,
        y: 3,
        width: 32,
        height: 11,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::Cyan) }),
        ..Default::default()
    }));

    // cursor на контейнер -> push_child кладёт текст внутрь него
    app.cursor = find_id(app, "box_colors").expect("box_colors");
    app.push_child(Element::text("colors_title", "NamedColor", Style {
        x: 1,
        y: 0,
        width: 28,
        height: 1,
        color: Color::Named(NamedColor::BrightWhite),
        ..Default::default()
    }));

    // (id, подпись, цвет) — левая колонка
    let left = [
        ("color_black", "Black", NamedColor::Black),
        ("color_red", "Red", NamedColor::Red),
        ("color_green", "Green", NamedColor::Green),
        ("color_yellow", "Yellow", NamedColor::Yellow),
        ("color_blue", "Blue", NamedColor::Blue),
        ("color_magenta", "Magenta", NamedColor::Magenta),
        ("color_cyan", "Cyan", NamedColor::Cyan),
        ("color_white", "White", NamedColor::White),
    ];
    for (i, (id, label, color)) in left.iter().enumerate() {
        app.cursor = find_id(app, "box_colors").expect("box_colors");
        app.push_child(Element::text(id, label, Style {
            x: 1,
            y: 1 + i as i16,
            width: 15,
            height: 1,
            color: Color::Named(*color),
            ..Default::default()
        }));
    }

    // правая колонка — bright-версии
    let right = [
        ("color_bright_black", "BrightBlack", NamedColor::BrightBlack),
        ("color_bright_red", "BrightRed", NamedColor::BrightRed),
        ("color_bright_green", "BrightGreen", NamedColor::BrightGreen),
        ("color_bright_yellow", "BrightYellow", NamedColor::BrightYellow),
        ("color_bright_blue", "BrightBlue", NamedColor::BrightBlue),
        ("color_bright_magenta", "BrightMagenta", NamedColor::BrightMagenta),
        ("color_bright_cyan", "BrightCyan", NamedColor::BrightCyan),
        ("color_bright_white", "BrightWhite", NamedColor::BrightWhite),
    ];
    for (i, (id, label, color)) in right.iter().enumerate() {
        app.cursor = find_id(app, "box_colors").expect("box_colors");
        app.push_child(Element::text(id, label, Style {
            x: 17,
            y: 1 + i as i16,
            width: 13,
            height: 1,
            color: Color::Named(*color),
            ..Default::default()
        }));
    }
}

fn build_rgb(app: &mut AppState) {
    // truecolor: цвет символов и заливка через Rgb
    app.cursor = 0;
    app.push_child(Element::container("box_rgb", Style {
        x: 35,
        y: 3,
        width: 32,
        height: 11,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::Yellow) }),
        ..Default::default()
    }));

    let foreground = [
        ("rgb_red", "Rgb(255,80,80)", Rgb { r: 255, g: 80, b: 80 }),
        ("rgb_green", "Rgb(80,255,80)", Rgb { r: 80, g: 255, b: 80 }),
        ("rgb_blue", "Rgb(80,80,255)", Rgb { r: 80, g: 80, b: 255 }),
        ("rgb_yellow", "Rgb(255,255,80)", Rgb { r: 255, g: 255, b: 80 }),
    ];
    for (i, (id, label, rgb)) in foreground.iter().enumerate() {
        app.cursor = find_id(app, "box_rgb").expect("box_rgb");
        app.push_child(Element::text(id, label, Style {
            x: 1,
            y: i as i16,
            width: 28,
            height: 1,
            color: Color::Rgb(*rgb),
            ..Default::default()
        }));
    }

    let backgrounds = [
        ("rgb_bg_1", "background Rgb(60,60,70)", Rgb { r: 60, g: 60, b: 70 }),
        ("rgb_bg_2", "background Rgb(120,0,120)", Rgb { r: 120, g: 0, b: 120 }),
        ("rgb_bg_3", "background Rgb(0,90,90)", Rgb { r: 0, g: 90, b: 90 }),
    ];
    for (i, (id, label, rgb)) in backgrounds.iter().enumerate() {
        app.cursor = find_id(app, "box_rgb").expect("box_rgb");
        app.push_child(Element::text(id, label, Style {
            x: 1,
            y: 4 + i as i16,
            width: 28,
            height: 1,
            background: Some(Color::Rgb(*rgb)),
            ..Default::default()
        }));
    }

    // цвет + фон одновременно
    app.cursor = find_id(app, "box_rgb").expect("box_rgb");
    app.push_child(Element::text("rgb_both", "white on Rgb(120,0,60)", Style {
        x: 1,
        y: 7,
        width: 28,
        height: 1,
        color: Color::Named(NamedColor::BrightWhite),
        background: Some(Color::Rgb(Rgb { r: 120, g: 0, b: 60 })),
        ..Default::default()
    }));
}

fn build_text_styles(app: &mut AppState) {
    // все 8 комбинаций TextStyle (3 флага = 2^3)
    app.cursor = 0;
    app.push_child(Element::container("box_styles", Style {
        x: 1,
        y: 15,
        width: 32,
        height: 10,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightBlack) }),
        ..Default::default()
    }));

    let styles = [
        ("style_normal", "Normal", TextStyle::Normal),
        ("style_bold", "Bold", TextStyle::Bold),
        ("style_italic", "Italic", TextStyle::Italic),
        ("style_underline", "Underline", TextStyle::Underline),
        ("style_bold_italic", "BoldItalic", TextStyle::BoldItalic),
        ("style_bold_underline", "BoldUnderline", TextStyle::BoldUnderline),
        ("style_italic_underline", "ItalicUnderline", TextStyle::ItalicUnderline),
        ("style_all", "BoldItalicUnderline", TextStyle::BoldItalicUnderline),
    ];
    for (i, (id, label, text_style)) in styles.iter().enumerate() {
        app.cursor = find_id(app, "box_styles").expect("box_styles");
        app.push_child(Element::text(id, label, Style {
            x: 1,
            y: i as i16,
            width: 28,
            height: 1,
            text_style: *text_style,
            ..Default::default()
        }));
    }
}

fn build_controls(app: &mut AppState) {
    app.cursor = 0;
    app.push_child(Element::container("box_controls", Style {
        x: 35,
        y: 15,
        width: 32,
        height: 13,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightGreen) }),
        ..Default::default()
    }));

    // 1. кнопка-счётчик: on_click меняет свой же label.
    // Cell нужен потому что Fn не даёт менять захваченные переменные
    let clicks = Cell::new(0u32);
    let mut counter = Element::button("counter", "count: 0", button_style(1, 0, 14));
    counter.code = Some(ElementCode {
        on_click: Some(Box::new(move |_, app: &mut AppState| {
            let n = clicks.get() + 1;
            clicks.set(n);
            let Some(index) = find_id(app, "counter") else { return };
            if let ElementData::ButtonData { label } = &mut app.elements[index].data {
                *label = format!("count: {n}");
            }
        })),
        on_key: None,
        on_render: None,
    });
    app.cursor = find_id(app, "box_controls").expect("box_controls");
    app.push_child(counter);

    // остальные кнопки кладутся через push_sibling:
    // cursor сам уезжает на предыдущего родителя, sibling попадает туда же
    let mut toggle = Element::button("toggle", "toggle red", button_style(16, 0, 14));
    toggle.code = Some(ElementCode {
        on_click: Some(Box::new(|_, app: &mut AppState| {
            let Some(index) = find_id(app, "red_target") else { return };
            if app.elements[index].style.visible {
                hide_tree(app, index);
                set_status(app, "hidden: red_target");
            } else {
                set_visible_tree(app, index, true);
                set_status(app, "shown: red_target");
            }
        })),
        on_key: None,
        on_render: None,
    });
    app.push_sibling(toggle);

    // 2. delete_element: скрывает цель навсегда (вернуть может только toggle)
    let mut remove = Element::button("remove", "remove red", button_style(1, 4, 14));
    remove.code = Some(ElementCode {
        on_click: Some(Box::new(|_, app: &mut AppState| {
            let Some(index) = find_id(app, "red_target") else { return };
            hide_tree(app, index);
            set_status(app, "delete_element: red_target");
        })),
        on_key: None,
        on_render: None,
    });
    app.push_sibling(remove);

    // 3. hit_test + координаты клика из Event::Click
    let mut log_click = Element::button("log_click", "log click", button_style(16, 4, 14));
    log_click.code = Some(ElementCode {
        on_click: Some(Box::new(|event, app: &mut AppState| {
            if let Event::Click { x, y } = event {
                set_status(app, &format!("click x={x} y={y}"));
            }
        })),
        on_key: None,
        on_render: None,
    });
    app.push_sibling(log_click);

    // 4. клавиатура: клик по кнопке -> она в фокусе -> клавиши идут в on_key
    let mut arrows = Element::button("arrows", "up/down", button_style(1, 8, 14));
    arrows.code = Some(ElementCode {
        on_key: Some(Box::new(|event, app: &mut AppState| {
            let text = match event {
                Event::Up => "on_key: Up".to_string(),
                Event::Down => "on_key: Down".to_string(),
                Event::Tab => "on_key: Tab".to_string(),
                Event::Enter => "on_key: Enter".to_string(),
                Event::Backspace => "on_key: Backspace".to_string(),
                Event::Key(c) => format!("on_key: '{c}'"),
                // Esc и Click сюда не доходят: Esc ловит main, Click идёт в on_click
                _ => return,
            };
            let Some(index) = find_id(app, "arrows") else { return };
            if let ElementData::ButtonData { label } = &mut app.elements[index].data {
                *label = text.clone();
            }
            set_status(app, &text);
        })),
        on_click: None,
        on_render: None,
    });
    app.push_sibling(arrows);

    // 5. модалка: создаётся при первом открытии, дальше только show/hide
    let mut open = Element::button("open_modal", "open modal", button_style(16, 8, 14));
    open.code = Some(ElementCode {
        on_click: Some(Box::new(|_, app: &mut AppState| open_modal(app))),
        on_key: None,
        on_render: None,
    });
    app.push_sibling(open);
}

fn open_modal(app: &mut AppState) {
    // уже создавали раньше? тогда просто показать (элементы не удаляются из вектора)
    if let Some(index) = find_id(app, "modal") {
        set_visible_tree(app, index, true);
        app.focused = find_id(app, "modal_close").unwrap_or(0);
        set_status(app, "modal: shown");
        return;
    }

    // absolute = координаты от угла экрана.
    // hit_test идёт с конца rects -> модалка, нарисованная последней, кликается первой
    app.cursor = 0;
    app.push_child(Element::container("modal", Style {
        x: 40,
        y: 10,
        width: 40,
        height: 10,
        position: Position::Absolute,
        background: Some(Color::Rgb(Rgb { r: 30, g: 30, b: 45 })),
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightMagenta) }),
        ..Default::default()
    }));

    app.push_child(Element::text(
        "modal_title",
        "модалка: кликается раньше фона",
        Style {
            x: 1,
            y: 1,
            width: 36,
            height: 1,
            color: Color::Named(NamedColor::BrightWhite),
            background: Some(Color::Rgb(Rgb { r: 30, g: 30, b: 45 })),
            text_style: TextStyle::Bold,
            ..Default::default()
        },
    ));

    let mut close = Element::button("modal_close", "close", button_style(12, 5, 16));
    close.style.background = Some(Color::Rgb(Rgb { r: 30, g: 30, b: 45 }));
    close.code = Some(ElementCode {
        on_click: Some(Box::new(|_, app: &mut AppState| close_modal(app))),
        on_key: Some(Box::new(|event, app: &mut AppState| {
            // кнопка в фокусе -> Enter закрывает
            if event == Event::Enter {
                close_modal(app);
            }
        })),
        on_render: None,
    });
    app.push_sibling(close);

    app.focused = find_id(app, "modal_close").unwrap_or(0);
    set_status(app, "modal: created");
}

fn close_modal(app: &mut AppState) {
    let Some(index) = find_id(app, "modal") else { return };
    set_visible_tree(app, index, false);
    app.focused = 0;
    set_status(app, "modal: closed");
}

// Esc: модалка открыта -> закрыть и остаться. false = закрывать нечего, main выходит
fn close_modal_if_open(app: &mut AppState) -> bool {
    let Some(index) = find_id(app, "modal") else { return false };
    if !app.elements[index].style.visible {
        return false;
    }
    close_modal(app);
    true
}

fn build_inputs(app: &mut AppState) {
    app.cursor = 0;
    app.push_child(Element::container("box_inputs", Style {
        x: 69,
        y: 3,
        width: 50,
        height: 19,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightBlue) }),
        ..Default::default()
    }));

    // 4 инпута = 4 режима direction_expansion, placeholder показывает режим.
    // Печать в любой: Key дописывает символ в value, Backspace стирает
    app.cursor = find_id(app, "box_inputs").expect("box_inputs");
    let mut input_down = Element::input("input_down", "direction: down", 1, Style {
        x: 1,
        y: 0,
        width: 18,
        height: 3,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightBlue) }),
        ..Default::default()
    });
    if let ElementData::InputData { max_width, max_height, .. } = &mut input_down.data {
        *max_width = 30;
        *max_height = 4;
    }
    input_down.code = Some(ElementCode {
        on_key: Some(Box::new(|event, app: &mut AppState| {
            let Some(index) = find_id(app, "input_down") else { return };
            if let ElementData::InputData { value, .. } = &mut app.elements[index].data {
                match event {
                    Event::Key(c) => value.push(c),
                    Event::Backspace => {
                        value.pop();
                    }
                    _ => {}
                }
            }
        })),
        on_click: None,
        on_render: None,
    });
    app.push_child(input_down);

    app.cursor = find_id(app, "box_inputs").expect("box_inputs");
    let mut input_right = Element::input("input_right", "direction: right", 2, Style {
        x: 1,
        y: 4,
        width: 18,
        height: 3,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightBlue) }),
        ..Default::default()
    });
    if let ElementData::InputData { max_width, max_height, .. } = &mut input_right.data {
        *max_width = 46;
        *max_height = 6;
    }
    input_right.code = Some(ElementCode {
        on_key: Some(Box::new(|event, app: &mut AppState| {
            let Some(index) = find_id(app, "input_right") else { return };
            if let ElementData::InputData { value, .. } = &mut app.elements[index].data {
                match event {
                    Event::Key(c) => value.push(c),
                    Event::Backspace => {
                        value.pop();
                    }
                    _ => {}
                }
            }
        })),
        on_click: None,
        on_render: None,
    });
    app.push_child(input_right);

    app.cursor = find_id(app, "box_inputs").expect("box_inputs");
    let mut input_both = Element::input("input_both", "direction: right + down", 3, Style {
        x: 1,
        y: 9,
        width: 25,
        height: 3,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightBlue) }),
        ..Default::default()
    });
    if let ElementData::InputData { max_width, max_height, .. } = &mut input_both.data {
        *max_width = 30;
        *max_height = 4;
    }
    input_both.code = Some(ElementCode {
        on_key: Some(Box::new(|event, app: &mut AppState| {
            let Some(index) = find_id(app, "input_both") else { return };
            if let ElementData::InputData { value, .. } = &mut app.elements[index].data {
                match event {
                    Event::Key(c) => value.push(c),
                    Event::Backspace => {
                        value.pop();
                    }
                    _ => {}
                }
            }
        })),
        on_click: None,
        on_render: None,
    });
    app.push_child(input_both);

    app.cursor = find_id(app, "box_inputs").expect("box_inputs");
    let mut input_none = Element::input("input_none", "direction: none", 0, Style {
        x: 1,
        y: 14,
        width: 17,
        height: 3,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightBlue) }),
        ..Default::default()
    });
    input_none.code = Some(ElementCode {
        on_key: Some(Box::new(|event, app: &mut AppState| {
            let Some(index) = find_id(app, "input_none") else { return };
            if let ElementData::InputData { value, .. } = &mut app.elements[index].data {
                match event {
                    Event::Key(c) => value.push(c),
                    Event::Backspace => {
                        value.pop();
                    }
                    _ => {}
                }
            }
        })),
        on_click: None,
        on_render: None,
    });
    app.push_child(input_none);
}

fn build_misc(app: &mut AppState) {
    app.cursor = 0;
    app.push_child(Element::container("box_misc", Style {
        x: 69,
        y: 23,
        width: 50,
        height: 11,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightBlack) }),
        ..Default::default()
    }));

    // 1. border у текста: draw_text рисует рамку вокруг строки + margin сдвигает
    app.cursor = find_id(app, "box_misc").expect("box_misc");
    app.push_child(Element::text("text_border", "text + border", Style {
        x: 1,
        y: 0,
        width: 22,
        height: 3,
        margin: 1,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::Magenta) }),
        ..Default::default()
    }));

    // 2. padding контейнера: дети считаются от контента = рамка + padding
    app.cursor = find_id(app, "box_misc").expect("box_misc");
    app.push_child(Element::container("padding_box", Style {
        x: 25,
        y: 0,
        width: 22,
        height: 5,
        padding: 2,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::Green) }),
        ..Default::default()
    }));

    app.cursor = find_id(app, "padding_box").expect("padding_box");
    app.push_child(Element::text("padding_text", "padding=2", Style {
        x: 0,
        y: 0,
        width: 16,
        height: 1,
        ..Default::default()
    }));

    // 3. красная цель для кнопок toggle / remove
    app.cursor = find_id(app, "box_misc").expect("box_misc");
    app.push_child(Element::text(
        "red_target",
        "RED TARGET (toggle / remove)",
        Style {
            x: 1,
            y: 6,
            width: 46,
            height: 3,
            color: Color::Named(NamedColor::BrightWhite),
            background: Some(Color::Named(NamedColor::Red)),
            ..Default::default()
        },
    ));
}

// ───────────── темы ─────────────

// кастомная тема: палитра перекрывает значения NamedColor,
// placeholder приглушается формулой (mix с белым)
fn theme_custom() -> vexel::Theme {
    use std::collections::HashMap;
    let mut palette = HashMap::new();
    palette.insert(NamedColor::Red, Color::Rgb(Rgb { r: 255, g: 60, b: 60 }));
    palette.insert(NamedColor::White, Color::Rgb(Rgb { r: 240, g: 240, b: 240 }));
    palette.insert(NamedColor::BrightWhite, Color::Rgb(Rgb { r: 255, g: 255, b: 255 }));
    vexel::Theme {
        background: Color::Rgb(Rgb { r: 18, g: 20, b: 28 }),
        palette,
    }
}

// блок-свитч тем: кнопки, каждая ставит свою тему в app.theme
fn build_themes(app: &mut AppState) {
    app.cursor = 0;
    app.push_child(Element::container("box_themes", Style {
        x: 1,
        y: 28,
        width: 64,
        height: 7,
        border: Some(Border { width: 1, color: Color::Named(NamedColor::BrightYellow) }),
        ..Default::default()
    }));

    app.cursor = find_id(app, "box_themes").expect("box_themes");
    app.push_child(Element::text(
        "themes_title",
        "темы: дефолт / кастом (палитра) / светлая",
        Style { x: 1, y: 0, width: 60, height: 1, ..Default::default() },
    ));

    let mut theme_default = Element::button("theme_default", "default", button_style(1, 2, 13));
    theme_default.code = Some(ElementCode {
        on_click: Some(Box::new(|_, app: &mut AppState| {
            app.theme = vexel::Theme::default();
            set_status(app, "theme: default");
        })),
        on_key: None,
        on_render: None,
    });
    app.push_child(theme_default);

    let mut theme_custom_btn = Element::button("theme_custom", "custom", button_style(15, 2, 13));
    theme_custom_btn.code = Some(ElementCode {
        on_click: Some(Box::new(|_, app: &mut AppState| {
            app.theme = theme_custom();
            set_status(app, "theme: custom");
        })),
        on_key: None,
        on_render: None,
    });
    app.push_sibling(theme_custom_btn);

    let mut theme_light = Element::button("theme_light", "light", button_style(29, 2, 13));
    theme_light.code = Some(ElementCode {
        on_click: Some(Box::new(|_, app: &mut AppState| {
            app.theme = vexel::Theme {
                background: Color::Rgb(Rgb { r: 235, g: 235, b: 230 }),
                ..Default::default()
            };
            set_status(app, "theme: light");
        })),
        on_key: None,
        on_render: None,
    });
    app.push_sibling(theme_light);
}

// ───────────── main ─────────────

fn main() -> io::Result<()> {
    let mut app = AppState::new();
    let mut renderer = Renderer::new();
    let mut stdout = io::stdout();

    build_title(&mut app);
    build_status(&mut app);
    build_colors(&mut app);
    build_rgb(&mut app);
    build_text_styles(&mut app);
    build_controls(&mut app);
    build_inputs(&mut app);
    build_misc(&mut app);
    build_themes(&mut app);

    // фокус на инпуте: без фокуса символы уходят в root и никуда не печатаются
    app.focused = find_id(&app, "input_down").unwrap_or(0);

    // события → диспетчер → кадр
    stdout.execute(EnableMouseCapture)?;
    loop {
        if poll_event(Duration::from_millis(16)) {
            if let Some(ev) = read_event() {
                match ev {
                    Event::Escape => {
                        // Esc сначала закрывает модалку, второй Esc выходит
                        if close_modal_if_open(&mut app) {
                            continue;
                        }
                        break;
                    }
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
