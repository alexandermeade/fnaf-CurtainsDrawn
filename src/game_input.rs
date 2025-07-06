use crate::*;
use crate::gamestate::GameState;

use crate::anim::AnimAsync;
pub trait GameInputAsync {
    async fn process_input_home(&mut self, key: KeyEvent, terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>);
    async fn process_input_night1(&mut self, key: KeyEvent, terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>); 
    async fn process_input(&mut self, keycode: KeyEvent, terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>); 
}

impl GameInputAsync for Arc<Mutex<GameState>> {
    async fn process_input(&mut self, keycode: KeyEvent, terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>) {        
        let night = self.night().await;
        if !self.game_started().await {
            self.process_input_home(keycode, terminal).await;
            return;
        }
        match night {
            Night::First => self.process_input_night1(keycode, terminal).await,
            Night::Second => self.process_input_night1(keycode, terminal).await,
            _ => todo!("not in yet"),
        }
    }

    async fn process_input_home(&mut self, key: KeyEvent, terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>) {
        let help_text = fs::read_to_string("./assets/help.txt").unwrap();
        let map_text = fs::read_to_string("./assets/map.txt").unwrap();

        let mut fg = Color::White;
        let mut bg = Color::Reset;

        match key.code {
            KeyCode::Char(c) => {
                let mut game_state = self.lock().await;
                game_state.input.push(c);
                drop(game_state);
            },
            KeyCode::Backspace => {
                let mut game_state = self.lock().await;
                game_state.input.pop();
                drop(game_state);
            }
            KeyCode::Enter => {
                let input = self.input().await;
                let commands: Vec<String> = input.split_whitespace().collect::<Vec<&str>>().into_iter().map(|s| s.to_string()).collect::<Vec<String>>();
                if commands.len() <= 0 {
                    return;
                }
                let mut output = String::new();
                if let Some(command) = commands.get(0) {
                    output = match command.as_str() {
                        "clear" => {
                            let mut game_state = self.lock().await;
                            game_state.logs = vec![];
                            game_state.input = String::new();
                            return;
                            String::from("")
                        },
                        "help" => { 
                            fg = Color::Green;
                            format!("{}", help_text.clone())
                        },
                        "map" => {
                            let mut game_state = self.lock().await;
                            fg = Color::Green;
                            game_state.scroll = 0;
                            drop(game_state);
                            map_text.clone()
                        },
                        "exit-game" | "quit-game" => {
                            let mut game_state = self.lock().await;
                            game_state.exit = true;
                            drop(game_state);
                            String::from("")
                        }, 
                        "start" => {
                            let mut game_state = self.lock().await;
                            drop(game_state);
                            self.clear_logs().await;                        
                            self.night_start_text().await;
                            let mut game_state = Arc::clone(&self);
                            tokio::spawn(async move {
                                nights::start_game(&mut game_state).await;
                            });
                            String::from("")
                        },
                        _ => { 
                            bg = Color::Red;
                            format!("unknown command: {}", self.input().await)
                        },
                    };
                }

                if self.input().await.as_str() == "exit" {
                    let mut game_state = self.lock().await;
                    game_state.exit = true;
                    drop(game_state);
                }

                let input = self.input().await;

                self.add_log(format!("> {} ", input), fg, bg).await;
                for line in output.lines() {
                    self.add_log(format!("{}", line), fg, bg).await;
                    self.set_scroll(0);
                }
                let mut game_state = self.lock().await;
                game_state.input = String::new();
                drop(game_state);
            }
            KeyCode::Esc => {},
            KeyCode::Up => {
                let mut game_state = self.lock().await;
                if game_state.scroll < game_state.logs.len() {
                    game_state.scroll += 1;
                }
                drop(game_state);
            }
            KeyCode::Down => {
                let mut game_state = self.lock().await;
                if game_state.scroll > 0 {
                    game_state.scroll -= 1;
                }
                drop(game_state);
            }
            _ => {}
        }
    }

    async fn process_input_night1(&mut self, key: KeyEvent, terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>) {
        let help_text = fs::read_to_string("./assets/help.txt").unwrap();
        
        let mut fg = Color::White;
        let mut bg = Color::Reset;
        
        match key.code {
            KeyCode::Char(c) => {
                let mut game_state = self.lock().await;
                game_state.input.push(c);
                drop(game_state);
            },
            KeyCode::Backspace => {
                let mut game_state = self.lock().await;
                game_state.input.pop();
                drop(game_state);
            }
            KeyCode::Enter => {
                let input = self.input().await.to_lowercase();
                let commands: Vec<String> = input.split_whitespace().collect::<Vec<&str>>().into_iter().map(|s| s.to_string()).collect::<Vec<String>>();

                if commands.len() <= 0 {
                    return;
                }

                let mut output = String::new();
                let game_state = self.lock().await;
                let rooted = game_state.rooted.clone();
                drop(game_state);
                if let Some(anim_arc) = rooted {
                    anim_arc.execute(self.clone(), commands.clone()).await;
                    return;
                } else if let Some(command) = commands.get(0) {
                    output = match command.as_str() {
                        "error-logs" => {
                            "blah blah blah probably some lore".to_string()
                        },
                        "ping-near" => { 
                            fg = Color::Green;
                            bg = Color::Reset;
                            "pinging near ···".to_string()
                        }, 
                        "clear" => {
                            let mut game_state = self.lock().await;
                            game_state.logs = vec![];
                            game_state.input = String::new();
                            "".to_string()
                        },
                        "help" => { 
                            fg = Color::Green;
                            help_text.clone()
                        },
                        "anims" => {
                            let anims =  {
                                let game_state = self.lock().await;
                                game_state.anims.clone()
                            };
                            for anim_arc in anims {
                                let anim = anim_arc.lock().await;
                                self.add_log(format!("{} {} | {}   \\ {}", anim.name, anim.cooldown, anim.move_delay, anim.can_move), Color::White, Color::Red).await;
                            }

                            "ran anims".to_string()
                        }
                        "map" => {
                            if commands.len() > 1 {
                                self.add_log(format!("> {} ", input), Color::White, Color::Red).await;
                                self.add_log(format!("[ERROR] too many arguments the command \"map\" takes zero extra arguments"), Color::White, Color::Red).await;
                                let mut game_state = self.lock().await;
                                game_state.input = String::new();
                                drop(game_state);
                                return;
                            }

                            let map_text = fs::read_to_string("./assets/map.txt").unwrap();
                            let mut game_state = self.lock().await;
                            fg = Color::Green;
                            game_state.scroll = 0;
                            
                            let input = game_state.input.clone(); 
                            game_state.input = String::new();
                            drop(game_state);

                            self.add_log(format!("> {} ", input), fg, bg).await;

                            for line in map_text.lines() {
                                self.add_line(self.styled_replacements(line.to_string()).await).await;
                                tokio::task::yield_now().await; // Force context switch
                            }
                            return;
                        },
                        "root" => {  
                            
                            let mut game_state = self.lock().await;

                            if let Some(rooted) = &game_state.rooted {
                                game_state.rooted = None;
                            }

                            if let Some(target) = commands.get(1) {

                                for anim in &game_state.anims {
                                    let anim_lock = anim.lock().await;
                                    let name = anim_lock.name.clone();
                                    drop(anim_lock);

                                    if name == *target {
                                        anim.root_into(self.clone()).await;

                                        drop(game_state);
                                        self.clear_logs().await;
                                        return;
                                    }
                                }
                                "no anim was found".to_string()
                            } else {
                                "you need to specfiy the animatronic to root into".to_string()
                            }
                            
                        },
                        "intercom" => {
                            if let Some(target) = commands.get(1) {
                                let mut game_state = self.lock().await;   
                                for room_arc in &game_state.room_list {
                                    room_arc.intercom().await;
                                }

                                "successfully intercomed".to_string()
                            } else {
                                "already intercom".to_string()
                            }
                    
                        },
                        "continue" => {                    
                            let mut game_state = self.lock().await;
                            game_state.logs = vec![];
                            game_state.input = String::new();
                            String::from("")
                        },
                        "root" => {
                            "rooting".to_string()
                        },
                        "" => "".to_string(),
                        _ => { 
                            bg = Color::Red;
                            format!("unknown command: {}", self.input().await)
                        },
                    };
                }

                if self.input().await.as_str() == "exit-game" {
                    let mut game_state = self.lock().await;
                    game_state.exit = true;
                    drop(game_state);
                }

                let input = self.input().await.clone(); 
                let mut game_state = self.lock().await;
                let precursor = if let Some(rooted_anim) = &game_state.rooted {
                    let anim  = rooted_anim.lock().await;
                    let name = anim.name.clone();
                    drop(anim);
                    name
                } else {
                    String::from("")
                };
                drop(game_state);
                self.add_log(format!("{}> {} ", precursor, input), fg, bg).await;
                for line in output.lines() {
                    self.add_log(format!("{}", line), fg, bg).await;
                    self.set_scroll(0).await; 
                }

                let mut game_state = self.lock().await;
                game_state.input = String::new();
            }
            KeyCode::Esc => {},
            KeyCode::Up => {
                let mut game_state = self.lock().await;
                if game_state.scroll < game_state.logs.len() {
                    game_state.scroll += 1;
                }
            }
            KeyCode::Down => {
                let mut game_state = self.lock().await;
                if game_state.scroll > 0 {
                    game_state.scroll -= 1;
                }
            }
            _ => {}
        }
    }
}


