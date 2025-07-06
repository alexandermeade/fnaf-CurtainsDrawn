
const GRID_WIDTH: usize = 100;
const GRID_HEIGHT: usize = 40;
const BOX_WIDTH: i32 = 12;
const BOX_HEIGHT: i32 = 3;
const SPACING_X: i32 = BOX_WIDTH + 6;
const SPACING_Y: i32 = BOX_HEIGHT + 3;
const NODE_WIDTH: i32 = 11;
const NODE_HEIGHT: i32 = 3;


use crate::DestNode;
use crate::Direction;

use std::rc::Rc;
use std::cell::RefCell;
use crate::Grid;

use crate::*;


use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::style::{Style, Color};
use ratatui::layout::{Constraint, Layout};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn render_ui(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>, 
    game_state: Arc<Mutex<gamestate::GameState>>,
) {
    let mut game_state_copy = game_state.lock().await;
        let precursor = if let Some(rooted_anim) = &game_state_copy.rooted {
                let mut anim = rooted_anim.lock().await;
                anim.name.clone()
            } else {
                String::new()
            };

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Top bar height
                Constraint::Min(0),     // Fill remaining space
            ])
            .split(f.size());

        // Top bar widget
        let top_bar = Paragraph::new(format!(
            "🧭 Status: Active | Battery: 87% | Employee ID: 00129072 | Connection Status: STRONG | FootPrint: None             Afton co. Animatronic Intefacing Terminal | scroll: {} | time: {}:{} ",
            game_state_copy.scroll,
            game_state_copy.time.0,
            if game_state_copy.time.1 < 10 {format!("0{}", game_state_copy.time.1)} else {game_state_copy.time.1.to_string()}
        ))
        .style(Style::default().bg(Color::Blue).fg(Color::White));
        f.render_widget(top_bar, chunks[0]);

        // Block for logs
        let block = Block::default()
            .title("Terminal")
            .borders(Borders::ALL);
        // Clone logs and add current input line
        let mut lines = game_state_copy.logs.clone();
        lines.push(Line::styled(
            format!("{}> {}",precursor, game_state_copy.input),
            Style::default().fg(Color::Yellow),
        ));

        let total_lines = lines.len();

        // Subtract 2 for block borders (top + bottom)
        let mut visible_height = chunks[1].height.saturating_sub(2) as usize;
        visible_height = visible_height.saturating_sub((visible_height as f32 * 0.03) as i32 as usize);
        // Maximum scroll offset (how many lines can you scroll up)
        let max_scroll = total_lines.saturating_sub(visible_height);

        // Clamp scroll so it doesn’t go out of bounds
        game_state_copy.scroll = game_state_copy.scroll.min(max_scroll);
        // Calculate scroll offset for Paragraph::scroll (lines from top)
        let scroll_offset = max_scroll.saturating_sub(game_state_copy.scroll);

        // Shift viewport up by 1 to make bottom lines fully visible
        let adjusted_scroll_offset = scroll_offset.saturating_sub(1);

        // Calculate scroll offset for Paragraph::scroll (lines from top)

        let paragraph = Paragraph::new(lines)
            .block(block)
            //.wrap(Wrap { trim: false })
            .scroll((adjusted_scroll_offset as u16, 0)); // vertical scroll

        f.render_widget(paragraph, chunks[1]);
    }).unwrap();
}

