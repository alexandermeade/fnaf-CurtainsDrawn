use crate::*;

use crate::gamestate::GameState;
use rodio::Sink;


pub trait GameAudioAsync {
    async fn play_sound(&mut self, src: &str,looping: bool) -> Result<(), Box<dyn std::error::Error>>; 
    async fn stop_all_sounds(&mut self);

}


impl GameAudioAsync for Arc<Mutex<GameState>> {
    async fn play_sound(
        &mut self, 
        src: &str,
        looping: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut game_state = self.lock().await;
        let file = File::open(src)?;
        let source = Decoder::new(BufReader::new(file))?;
        if let Some(out_stream) = &game_state.out_stream {
            let sink = Sink::try_new(&out_stream)?;
            if looping {
                sink.append(source.repeat_infinite());
            }else {
                sink.append(source);
            }
    
            //sin.set_loop(true);

            // Store sink inside the game state so it lives as long as needed
            game_state.sound_sinks.push(sink);
        
        }
        
        Ok(())
    }

    async fn stop_all_sounds(&mut self) {
        let mut game_state = self.lock().await;
        for sink in &game_state.sound_sinks {
            sink.stop();  // Stop playback immediately
        }
        game_state.sound_sinks.clear(); // Clear the list so no references remain
    }

}



