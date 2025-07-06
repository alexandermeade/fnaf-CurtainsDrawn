// Standard Library
use std::{
    cell::RefCell,
    fs::{self, File},
    io::{self, BufReader, stdout, Write},
    rc::Rc,
    sync::Arc,
    thread,
};

use crate::gamestate::Night;
// External Crates
use chrono::Local;
use crossterm::{
    cursor::*,
    event::{self, *},
    execute,
    terminal::*,
};
use ratatui::{
    prelude::*,
    style::*,
    widgets::*,
    Terminal,
};
use rodio::{Decoder, OutputStream, Source};

// Async
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

use crate::game_audio::GameAudioAsync;
use crate::gamestate::GameStateAsync;

use crate::game_io::GameIOAsync;
use crate::game_logic::GameLogicAsync;

// Internal Modules
mod anim;
mod destNode;
mod visual;
mod gamestate;
mod nights;
mod game_audio;
mod game_io;
mod game_logic;
mod game_input;

use crate::game_input::GameInputAsync;

use crate::destNode::{DestNode, DestNodeFns, Direction};


use std::io::Read;
type Grid = Vec<Vec<char>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let stream_handle_clone = stream_handle.clone();

    // Spawn the clock task
    //
    let power = Arc::new(Mutex::new(100));
    let power_clone = Arc::clone(&power);

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut inputs = String::new();
    let mut input = String::new();
    let mut fg = Color::White;
    let mut bg = Color::Reset;
    let mut gameState = gamestate::GameState::new();          
    gameState.lock().await.out_stream = Some(stream_handle);   
   

    let welcome_text = fs::read_to_string("./assets/welcome.txt")?;

    let home_menu_text = fs::read_to_string("./assets/home_menu.txt")?;
    let help_text = fs::read_to_string("./assets/help.txt")?;

    let map_text = fs::read_to_string("./assets/map.txt")?;

    let mut game_state_tmp = Arc::clone(&gameState);
        //start up text
        //
        //

    //let game_state_tmp_bgth = Arc::clone(&gameState);
    /*tokio::spawn(async move{
            let mut state = game_state_tmp_bgth.lock().await;
            let mut buf = [0u8; 1];
            let result = io::stdin().read(&mut buf);
            if result.is_err() || result.unwrap_or(0) == 0 {

                state.stop_all_sounds(); // or other shutdown logic
                eprintln!("Terminal disconnected.");
                std::process::exit(0);
            }
        }
    );*/
    tokio::spawn(async move {
        let mut delay:usize = 120;
        let mut factor: usize = 1;
 
        //let mut state = game_state_tmpii.lock().await;
        
        game_state_tmp.play_sound("./assets/sound/pc-start-63725(1).wav", false).await;

        for line in welcome_text.lines() {
            game_state_tmp.add_log(line.to_string(), Color::Green, Color::Reset).await;
            sleep(Duration::from_millis(delay.try_into().unwrap())).await;
            if delay > 2 {
                delay = delay.saturating_sub(factor);

                factor += 1;
            }
        }
        
        game_state_tmp.clear_logs().await;
        for line in home_menu_text.lines() {
            let line = line.replace("\t", "    "); // Replace tab with 4 spaces
            game_state_tmp.add_log(line.to_string(), Color::Green, Color::Black).await;
        }

        game_state_tmp.set_scroll(0).await;
        let night = game_state_tmp.night().await; 
        game_state_tmp.add_log(format!("type continue to continue night {:?}", night), Color::Green, Color::Reset).await;
        game_state_tmp.play_sound("./assets/sound/fnaf1-ambience.wav", true).await;
    });

    /*
    // Load the audio file
    let _ = {
        let mut game_state = gameState.lock().await;
        game_state.play_sound(stream_handle, "./assets/sound/pc-start-63725(1).mp3").await;
    };*/

    loop {
        fg = Color::White;
        bg = Color::Reset;
        
        //handle inputs
        
        if gameState.exit().await {
            break;
        }
        
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                gameState.process_input(key, &mut terminal).await; 
            }
        }

        gameState.update().await;
        visual::render_ui(&mut terminal, Arc::clone(&gameState)).await;
    }

    // Clean up
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    println!("You typed: {}", input);
    Ok(())
    /* 
    print!("\n>");
    io::stdout().flush().unwrap();

    let mut input = String::new();

    io::stdin().read_line(&mut input).expect("Failed to read line");
    let name = input.trim();
    println!("Hello, {}!", name);
*/
}
