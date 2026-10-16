use crossterm::{
    cursor,
    execute,
    style::{Color, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();

    // Очищаем экран
    execute!(stdout, Clear(ClearType::All))?;

    // Ставим курсор на позицию 5, 3
    execute!(stdout, cursor::MoveTo(5, 3))?;

    // Красим в красный
    execute!(stdout, SetForegroundColor(Color::Red))?;

    // Пишем текст
    print!("Привет, мир!");

    // Возвращаем белый цвет
    execute!(stdout, SetForegroundColor(Color::White))?;

    stdout.flush()?;

    // Ждём нажатия любой клавиши
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(())
}
