pub mod render;

pub struct AppState {
    pub elements: Vec<Element>,
    pub cursor: usize,
    pub focused: usize,
    pub prev: usize,
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
            dirty: false,
        };
        let root_index = root.index;
        Self {   
            elements: vec![root],
            cursor: root_index,
            focused: root_index,
            prev: root_index,
        }
    }


    pub fn push_child(&mut self, mut element: Element) {
        let idx = self.elements.len();
        element.dirty = true;
        element.index = idx;
        self.elements.push(element);
        self.elements[self.cursor].children.push(idx);
        self.prev = self.cursor;
        self.cursor = idx;
    }
    pub fn update_element(&mut self, element: Element) { 
        let idx = element.index;
        self.cursor = idx;
        self.elements[idx] = element;
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
        let a = self.cursor;
        self.cursor = self.prev;
        self.prev = a;
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
    pub dirty: bool,
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
            dirty: false,
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
    pub fn input(id: &str, placeholder: &str, style: Style) -> Self {
        Self::new(id, ElementData::InputData {
            placeholder: placeholder.to_string(),
            value: String::new(),
        }, style)
    }
    pub fn style_x(&self) -> u16 {
        self.style.x.max(0) as u16
    }
    pub fn style_y(&self) -> u16 {
        self.style.y.max(0) as u16
    }
    pub fn style_width(&self) -> u16 {
        self.style.width
    }
    pub fn style_height(&self) -> u16 {
        self.style.height
    }
    pub fn style_position(&self) -> Position {
        self.style.position
    }
}

pub struct ElementCode {
    pub on_click: Option<Box<dyn Fn(Event, &mut AppState)>>,
    pub on_key: Option<Box<dyn Fn(Event, &mut AppState)>>,
    pub on_render: Option<Box<dyn Fn(&mut Element)>>,
}

pub enum Event {
    Click { x: u16, y: u16 },
    Key(char),
    Enter,
    Tab,
    Escape,
}

pub enum ElementData {
    BoxData,
    TextData(String),
    ButtonData { label: String },
    InputData {
        placeholder: String,
        value: String,
    },
}

pub struct Style {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub position: Position,
    pub color: Color,
    pub background: Option<Color>,
    pub border: Option<Border>,
    pub padding: usize,//внутреннй отсуп
    pub margin: usize,//внешний отсуп 
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
            padding: 0,
            margin: 0,
            visible: true,
        }
    }
}

pub struct Border {
    pub width: u8,
    pub color: Color,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Copy, PartialEq)]
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
}

#[derive(Clone, Copy, PartialEq)]
pub enum Color {
    Rgb(Rgb),
    Named(NamedColor),
}

impl Color {
    pub const WHITE: Color = Color::Named(NamedColor::White);
}

#[derive(Clone, Copy, PartialEq)]
pub enum Position {
    Relative,
    Absolute,
}

