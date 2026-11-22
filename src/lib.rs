pub mod events;
pub mod render;

use std::collections::HashMap;

pub struct AppState {
    pub elements: Vec<Element>,
    pub cursor: usize,
    pub focused: usize,
    pub prev: usize,
    pub theme: Theme,
}

impl AppState {
    pub fn new() -> Self {
        let root = Element {
            id: "root".to_string(),
            index: 0,
            data: ElementData::BoxData,
            style: Style::default(),
            children: Vec::new(),
            parent: 0,
            code: None,
        };
        let root_index = root.index;
        Self {
            elements: vec![root],
            cursor: root_index,
            focused: root_index,
            prev: root_index,
            theme: Theme::default(),
        }
    }


    pub fn push_child(&mut self, mut element: Element) {
        let idx = self.elements.len();
        element.index = idx;
        element.parent = self.cursor;
        self.elements.push(element);
        self.elements[self.cursor].children.push(idx);
        self.prev = self.cursor;
        self.cursor = idx;
    }
    pub fn push_sibling(&mut self, mut element: Element) {
        let idx = self.elements.len();
        element.index = idx;
        let copy = self.cursor;
        self.cursor_up();
        element.parent = self.cursor;
        self.elements.push(element);
        self.elements[self.cursor].children.push(idx);
        self.prev = copy;
        self.cursor = idx;
    }
    pub fn delete_element(&mut self, index: usize) {
        let Some(el) = self.elements.get_mut(index) else { return };
        el.style.visible = false;
        // фокус не должен висеть на скрытом элементе
        if self.focused == index {
            self.focused = el.parent;
        }
    }
    pub fn update_element(&mut self, mut element: Element) {
        let idx = element.index;
        let Some(slot) = self.elements.get_mut(idx) else { return };
        // дерево не переносится обновлением: у свежего Element parent=0, children пусты.
        element.parent = slot.parent;
        if element.children.is_empty() {
            element.children = std::mem::take(&mut slot.children);
        }
        // code не клонируется (Box<dyn Fn>): прислал элемент без колбэков — оставляем старые
        if element.code.is_none() {
            element.code = slot.code.take();
        }
        *slot = element;
    }
    pub fn cursor_up(&mut self) {
        if self.cursor == 0 { return }
        self.prev = self.cursor;
        self.cursor = self.elements[self.cursor].parent;
    }
    pub fn cursor_down(&mut self) {
        if self.elements[self.cursor].children.is_empty() { return }
        self.prev = self.cursor;
        self.cursor = self.elements[self.cursor].children[0];
    }
    pub fn back_cursor(&mut self) {
        std::mem::swap(&mut self.cursor, &mut self.prev);
    }
}

pub struct Element {
    pub id: String,
    pub index: usize,
    pub data: ElementData,
    pub style: Style,
    pub children: Vec<usize>,
    pub parent: usize,
    pub code: Option<ElementCode>,
}

impl Element {
    fn new(id: &str, data: ElementData, style: Style) -> Self {//custom
        Self {
            id: id.to_string(),
            index: 0,
            data,
            style,
            children: Vec::new(),
            parent: 0,
            code: None,
        }
    }
  
    pub fn text(id: &str, text: &str, style: Style) -> Self {
        Self::new(id, ElementData::TextData(text.to_string()), style)
    }
    pub fn container(id: &str, style: Style) -> Self {
        Self::new(id, ElementData::BoxData, style)
    }
    pub fn button(id: &str, label: &str, style: Style) -> Self {
        Self::new(id, ElementData::ButtonData { label: label.to_string() }, style)
    }
    pub fn input(id: &str, placeholder: &str, expansion: u8, style: Style) -> Self {
        Self::new(id, ElementData::InputData {
            placeholder: placeholder.to_string(),
            value: String::new(),
            max_width: 10,
            max_height: 4,
            direction_expansion: expansion as i8,
        }, style)
    }
    pub fn style_x(&self) -> u16 {
        self.style.x.max(0) as u16
    }
    pub fn style_y(&self) -> u16 {
        self.style.y.max(0) as u16
    }
}

pub fn clone_element(el: &Element) -> Element {
    Element {
        id: el.id.clone(),
        index: el.index,
        data: el.data.clone(),
        style: el.style.clone(),
        children: el.children.clone(),
        parent: el.parent,
        code: None,
    }
  }


pub struct ElementCode {
    pub on_click: Option<Box<dyn Fn(Event, &mut AppState)>>,
    pub on_key: Option<Box<dyn Fn(Event, &mut AppState)>>,
    pub on_render: Option<Box<dyn Fn(&mut Element)>>,
    
}

#[derive(Clone, Copy, PartialEq)]
pub enum Event {
    Click { x: u16, y: u16 },
    Key(char),
    Enter,
    Tab,
    Escape,
    Up,
    Down,
    Backspace,
}

#[derive(Clone)]
pub enum ElementData {
    BoxData,
    TextData(String),
    ButtonData { label: String },
    InputData {
        placeholder: String,
        value: String,
        max_width: usize,
        max_height: usize,
        direction_expansion: i8, // 0 no expansion, 1 down, 2 rigth, 3 rigth + down
    },
}

#[derive(Clone)]
pub struct Style {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub position: Position,
    pub color: Color,
    pub background: Option<Color>,
    pub border: Option<Border>,
    pub text_style: TextStyle,//жирность/курсив/подчёркивание — единственный источник стилей текста
    pub padding: u16,//внутреннй отсуп
    pub margin: u16,//внешний отсуп
    pub visible: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            position: Position::Relative,
            color: Color::WHITE,
            background: None,
            border: None,
            text_style: TextStyle::Normal,
            padding: 0,
            margin: 0,
            visible: true,
        }
    }
}

// атрибуты текста: все комбинации 3 флагов (2^3)
#[derive(Clone, Copy, PartialEq, Default)]
pub enum TextStyle {
    #[default]
    Normal,
    Bold,
    Italic,
    Underline,
    BoldItalic,
    BoldUnderline,
    ItalicUnderline,
    BoldItalicUnderline,
}

impl TextStyle {
    pub fn bold(&self) -> bool {
        matches!(self, TextStyle::Bold | TextStyle::BoldItalic | TextStyle::BoldUnderline | TextStyle::BoldItalicUnderline)
    }
    pub fn italic(&self) -> bool {
        matches!(self, TextStyle::Italic | TextStyle::BoldItalic | TextStyle::ItalicUnderline | TextStyle::BoldItalicUnderline)
    }
    pub fn underline(&self) -> bool {
        matches!(self, TextStyle::Underline | TextStyle::BoldUnderline | TextStyle::ItalicUnderline | TextStyle::BoldItalicUnderline)
    }
}

#[derive(Clone)]
pub struct Border {
    pub width: u8,
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Muted,//цвет placeholder
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Rgb(Rgb),
    Named(NamedColor),
}

impl Color {
    pub const WHITE: Color = Color::Named(NamedColor::White);
    pub const BLACK: Color = Color::Named(NamedColor::Black);
}

#[derive(Clone, Copy, PartialEq)]
pub enum Position {
    Relative,
    Absolute,
}

// таблица цветовых оверрайдов. ключа нет -> дефолт
pub type Palette = HashMap<NamedColor, Color>;

pub struct Theme {
    pub background: Color,       // фон всего приложения
    pub palette: Palette,        // юзер переопределяет значения NamedColor
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::BLACK,
            palette: Palette::new(),
        }
    }
}

impl Theme {
    //NamedColor: юзерская палитра -> сам NamedColor как Color
    pub fn color(&self, c: NamedColor) -> Color {
        *self.palette.get(&c).unwrap_or(&Color::Named(c))
    }

    // приглушённая в2wерсия цвета
    pub fn muted(&self, c: Color) -> Color {
        blend(self.to_rgb(c, 128), Rgb { r: 255, g: 255, b: 255 })
    }

    // named проходит через палитру; палитра вернула named -> fallback-серый
    fn to_rgb(&self, c: Color, fallback: u8) -> Rgb {
        match c {
            Color::Rgb(rgb) => rgb,
            Color::Named(n) => match self.color(n) {
                Color::Rgb(rgb) => rgb,
                Color::Named(_) => Rgb { r: fallback, g: fallback, b: fallback },
            },
        }
    }
}

// среднее двух цветов покомпонентно
pub(crate) fn blend(a: Rgb, b: Rgb) -> Color {
    Color::Rgb(Rgb {
        r: ((a.r as u16 + b.r as u16) / 2) as u8,
        g: ((a.g as u16 + b.g as u16) / 2) as u8,
        b: ((a.b as u16 + b.b as u16) / 2) as u8,
    })
}

