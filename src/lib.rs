pub mod events;
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
        element.parent = self.cursor;
        self.elements.push(element);
        self.elements[self.cursor].children.push(idx);
        self.prev = self.cursor;
        self.cursor = idx;
    }
    pub fn push_sibling(&mut self, mut element: Element) {
        let idx = self.elements.len();
        element.dirty = true;
        element.index = idx;
        let copy = self.cursor;
        self.cursor_up();
        element.parent = self.cursor;// сосед = в того же родителя, что и текущий
        self.elements.push(element);
        self.elements[self.cursor].children.push(idx);
        self.prev = copy;
        self.cursor = idx;
    }
    pub fn delete_element(&mut self, index: usize) {
        let Some(el) = self.elements.get_mut(index) else { return };
        el.style.visible = false;
        // фокус не должен висеть на скрытом элементе, иначе клавиши уходят в пустоту
        if self.focused == index {
            self.focused = el.parent;
        }
    }
    pub fn update_element(&mut self, mut element: Element) {
        let idx = element.index;
        let Some(slot) = self.elements.get_mut(idx) else { return };
        // дерево не переносится обновлением: у свежего Element parent=0, children пусты.
        // без этого дети отвязались бы от родителя, а элемент уехал в root
        element.parent = slot.parent;
        if element.children.is_empty() {
            element.children = std::mem::take(&mut slot.children);
        }
        self.cursor = idx;
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

pub fn clone_element(el: &Element) -> Element {
    Element {
        id: el.id.clone(),
        index: el.index,
        data: el.data.clone(),
        style: el.style.clone(),
        children: el.children.clone(),
        parent: el.parent,
        code: None,
        dirty: false,
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

