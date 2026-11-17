use std::io::{self, Write};
use crate::{AppState, Element, ElementData, Position, Color, NamedColor};

use crossterm::{
    cursor::{MoveTo, Hide, Show},
    style::{Color as CTColor, Print, SetForegroundColor, SetBackgroundColor, SetAttribute, Attribute},
    terminal::{self, Clear, ClearType, enable_raw_mode, disable_raw_mode},
    ExecutableCommand, QueueableCommand,
};

#[derive(Clone, Copy, PartialEq)]
pub struct Cell {//символ
    pub ch: char,
    pub fg: Color,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self { ch: ' ', fg: Color::WHITE, bg: None, bold: false, italic: false, underline: false }
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
    if new.width == 0 { return out; } // деление на ширину ниже
    // буферы разного размера бывают только сразу после ресайза, render их сбрасывает
    let len = old.cells.len().min(new.cells.len());
    for i in 0..len {
        if old.cells[i] != new.cells[i] {
            let x = (i % new.width as usize) as u16;
            let y = (i / new.width as usize) as u16;
            out.push((x, y, new.cells[i]));
        }
    }
    out
}

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
    front: Buffer,// между кадрами
    rects: Vec<Rect>,
    initialized: bool,// raw mode + скрытый курсор включаются один раз, на первом кадре
}

impl Default for Renderer {
    fn default() -> Self { Self::new() }
}

impl Renderer {
    pub fn new() -> Self {
        Self { back: Buffer::new(0, 0), front: Buffer::new(0, 0), rects: Vec::new(), initialized: false }
    }

    pub fn render(&mut self, app: &AppState) -> io::Result<()> {
        let mut stdout = io::stdout();

        if !self.initialized {
            enable_raw_mode()?;
            stdout.execute(Hide)?;
            self.initialized = true;
        }

        let (w, h) = terminal::size()?;
        //ресайз терминала. буферы несовместимы, полный сброс
        if self.back.width != w || self.back.height != h {
            stdout.execute(Clear(ClearType::All))?;
            self.back = Buffer::new(w, h);
            self.front = Buffer::new(w, h);
        }

        //1 заполняем фронт буфер (без stdout)
        self.front.cells.fill(Cell::default());
        self.rects.clear(); // каждый кадр заново: дерево могло измениться
        let root = &app.elements[0];
        draw_element(&mut self.front, &mut self.rects, app, root, 0, 0);

        //2 сравнение буферов, в stdout уезжает только разница
        // кэш цветов: слать SGR только при смене; Attribute::Reset сбрасывает и цвета
        let mut last_fg: Option<Color> = None;
        let mut last_bg: Option<Option<Color>> = None;
        for (x, y, cell) in diff(&self.back, &self.front) {
            stdout.queue(MoveTo(x, y))?;
            if last_fg != Some(cell.fg) {
                stdout.queue(SetForegroundColor(to_ct(cell.fg)))?;
                last_fg = Some(cell.fg);
            }
            // без фона в терминале останется фон предыдущей ячейки, поэтому None = Reset
            if last_bg != Some(cell.bg) {
                match cell.bg {
                    Some(bg) => { stdout.queue(SetBackgroundColor(to_ct(bg)))?; }
                    None => { stdout.queue(SetBackgroundColor(CTColor::Reset))?; }
                }
                last_bg = Some(cell.bg);
            }
            // атрибуты: включаем → печатаем → сбрасываем (иначе залипнут на следующие ячейки)
            if cell.bold { stdout.queue(SetAttribute(Attribute::Bold))?; }
            if cell.italic { stdout.queue(SetAttribute(Attribute::Italic))?; }
            if cell.underline { stdout.queue(SetAttribute(Attribute::Underlined))?; }
            stdout.queue(Print(cell.ch))?;
            if cell.bold || cell.italic || cell.underline {
                stdout.queue(SetAttribute(Attribute::Reset))?;
                last_fg = None;// Reset сбросил и цвета
                last_bg = None;
            }
        }
        stdout.flush()?;

        //3 свап: фронт становится бек буфером
        std::mem::swap(&mut self.back, &mut self.front);
        Ok(())
    }
    pub fn hit_test(&self, x: u16, y: u16) -> Option<usize> {
        let (x, y) = (x as u32, y as u32);//u32: x+w может переполнить u16
        for r in self.rects.iter().rev() {
            if x >= r.x as u32 && x < r.x as u32 + r.w as u32 && y >= r.y as u32 && y < r.y as u32 + r.h as u32 {
                return Some(r.index);
            }
        }
        None
    }
}

// терминал возвращается в нормальное состояние даже при панике
impl Drop for Renderer {
    fn drop(&mut self) {
        if !self.initialized { return; }
        let mut stdout = io::stdout();
        let _ = disable_raw_mode();
        let _ = stdout.execute(Show);
    }
}

fn draw_text(buf: &mut Buffer, el: &Element, x: u16, y: u16, text: &str) {
    let (fg, bg) = (el.style.color, el.style.background);
    let bold = el.style.text_style.bold();
    let italic = el.style.text_style.italic();
    let underline = el.style.text_style.underline();

    // height = 0 у текста трактуем как одну строку, иначе фон пропал бы совсем
    if let Some(bg) = bg {
        let rows = el.style_height().max(1);
        for row in 0..rows {
            for col in 0..el.style_width() {
                buf.set(x.saturating_add(col), y.saturating_add(row), Cell { ch: ' ', fg, bg: Some(bg), bold, italic, underline });
            }
        }
    }
    // рамка поверх заливки, если задана. draw_box заливает фон повторно тем же цветом — безвредно
    if el.style.border.is_some() {
        draw_box(buf, el, x, y);
    }

    // padding тоже внутренняя территория: текст рисуется внутри него, как в button/input
    let inset = content_inset(el);
    let tx = x.saturating_add(inset);
    let ty = y.saturating_add(inset);
    let max_chars = el.style_width().saturating_sub(inset.saturating_mul(2)) as usize;
    for (i, ch) in text.chars().take(max_chars).enumerate() {
        buf.set(tx.saturating_add(i as u16), ty, Cell { ch, fg, bg, bold, italic, underline });
    }
}

fn draw_element(buf: &mut Buffer, rects: &mut Vec<Rect>, app: &AppState, el: &Element, parent_x: u16, parent_y: u16) {
    let margin = el.style.margin;
    // saturating: координаты приходят из i16/style, лишний сдвиг не должен паниковать
    let (x, y) = if el.style_position() == Position::Relative {
        (parent_x.saturating_add(el.style_x()).saturating_add(margin),
         parent_y.saturating_add(el.style_y()).saturating_add(margin))
    } else {
        (el.style_x().saturating_add(margin), el.style_y().saturating_add(margin))
    };

    if !el.style.visible { return; }// скрытый элемент не рисуется и в rects не попадает

    // кэш координат элементов (rect)
    let idx = el.index;
    let h = match &el.data {
        ElementData::InputData { .. } => input_needed_height(el),
        _ => el.style_height(),
    };
    if idx >= rects.len() {
        rects.resize(idx + 1, Rect { x: 0, y: 0, w: 0, h: 0, index: 0 });
    }
    rects[idx] = Rect {
        x,
        y,
        w: el.style_width(),
        h,
        index: el.index,
    };

    match &el.data {
        ElementData::TextData(text) => draw_text(buf, el, x, y, text),
        ElementData::BoxData => draw_box(buf, el, x, y),
        ElementData::ButtonData { label } => draw_button(buf, el, x, y, label),
        ElementData::InputData { .. } => draw_input(buf, el, x, y),
    }

    // дети считаются от контента: рамка и padding не их территория.
    // inset = border(1 если есть) + padding
    let inset = content_inset(el);
    let (cx, cy) = (x.saturating_add(inset), y.saturating_add(inset));
    for &child_idx in &el.children {
        draw_element(buf, rects, app, &app.elements[child_idx], cx, cy);
    }
}

// отступ контента от внешней рамки: рамка + padding
fn content_inset(el: &Element) -> u16 {
    let border: u16 = if el.style.border.is_some() { 1 } else { 0 };
    border.saturating_add(el.style.padding)
}

fn draw_button(buf: &mut Buffer, el: &Element, x: u16, y: u16, label: &str) {
    draw_box(buf, el, x, y);

    let (width, height) = (el.style_width(), el.style_height());
    if width == 0 || height == 0 { return; }

    // рамка + padding — внутренняя территория кнопки
    let inset = content_inset(el);
    // saturating: width=1 + рамка = underflow в u16 (паника на debug)
    let (iw, ih) = (width.saturating_sub(inset.saturating_mul(2)), height.saturating_sub(inset.saturating_mul(2)));
    if iw == 0 || ih == 0 { return; }

    //label(текст) по центру, обрезается по ширине, стиль из Style
    let bold = el.style.text_style.bold();
    let italic = el.style.text_style.italic();
    let underline = el.style.text_style.underline();
    let len = label.chars().count() as u16;
    let tx = x.saturating_add(inset).saturating_add(iw.saturating_sub(len) / 2);
    let ty = y.saturating_add(inset).saturating_add(ih / 2);
    for (i, ch) in label.chars().take(iw as usize).enumerate() {
        buf.set(tx.saturating_add(i as u16), ty, Cell { ch, fg: el.style.color, bg: el.style.background, bold, italic, underline });
    }
}

fn draw_input(buf: &mut Buffer, el: &Element, x: u16, y: u16) {
    // input выше объявленной высоты, если текст не влез: рисуем box с реальной высотой
    let height = input_needed_height(el);
    draw_box_sized(buf, el, x, y, height);

    let width = el.style_width();
    // рамка занимает 1 колонку/строку с каждой стороны, padding тоже внутренняя территория
    let inset = content_inset(el);
    let (iw, ih) = (width.saturating_sub(inset.saturating_mul(2)), height.saturating_sub(inset.saturating_mul(2)));
    if iw == 0 || ih == 0 { return; }

    if let ElementData::InputData { placeholder, value } = &el.data {
        // value непустой ? показываем value : показываем placeholder
        // dim: placeholder рисуется приглушённым серым, чтобы отличать от введённого
        let (text, dim) = if value.is_empty() { (placeholder, true) } else { (value, false) };
        let fg = if dim { Color::Named(NamedColor::BrightBlack) } else { el.style.color };
        let bold = el.style.text_style.bold();
        let italic = el.style.text_style.italic();
        let underline = el.style.text_style.underline();

        let line_len = iw as usize;
        let rows = ih as usize; // сколько строк влезает внутри
        let (tx, ty) = (x.saturating_add(inset), y.saturating_add(inset));
        for (row, chunk) in text.chars().collect::<Vec<char>>().chunks(line_len).enumerate() {
            if row >= rows { break; }
            for (i, ch) in chunk.iter().enumerate() {
                buf.set(tx.saturating_add(i as u16), ty.saturating_add(row as u16), Cell { ch: *ch, fg, bg: el.style.background, bold, italic, underline });
            }
        }
    }
}

// сколько строк занимает текст input + рамка/padding сверху и снизу
fn input_needed_height(el: &Element) -> u16 {
    if let ElementData::InputData { placeholder, value } = &el.data {
        let (text, _) = if value.is_empty() { (placeholder, true) } else { (value, false) };
        let width = el.style_width();
        let inset = content_inset(el);
        let iw = width.saturating_sub(inset.saturating_mul(2));
        if iw == 0 { return el.style_height(); } // строк нет, рисовать нечего — минимум
        // пустой placeholder = 0 строк, div_ceil(0) = 0; ниже max(1) защищает от height без контента
        let lines = text.chars().count().div_ceil(iw as usize).max(1);
        // высота = строки текста + inset сверху и снизу, но не меньше style.height
        (lines as u16).saturating_add(inset.saturating_mul(2)).max(el.style_height())
    } else {
        el.style_height()
    }
}

fn draw_box(buf: &mut Buffer, el: &Element, x: u16, y: u16) {
    draw_box_sized(buf, el, x, y, el.style_height());
}

// box с явной высотой (input растёт под текст, не меняя style)
fn draw_box_sized(buf: &mut Buffer, el: &Element, x: u16, y: u16, height: u16) {
    let (width, height) = (el.style_width(), height);
    if width == 0 || height == 0 { return; }

    // заливка фоном всей площади
    if let Some(bg) = el.style.background {
        for row in 0..height {
            for col in 0..width {
                buf.set(x.saturating_add(col), y.saturating_add(row), Cell { ch: ' ', fg: el.style.color, bg: Some(bg), ..Default::default() });
            }
        }
    }

    let border = match &el.style.border {
        Some(b) => b,
        None => return, // рамки нет - box это чистая заливка
    };
    let border_cell = Cell { ch: ' ', fg: border.color, bg: el.style.background, ..Default::default() };

    // saturating: x + width может выйти за u16, Buffer::set всё равно обрежет
    let right = x.saturating_add(width - 1);
    let bottom = y.saturating_add(height - 1);

    if width < 2 || height < 2 {
        // рамка не влезает: однострочный box = сплошные ─, узкий = │
        if height == 1 {
            for i in 0..width {
                buf.set(x.saturating_add(i), y, Cell { ch: '─', ..border_cell });
            }
        } else if width == 1 {
            for row in 0..height {
                buf.set(x, y.saturating_add(row), Cell { ch: '│', ..border_cell });
            }
        }
        return;
    }

    for i in 0..width {
        buf.set(x.saturating_add(i), y, Cell { ch: '─', ..border_cell });
        buf.set(x.saturating_add(i), bottom, Cell { ch: '─', ..border_cell });
    }
    for row in 1..height - 1 {
        buf.set(x, y.saturating_add(row), Cell { ch: '│', ..border_cell });
        buf.set(right, y.saturating_add(row), Cell { ch: '│', ..border_cell });
    }
    buf.set(x, y, Cell { ch: '┌', ..border_cell });
    buf.set(right, y, Cell { ch: '┐', ..border_cell });
    buf.set(x, bottom, Cell { ch: '└', ..border_cell });
    buf.set(right, bottom, Cell { ch: '┘', ..border_cell });
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
