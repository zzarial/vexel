pub mod render;

pub struct AppState {
    pub elements: Vec<Element>,
    pub cursor: usize, 
    pub prev: usize,
}

impl AppState {
    pub fn new() -> Self {
        let root = Element {
            id: "root".to_string(),
            index: 0,
            kind: ElementKind::Box,
            data: ElementData::BoxData,
            style: None,
            childrens: Vec::new(),
            parent: 0,
            code: None,
            dirty: false,
            focusable: false,
        };
        let root_index = root.index;
        Self {   
            elements: vec![root],
            cursor: root_index,
            prev: root_index,
        }
    }


    pub fn push_child(&mut self, mut element: Element) {
        let idx = self.elements.len();
        element.dirty = true;
        element.index = idx;
        self.elements.push(element);
        self.elements[self.cursor].childrens.push(idx);
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
        if self.elements[self.cursor].childrens.is_empty() { return }
        self.prev = self.cursor;
        self.cursor = self.elements[self.cursor].childrens[0];
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
    pub kind: ElementKind,
    pub data: ElementData,
    pub style: Option<Style>,
    pub childrens: Vec<usize>,
    pub parent: usize,
    pub code: Option<ElementCode>,
    pub dirty: bool,
    pub focusable: bool,
}

impl Element {
    // у элемента без стиля — координаты (0, 0), как у Default
    // отрицательные x/y обрезаются до 0: за левым краем терминала не рисуем
    pub fn style_x(&self) -> u16 {
        self.style.as_ref().map_or(0, |s| s.x.max(0) as u16)
    }
    pub fn style_y(&self) -> u16 {
        self.style.as_ref().map_or(0, |s| s.y.max(0) as u16)
    }
    pub fn style_width(&self) -> u16 {
        self.style.as_ref().map_or(0, |s| s.width)
    }
    pub fn style_height(&self) -> u16 {
        self.style.as_ref().map_or(0, |s| s.height)
    }
    pub fn style_position(&self) -> Position {
        self.style.as_ref().map_or(Position::Relative, |s| s.position)
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

pub enum ElementKind {
    Box,
    Text,
    Button,
    Input,
}

pub enum ElementData {
    BoxData,
    TextData(String),
    ButtonData { label: String },
    InputData {
        placeholder: String,
        value: String,
        focused: bool,
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
            width: 0,//обязательное поле
            height: 0,//обязательное поле
            position: Position::Relative,
            color: Color::Named("White".to_string()),
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

pub enum Color {
    Rgb(u8, u8, u8),
    Named(String),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Position {
    Relative,
    Absolute,
}

