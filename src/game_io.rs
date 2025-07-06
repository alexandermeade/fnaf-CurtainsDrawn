
use crate::*;

use crate::gamestate::GameState;

pub trait GameIOAsync {
    async fn add_log(&mut self, content:String, fg: Color, bg: Color); 
    async fn clear_logs(&mut self);
    async fn night_start_text(&mut self); 
    async fn add_line(&mut self, line: Line<'static>);
    async fn styled_replacements(&self, input: String) -> Line<'static>;
}


impl GameIOAsync for Arc<Mutex<GameState>> {

    async fn add_line(&mut self, line: Line<'static>) {
        let mut game_state = self.lock().await;
        game_state.logs.push(line);
    }



    async fn night_start_text(&mut self) {
        let night = self.night().await;
        self.play_sound("./assets/sound/night_start.wav", true).await;
        let night_text:String = match night {
            Night::First => fs::read_to_string("./assets/night1.txt").unwrap(),
            Night::Second => fs::read_to_string("./assets/night2.txt").unwrap(),

            _ => "bleh :p".to_string(),
        };
        for line in night_text.lines() {
            self.add_log(line.to_string(), Color::Green, Color::Reset).await; 
            sleep(Duration::from_millis(10)).await;
        }
    }

    async fn add_log(&mut self, content:String, fg: Color, bg: Color) {
        let mut game_state = self.lock().await;
    // Replace tabs with 4 spaces and pad to 100 width
        const MAX_WIDTH: usize = 100;
        let line = content.replace("\t", "    ");
        let padded = format!("{:<width$}", line, width = MAX_WIDTH);
        let line = Line::styled(padded, Style::default().fg(fg).bg(bg));
        game_state.logs.push(line);
        game_state.scroll = 0;
        //self.logs.push(Line::styled(content, Style::default().fg(fg).bg(bg)));
    }

    async fn clear_logs(&mut self) {
        let mut game_state = self.lock().await;
        game_state.logs = vec![];
        game_state.input = String::new();
        drop(game_state);
    }
    
    async fn styled_replacements(&self, input: String) -> Line<'static> {
        let rooms = {
            let game_state = self.lock().await;
            game_state.room_list.clone()
        };
        let mut spans = Vec::new();
        let mut i = 0;

        // All the patterns you want to replace
        let mut replacements = vec![];
        
        for room in &rooms {
            replacements.push(room.map_replacement().await);
        }
        // Define the color pattern you want for every replacement
        let color_pattern = [
            Color::Red,        // 'r'
            Color::LightBlue,  // number
            Color::Yellow,       // '_'
            Color::Rgb(150, 75, 0),  // a medium brown,
            Color::White,
            Color::Red,
        ];

        let chars = input.chars().collect::<Vec<_>>();

        while i < chars.len() {
            let mut matched = false;

            for (pattern, replacement) in &replacements {
                let pat_len = pattern.len();
                let slice: String = chars[i..].iter().take(pat_len).collect();

                if slice == *pattern {
                    for (j, ch) in replacement.chars().enumerate() {
                        let color = color_pattern.get(j).copied().unwrap_or(Color::Reset);
                        spans.push(Span::styled(ch.to_string(), Style::default().fg(color)).add_modifier(Modifier::BOLD));
                    }
                    i += pat_len;
                    matched = true;
                    break;
                }
            }

            if !matched {
                spans.push(Span::raw(chars[i].to_string()).fg(Color::Green));
                i += 1;
            }
        }

        Line::from(spans)
    }

}
