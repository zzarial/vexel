# Vexel

Библиотека для создания интерактивных TUI (Terminal User Interface) на ванильном Rust, без внешних зависимостей.

## Структура проекта

```text
vexel/
├── src/
│   └── lib.rs          // Все основные структуры и логика (этап ядра)
├── Cargo.toml
└── README.md
```

Проект в ранней стадии — сейчас это ядро: модель элементов и управление состоянием.

## Архитектура (план)

```text
┌─────────────────────────────────────────────────────────────┐
│  ЯДРО (Core)                                       [готово] │
│  AppState — плоский Vec<Element>, курсор для навигации,     │
│  dirty-флаги для частичной перерисовки                      │
├─────────────────────────────────────────────────────────────┤
│  Renderer                                          [план]   │
│  Обход дерева, расчёт координат, ANSI-вывод в терминал      │
├─────────────────────────────────────────────────────────────┤
│  Events                                            [план]   │
│  Чтение ввода, диспетчеризация событий (click, key...)      │
└─────────────────────────────────────────────────────────────┘
```

## Основные структуры

Актуально на текущий этап:

```text
AppState
├── elements: Vec<Element>      // плоский массив всех элементов (root всегда индекс 0)
├── cursor: usize               // индекс элемента под курсором
└── prev: usize                 // предыдущая позиция курсора

Методы AppState
├── push_child(element)         // добавить ребёнка к элементу под курсором
├── update_element(element)     // заменить элемент по его index
├── cursor_up()                 // перейти к родителю (no-op на root)
├── cursor_down()               // перейти к первому ребёнку (no-op если детей нет)
└── back_cursor()               // вернуться на предыдущую позицию

Element
├── id: String
├── index: usize                // позиция в elements
├── kind: ElementKind           // Box | Text | Button | Input
├── data: ElementData           // данные элемента (текст, label, value...)
├── style: Option<Style>
├── add_style: Option<AdditionalStyle>
├── childrens: Vec<usize>       // индексы детей
├── parent: usize               // индекс родителя (у root — сам на себя, 0)
├── code: Option<ElementCode>   // колбэки: on_click, on_key, on_render
├── dirty: bool                 // нужно ли перерисовать
└── focusable: bool             // может ли получать фокус

Style
├── x: i16, y: i16              // позиция относительно родителя
├── width: u16, height: u16     // размер
├── color: Color                // цвет элемента
├── background: Option<Color>   // фон
├── border: Option<Border>      // бордер
├── padding: usize              // внутренний отступ
├── margin: usize               // внешний отступ
└── visible: bool               // видимость
```

## Сборка

```bash
cargo check   # проверка компиляции
cargo build   # сборка
```


