use std::io::{self, Write};
use crate::{AppState, Element, ElementData, Position, Color, NamedColor};

use crossterm::{
    cursor::{MoveTo, Hide},
    style::{Color as CTColor, Print, SetForegroundColor, SetBackgroundColor},
    terminal::{self, Clear, ClearType, enable_raw_mode},
    ExecutableCommand, QueueableCommand,
};

// Ячейка экрана: что должно быть нарисовано в точке (x, y).
// Рендер больше не пишет в stdout — он заполняет буфер ячеек.
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
pub fn diff(front: &Buffer, back: &Buffer) -> Vec<(u16, u16, Cell)> {//сравнение фронт и бек буффреа
    let mut out = Vec::new();
    for i in 0..front.cells.len() {
        if back.cells[i] != front.cells[i] {
            let x = (i % front.width as usize) as u16;
            let y = (i / front.width as usize) as u16;
            out.push((x, y, front.cells[i]));
        }
    }
    out
}

// владеет бек буфером между кадрами
pub struct Renderer {
    back: Buffer,
}

impl Renderer {
    pub fn new() -> Self {
        Self { back: Buffer::new(0, 0) }
    }

    pub fn render(&mut self, app: &AppState) -> io::Result<()> {
        enable_raw_more()?;
        let mut stdout = io::stdout();

        let (w, h) = terminal::size()?;
        //ресайз терминала. буферы несовместимы, полный сброс
        if self.back.width != w || self.back.height != h {
            stdout.execute(Clear(ClearType::All))?;
            self.back = Buffer::new(w, h);
        }

        //1 заполняем фронт буфер (без stdout)
        let mut front = Buffer::new(w, h);
        let root = &app.elements[0];
        draw_element(&mut front, app, root, 0, 0);

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
}

fn draw_element(buf: &mut Buffer, app: &AppState, el: &Element, parent_x: u16, parent_y: u16) {
    let (x, y) = if el.style_position() == Position::Relative {
        (parent_x + el.style_x(), parent_y + el.style_y())
    } else {
        (el.style_x(), el.style_y())
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
            _ => { /* Button/Input заглушка */ }
        }
    }

    for &child_idx in &el.children {
        draw_element(buf, app, &app.elements[child_idx], x, y);
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
    for row in 0..height {
        buf.set(x, y + row, cell(el, if row == 0 { '┌' } else if row == height - 1 { '└' } else { '│' }));
        buf.set(x + width - 1, y + row, cell(el, if row == 0 { '┐' } else if row == height - 1 { '┘' } else { '│' }));
    }
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
