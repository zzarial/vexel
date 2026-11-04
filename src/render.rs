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
    draw_element(&mut stdout, app, root, root.style_x(), root.style_y())?;
    stdout.flush()
}

pub fn draw_element(w: &mut io::Stdout, app: &AppState, el: &Element, abs_x: u16, abs_y: u16) -> Result<(), std::io::Error> { 
//draw_element(w, child, x, y) x,y parent cordnt
    let (x, y) = if el.style_position() == Position::Relative {
        (abs_x + app.elements[el.parent].style_x(), abs_y + app.elements[el.parent].style_y())
    } else {
        (abs_x, abs_y)
    };

    match &el.data {
        ElementData::TextData(text) => {
            w.queue(MoveTo(x, y))?;
            w.queue(Print(text))?;
        }
        ElementData::BoxData => draw_box(w, el, x, y)?,
        _ => { /* Button/Input заглушка */ }
    }

    for &child_idx in &el.childrens {
        let child = &app.elements[child_idx];
        draw_element(w, app, child, x, y)?;
    }
    Ok(())
}


fn draw_box(w: &mut io::Stdout, el: &Element, x: u16, y: u16) -> io::Result<()> {
  let (width, height) = (el.style_width(), el.style_height());
  let top    = "┌".to_owned() + &"─".repeat((width - 2) as usize) + "┐";
  let bottom = "└".to_owned() + &"─".repeat((width - 2) as usize) + "┘";
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
