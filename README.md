# Vexel

TUI-библиотека на Rust под AI-агентов. Узкий инструмент, не универсальный фреймворк.

## Сборка / запуск

```bash
cargo check
cargo run --example test   
```

## Ядро

Плоское дерево: `Vec<Element>`, связи — индексы (`children: Vec<usize>`, `parent: usize`), root всегда 0.

```text
AppState
├── elements: Vec<Element>
├── cursor: usize          // куда упадёт следующий push_child
├── focused: usize         // кому идут клавиши
├── prev: usize            // для back_cursor
└── theme: Theme           // фон + палитра оверрайдов NamedColor

Element
├── id: String
├── index: usize
├── data: ElementData      // BoxData | TextData | ButtonData | InputData
├── style: Style
├── children: Vec<usize>
├── parent: usize
└── code: Option<ElementCode>  // on_click / on_key / on_render

Style
├── x: i16, y: i16
├── width: u16, height: u16
├── position: Position     // realetive/absolute
├── color / background / border
├── text_style: TextStyle  // bold/italic/underline
├── padding / margin
└── visible: bool
```


## Renderer

Двойная буферизация: кадр собирается в памяти (`Buffer`), diff против прошлого кадра, в stdout уезжают только изменённые ячейки. Попутно собираются `Rect` — по ним `hit_test` находит элемент под кликом (идёт с конца, последний нарисованный сверху).

Input считается отдельно: `input_needed_size` возвращает реальный (w, h), его получают и rect, и рисование.

## Events

crossterm → `Event` (Click/Key/Enter/Tab/Escape/Up/Down/Backspace) → `dispatch`. Click идёт в элемент под курсором мыши (hit_test), остальное — в `app.focused`.
