use std::io::{self, Write};
use crate::{AppState, Element, ElementData, Position};

use crossterm::{
      cursor::{MoveTo, Hide},
      style::Print,
      terminal::{Clear, ClearType, enable_raw_mode},
      ExecutableCommand, QueueableCommand,
};

pub fn render(app: &AppState) -> io::Result<()> {
    let mut stdout = io::stdout();

    enable_raw_mode()?;//ручное упроавление
    stdout.execute(Hide)?;//убрать курсор
    stdout.execute(Clear(ClearType::All))?;

    let root = &app.elements[0];
    draw_element(&mut stdout, app, root, 0, 0)?;
    stdout.flush()
}

pub fn draw_element(w: &mut io::Stdout, app: &AppState, el: &Element, parent_x: u16, parent_y: u16) -> Result<(), std::io::Error> {
//draw_element(w, child, parent_x, parent_y) 
    let (x, y) = if el.style_position() == Position::Relative {
        (parent_x + el.style_x(), parent_y + el.style_y())
    } else {
        (el.style_x(), el.style_y())
    };

    if !el.style.visible {
        for &child_idx in &el.children {
            let child = &app.elements[child_idx];
            draw_element(w, app, child, x, y)?;
        }
        return Ok(());
    }

    match &el.data {
        ElementData::TextData(text) => {
            w.queue(MoveTo(x, y))?;
            w.queue(Print(text))?;
        }
        ElementData::BoxData => draw_box(w, el, x, y)?,
        _ => { /* Button/Input заглушка */ }
    }

    for &child_idx in &el.children {
        let child = &app.elements[child_idx];
        draw_element(w, app, child, x, y)?;
    }
    Ok(())
}


fn draw_box(w: &mut io::Stdout, el: &Element, x: u16, y: u16) -> io::Result<()> {
  let (width, height) = (el.style_width(), el.style_height());
  // слишком узкий/низкий для рамки - рисуем вертикальную черту или ничего
  if width < 2 || height < 2 {
      if width >= 1 && height >= 1 {
          w.queue(MoveTo(x, y))?;
          w.queue(Print("│"))?;
      }
      return Ok(());
  }
  let inner = (width - 2) as usize;
  let top    = "┌".to_owned() + &"─".repeat(inner) + "┐";
  let bottom = "└".to_owned() + &"─".repeat(inner) + "┘";
    w.queue(MoveTo(x, y))?;
    w.queue(Print(&top))?;
    for row in 1..height - 1 {
        w.queue(MoveTo(x, y + row))?;
        w.queue(Print("│"))?;
        w.queue(MoveTo(x + width - 1, y + row))?;
        w.queue(Print("│"))?;
    }
    w.queue(MoveTo(x, y + height - 1))?;
    w.queue(Print(&bottom))?;
    Ok(())
}
