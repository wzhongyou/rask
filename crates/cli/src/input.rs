use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyModifiers},
    execute,
    style::{self, Attribute, Color},
    terminal::{self, ClearType},
};
use std::io::{stdout, Result, Write};

const COMMANDS: &[(&str, &str)] = &[
    ("/help", "show available commands"),
    ("/model", "switch model: /model <name>"),
    ("/clear", "clear conversation context"),
];

struct RawGuard;
impl RawGuard {
    fn new() -> Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(RawGuard)
    }
}
impl Drop for RawGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

fn filtered_commands(prefix: &str) -> Vec<(&'static str, &'static str)> {
    COMMANDS
        .iter()
        .filter(|(cmd, _)| cmd.starts_with(prefix))
        .copied()
        .collect()
}

fn redraw_input(prompt: &str, buf: &[char], cursor: usize) -> Result<()> {
    let mut out = stdout();
    let s: String = buf.iter().collect();
    execute!(
        out,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::CurrentLine),
        style::Print(prompt),
        style::Print(&s),
        cursor::MoveToColumn((prompt.len() + cursor) as u16),
    )?;
    out.flush()?;
    Ok(())
}

fn draw_popup(
    _prompt: &str,
    items: &[(&str, &str)],
    selected: usize,
) -> Result<()> {
    let mut out = stdout();
    // scroll terminal down to make room, then come back up
    let n = items.len() as u16;
    // print n newlines to push content down, then move back up
    for _ in 0..n {
        execute!(out, style::Print("\r\n"))?;
    }
    execute!(out, cursor::MoveUp(n))?;
    execute!(out, cursor::SavePosition)?;

    for (i, (cmd, desc)) in items.iter().enumerate() {
        execute!(
            out,
            cursor::MoveToNextLine(1),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
        )?;
        if i == selected {
            execute!(
                out,
                style::SetAttribute(Attribute::Bold),
                style::SetForegroundColor(Color::Cyan),
                style::Print(format!("  {:<14}", cmd)),
                style::SetForegroundColor(Color::White),
                style::Print(desc),
                style::ResetColor,
                style::SetAttribute(Attribute::Reset),
            )?;
        } else {
            execute!(
                out,
                style::SetForegroundColor(Color::Cyan),
                style::Print(format!("  {:<14}", cmd)),
                style::SetForegroundColor(Color::DarkGrey),
                style::Print(desc),
                style::ResetColor,
            )?;
        }
    }
    execute!(out, cursor::RestorePosition)?;
    out.flush()?;
    Ok(())
}

fn erase_popup(lines: usize) -> Result<()> {
    if lines == 0 {
        return Ok(());
    }
    let mut out = stdout();
    execute!(out, cursor::SavePosition)?;
    for _ in 0..lines {
        execute!(
            out,
            cursor::MoveToNextLine(1),
            cursor::MoveToColumn(0),
            terminal::Clear(ClearType::CurrentLine),
        )?;
    }
    execute!(out, cursor::RestorePosition)?;
    out.flush()?;
    Ok(())
}

pub fn read_line(prompt: &str, history: &mut Vec<String>) -> Result<Option<String>> {
    let _guard = RawGuard::new()?;
    let mut out = stdout();

    // Print prompt
    execute!(out, style::Print(prompt))?;
    out.flush()?;

    let mut buf: Vec<char> = Vec::new();
    let mut cursor_pos: usize = 0;
    let mut hist_idx: Option<usize> = None; // None = current input
    let mut saved_input: Vec<char> = Vec::new(); // saved when browsing history
    let mut popup_lines: usize = 0;
    let mut popup_sel: usize = 0;

    loop {
        let event = read()?;
        let Event::Key(key) = event else { continue };

        // Compute slash-prefix for popup
        let current_str: String = buf.iter().collect();
        let in_popup = current_str.starts_with('/') && popup_lines > 0;

        match (key.code, key.modifiers) {
            // EOF / interrupt
            (KeyCode::Char('d'), KeyModifiers::CONTROL)
            | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                erase_popup(popup_lines)?;
                execute!(out, style::Print("\r\n"))?;
                out.flush()?;
                return Ok(None);
            }

            // Enter
            (KeyCode::Enter, _) => {
                if in_popup {
                    let items = filtered_commands(&current_str);
                    if !items.is_empty() {
                        let chosen = items[popup_sel].0.to_string();
                        erase_popup(popup_lines)?;
                        popup_lines = 0;
                        buf = chosen.chars().collect();
                        cursor_pos = buf.len();
                        redraw_input(prompt, &buf, cursor_pos)?;
                        // keep popup closed, let user press Enter again to submit
                        continue;
                    }
                }
                erase_popup(popup_lines)?;
                execute!(out, style::Print("\r\n"))?;
                out.flush()?;
                let result: String = buf.iter().collect();
                return Ok(Some(result));
            }

            // Escape: dismiss popup
            (KeyCode::Esc, _) => {
                if popup_lines > 0 {
                    erase_popup(popup_lines)?;
                    popup_lines = 0;
                    popup_sel = 0;
                    redraw_input(prompt, &buf, cursor_pos)?;
                }
            }

            // Up arrow: popup nav or history
            (KeyCode::Up, _) => {
                if in_popup {
                    let items = filtered_commands(&current_str);
                    if !items.is_empty() {
                        erase_popup(popup_lines)?;
                        popup_sel = popup_sel.saturating_sub(1);
                        draw_popup(prompt, &items, popup_sel)?;
                        popup_lines = items.len();
                    }
                } else {
                    // history navigation
                    if history.is_empty() {
                        continue;
                    }
                    if hist_idx.is_none() {
                        saved_input = buf.clone();
                    }
                    let new_idx = match hist_idx {
                        None => history.len() - 1,
                        Some(0) => 0,
                        Some(i) => i - 1,
                    };
                    hist_idx = Some(new_idx);
                    erase_popup(popup_lines)?;
                    popup_lines = 0;
                    buf = history[new_idx].chars().collect();
                    cursor_pos = buf.len();
                    redraw_input(prompt, &buf, cursor_pos)?;
                }
            }

            // Down arrow: popup nav or history
            (KeyCode::Down, _) => {
                if in_popup {
                    let items = filtered_commands(&current_str);
                    if !items.is_empty() {
                        erase_popup(popup_lines)?;
                        popup_sel = (popup_sel + 1).min(items.len() - 1);
                        draw_popup(prompt, &items, popup_sel)?;
                        popup_lines = items.len();
                    }
                } else {
                    match hist_idx {
                        None => {}
                        Some(i) if i + 1 >= history.len() => {
                            hist_idx = None;
                            buf = saved_input.clone();
                            cursor_pos = buf.len();
                            redraw_input(prompt, &buf, cursor_pos)?;
                        }
                        Some(i) => {
                            hist_idx = Some(i + 1);
                            buf = history[i + 1].chars().collect();
                            cursor_pos = buf.len();
                            redraw_input(prompt, &buf, cursor_pos)?;
                        }
                    }
                }
            }

            // Left arrow
            (KeyCode::Left, _) => {
                if cursor_pos > 0 {
                    cursor_pos -= 1;
                    execute!(out, cursor::MoveToColumn((prompt.len() + cursor_pos) as u16))?;
                    out.flush()?;
                }
            }

            // Right arrow
            (KeyCode::Right, _) => {
                if cursor_pos < buf.len() {
                    cursor_pos += 1;
                    execute!(out, cursor::MoveToColumn((prompt.len() + cursor_pos) as u16))?;
                    out.flush()?;
                }
            }

            // Home
            (KeyCode::Home, _) => {
                cursor_pos = 0;
                execute!(out, cursor::MoveToColumn(prompt.len() as u16))?;
                out.flush()?;
            }

            // End
            (KeyCode::End, _) => {
                cursor_pos = buf.len();
                execute!(out, cursor::MoveToColumn((prompt.len() + cursor_pos) as u16))?;
                out.flush()?;
            }

            // Backspace
            (KeyCode::Backspace, _) => {
                if cursor_pos > 0 {
                    erase_popup(popup_lines)?;
                    popup_lines = 0;
                    buf.remove(cursor_pos - 1);
                    cursor_pos -= 1;
                    redraw_input(prompt, &buf, cursor_pos)?;
                    // re-evaluate popup
                    let s: String = buf.iter().collect();
                    if s.starts_with('/') {
                        let items = filtered_commands(&s);
                        if !items.is_empty() {
                            popup_sel = popup_sel.min(items.len().saturating_sub(1));
                            draw_popup(prompt, &items, popup_sel)?;
                            popup_lines = items.len();
                        }
                    } else {
                        popup_sel = 0;
                    }
                }
            }

            // Delete
            (KeyCode::Delete, _) => {
                if cursor_pos < buf.len() {
                    erase_popup(popup_lines)?;
                    popup_lines = 0;
                    buf.remove(cursor_pos);
                    redraw_input(prompt, &buf, cursor_pos)?;
                    let s: String = buf.iter().collect();
                    if s.starts_with('/') {
                        let items = filtered_commands(&s);
                        if !items.is_empty() {
                            popup_sel = popup_sel.min(items.len().saturating_sub(1));
                            draw_popup(prompt, &items, popup_sel)?;
                            popup_lines = items.len();
                        }
                    } else {
                        popup_sel = 0;
                    }
                }
            }

            // Regular character
            (KeyCode::Char(c), m) if m == KeyModifiers::NONE || m == KeyModifiers::SHIFT => {
                hist_idx = None;
                erase_popup(popup_lines)?;
                popup_lines = 0;
                buf.insert(cursor_pos, c);
                cursor_pos += 1;
                redraw_input(prompt, &buf, cursor_pos)?;
                let s: String = buf.iter().collect();
                if s.starts_with('/') {
                    let items = filtered_commands(&s);
                    if !items.is_empty() {
                        popup_sel = 0;
                        draw_popup(prompt, &items, popup_sel)?;
                        popup_lines = items.len();
                    }
                }
            }

            _ => {}
        }
    }
}
