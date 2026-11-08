use std::io::{self, Write};
use crate::{AppState, Element, ElementData, Position, Color, NamedColor};

use crossterm::{
    cursor::{MoveTo, Hide},
    style::{Color as CTColor, Print, SetForegroundColor, SetBackgroundColor},
    terminal::{self, Clear, ClearType, enable_raw_mode},
    ExecutableCommand, QueueableCommand,
};

#[derive(Clone, Copy, PartialEq)]
pub struct Cell {//символ
    pub ch: char,
    pub fg: Color,
    pub bg: Option<Color>,
}

impl Default for Cell {
    fn default() -> Self {
        Self { ch: ' ', fg: Color::WHITE, bg: None }
    }
}

pub struct Buffer {//i = y * width + x
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); width as usize * height as usize],
        }
    }

        pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        if x >= self.width || y >= self.height { return; }//за пределами экрана - обрезание
        let i = y as usize * self.width as usize + x as usize;
        self.cells[i] = cell;
    }
}

// разница между кадрами: только изменившиеся ячейки
pub fn diff(old: &Buffer, new: &Buffer) -> Vec<(u16, u16, Cell)> {//сравнение прошлого и текущего кадров
    let mut out = Vec::new();
    for i in 0..new.cells.len() {
        if old.cells[i] != new.cells[i] {
            let x = (i % new.width as usize) as u16;
            let y = (i / new.width as usize) as u16;
            out.push((x, y, new.cells[i]));
        }
    }
    out
}

//позиция вектора != индекс элемента
#[derive(Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
    pub index: usize,
}

// владеет бек буфером между кадрами
pub struct Renderer {
    back: Buffer,
    rects: Vec<Rect>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { back: Buffer::new(0, 0), rects: Vec::new() }
    }

    pub fn render(&mut self, app: &AppState) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();

        let (w, h) = terminal::size()?;
        //ресайз терминала. буферы несовместимы, полный сброс
        if self.back.width != w || self.back.height != h {
            stdout.execute(Clear(ClearType::All))?;
            self.back = Buffer::new(w, h);
        }

        //1 заполняем фронт буфер (без stdout)
        let mut front = Buffer::new(w, h);
        self.rects.clear(); // каждый кадр заново: дерево могло измениться
        let root = &app.elements[0];
        draw_element(&mut front, &mut self.rects, app, root, 0, 0);

        //2 сравнение буферов, в stdout уезжает только разница
        stdout.execute(Hide)?;
        for (x, y, cell) in diff(&self.back, &front) {
            stdout.queue(MoveTo(x, y))?;
            stdout.queue(SetForegroundColor(to_ct(cell.fg)))?;
            if let Some(bg) = cell.bg {
                stdout.queue(SetBackgroundColor(to_ct(bg)))?;
            }
            stdout.queue(Print(cell.ch))?;
        }
        stdout.flush()?;

        //3 свап: фронт меняется с бек буфером
        self.back = front;
        Ok(())
    }
    pub fn hit_test(&self, x: u16, y: u16) -> Option<usize> {
        for i in (0..self.rects.len()).rev() {
            let r = &self.rects[i];
            //u32, x+w может переполнить u16
            let (x, y) = (x as u32, y as u32);
            if x >= r.x as u32 && x < r.x as u32 + r.w as u32 && y >= r.y as u32 && y < r.y as u32 + r.h as u32 {
                return Some(r.index);
            }
        }
        None
    }
}

fn draw_element(buf: &mut Buffer, rects: &mut Vec<Rect>, app: &AppState, el: &Element, parent_x: u16, parent_y: u16) {
    let (x, y) = if el.style_position() == Position::Relative {
        (parent_x + el.style_x(), parent_y + el.style_y())
    } else {
        (el.style_x(), el.style_y())
    };

    // кэш координат элементов (rect)
    let idx = el.index as usize;
    if idx >= rects.len() {
        rects.resize(idx + 1, Rect { x: 0, y: 0, w: 0, h: 0, index: 0 });
    }
    rects[idx] = Rect {
        x,
        y,
        w: el.style_width(),
        h: el.style_height(),
        index: el.index,
    };

    // невидимый сам не рисуется, но дети да
    if el.style.visible {
        match &el.data {
            ElementData::TextData(text) => {
                for (i, ch) in text.chars().enumerate() {
                    buf.set(x + i as u16, y, Cell { ch, fg: el.style.color, bg: el.style.background });
                }
            }
            ElementData::BoxData => draw_box(buf, el, x, y),
            ElementData::ButtonData { label } => draw_button(buf, el, x, y, label),
            ElementData::InputData { .. } => draw_input(buf, el, x, y),
        }
    }

    for &child_idx in &el.children {
        draw_element(buf, rects, app, &app.elements[child_idx], x, y);
    }
}

fn draw_button(buf: &mut Buffer, el: &Element, x: u16, y: u16, label: &str) {
    draw_box(buf, el, x, y);

    let (width, height) = (el.style_width(), el.style_height());
    if width == 0 || height == 0 { return; }

    // label(текст) по центру: смещение = (ширина - длина текста) / 2
    let len = label.chars().count() as u16;
    let tx = x + (width.saturating_sub(len)) / 2;
    let ty = y + height / 2;
    for (i, ch) in label.chars().enumerate() {
        buf.set(tx + i as u16, ty, Cell { ch, fg: el.style.color, bg: el.style.background });
    }
}

fn draw_input(buf: &mut Buffer, el: &Element, x: u16, y: u16) {
    draw_box(buf, el, x, y);

    let (width, height) = (el.style_width(), el.style_height());
    if width < 3 || height == 0 { return; }

    if let ElementData::InputData { placeholder, value } = &el.data {
        // value непустой ? показываем value : показываем placeholder
        // dim: placeholder рисуется приглушённым серым, чтобы отличать от введённого
        let (text, dim) = if value.is_empty() { (placeholder, true) } else { (value, false) };
        let fg = if dim { Color::Named(NamedColor::BrightBlack) } else { el.style.color };
        for (i, ch) in text.chars().take((width - 2) as usize).enumerate() {
            buf.set(x + 1 + i as u16, y + height / 2, Cell { ch, fg, bg: el.style.background });
        }
    }
}

fn draw_box(buf: &mut Buffer, el: &Element, x: u16, y: u16) {
    let (width, height) = (el.style_width(), el.style_height());
    if width < 2 || height < 2 {
        if width >= 1 && height >= 1 {
            buf.set(x, y, Cell { ch: '│', fg: el.style.color, bg: el.style.background });
        }
        return;
    }
    for i in 0..width {
        buf.set(x + i, y, cell(el, '─'));
        buf.set(x + i, y + height - 1, cell(el, '─'));
    }
    //вертикальные стороны без углов (0 и height-1)  их ставит горизонтальный проход
    for row in 1..height - 1 {
        buf.set(x, y + row, cell(el, '│'));
        buf.set(x + width - 1, y + row, cell(el, '│'));
    }
    //углы один раз, последними
    buf.set(x, y, cell(el, '┌'));
    buf.set(x + width - 1, y, cell(el, '┐'));
    buf.set(x, y + height - 1, cell(el, '└'));
    buf.set(x + width - 1, y + height - 1, cell(el, '┘'));
}

fn cell(el: &Element, ch: char) -> Cell {
    Cell { ch, fg: el.style.color, bg: el.style.background }
}

fn to_ct(c: Color) -> CTColor {
    match c {
        Color::Rgb(rgb) => CTColor::Rgb { r: rgb.r, g: rgb.g, b: rgb.b },
        Color::Named(n) => match n {
            NamedColor::Black => CTColor::Black,
            NamedColor::Red => CTColor::DarkRed,
            NamedColor::Green => CTColor::DarkGreen,
            NamedColor::Yellow => CTColor::DarkYellow,
            NamedColor::Blue => CTColor::DarkBlue,
            NamedColor::Magenta => CTColor::DarkMagenta,
            NamedColor::Cyan => CTColor::DarkCyan,
            NamedColor::White => CTColor::Grey,
            NamedColor::BrightBlack => CTColor::DarkGrey,
            NamedColor::BrightRed => CTColor::Red,
            NamedColor::BrightGreen => CTColor::Green,
            NamedColor::BrightYellow => CTColor::Yellow,
            NamedColor::BrightBlue => CTColor::Blue,
            NamedColor::BrightMagenta => CTColor::Magenta,
            NamedColor::BrightCyan => CTColor::Cyan,
            NamedColor::BrightWhite => CTColor::White,
        },
    }
}
