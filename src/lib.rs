pub struct AppState {
    pub elements: Vec<Element>,
    pub root: usize,
    pub dirty: Vec<usize>,
}

pub struct Element {
    pub id: String,
    pub kind: ElementKind,
    pub data: ElementData,
    pub style: Style,
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub dirty: bool,
    pub focusable: bool,
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
    pub color: Color,
    pub background: Option<Color>,
    pub border: Option<Border>,
    pub padding: usize,
    pub margin: usize,
    pub visible: bool,
}

pub struct Border {
    pub width: u8,
    pub color: Color,
}

pub enum Color {
    Rgb(u8, u8, u8),
    Named(String),
}
