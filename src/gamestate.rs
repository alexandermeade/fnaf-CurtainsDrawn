
use crate::*;

use rodio::OutputStreamHandle;
use rodio::Sink;

use num_derive::FromPrimitive; // Derive macro
use num_traits::FromPrimitive;

#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq)]
pub enum Night {
    First = 1,
    Second = 2,
    Third = 3,
    Fourth = 4,
    Fith = 5,
}

pub struct GameState {
    pub logs: Vec<Line<'static>>,
    pub input: String,
    pub scroll_offset: usize, // ← add this!
    pub scroll: usize,
    pub scroll_speed: usize,
    pub anims: Vec<Arc<Mutex<anim::Anim>>>,
    /*pub freddy: Arc<Mutex<anim::Anim>>,     
    pub chica: Arc<Mutex<anim::Anim>>,     
    pub bonnie: Arc<Mutex<anim::Anim>>,     
    pub foxy: Arc<Mutex<anim::Anim>>,     
    pub puppet: Arc<Mutex<anim::Anim>>,*/ 
    pub exit: bool,
    pub room_list: Vec<Arc<Mutex<DestNode>>>,
    
    pub night: Night,
    pub sound_sinks: Vec<Sink>, //this is how audio gets played
    pub out_stream: Option<OutputStreamHandle>,
    pub game_start: bool,
    pub time: (usize, usize), //(hours, seconds)
    pub end_night: bool,

    pub rooted: Option<Arc<Mutex<anim::Anim>>>,


}

impl GameState {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(GameState {
            logs: vec![],
            input: String::new(),
            scroll_offset: 0,
            scroll: 0,
            scroll_speed: 3,
            anims: vec![], /*
            */
            exit: false,
            room_list: vec![],
            night: Night::First,
            sound_sinks: vec![],
            out_stream: None,
            game_start: false,
            time: (0, 0),
            end_night: false,
            rooted: None,
        }))
    }    
}

pub trait GameStateAsync {
    async fn add_anim(&self, anim_type: anim::AnimType, name: &str, tag: &str, start_location: &str, aggression: u64, awareness: u64) -> Arc<Mutex<GameState>>;
    async fn add_room(&self, target:Arc<Mutex<DestNode>>) -> Arc<Mutex<GameState>>;
    async fn night(&self) -> Night;
    async fn set_scroll(&mut self, amount: usize); 
    async fn inc_scroll(&mut self, amount: usize); 
    async fn input(&mut self) -> String;
    async fn exit(&self) -> bool;
    async fn game_started(&self) -> bool;
    async fn toggle_game_active(&self);
    async fn start_clock(&mut self);
    async fn pop_log(&self);
    async fn cleanup_night(&mut self);
    async fn reset_clock(&self);
    async fn night_exit_win(&mut self);
}

impl GameStateAsync for Arc<Mutex<GameState>> { 
    async fn pop_log(& self) {
        let mut game_state = self.lock().await;
        game_state.logs.pop();
    }

    async fn reset_clock(&self) {
        let mut game_state = self.lock().await;
        game_state.time = (0, 0);
    }
    async fn add_anim(&self, anim_type: anim::AnimType, name: &str, tag: &str, start_location: &str, aggression: u64, awareness: u64) -> Arc<Mutex<GameState>> {
        let mut game_state = self.lock().await;
        let anim = anim::Anim::new(anim_type, name, tag, start_location, aggression, awareness);

        for room in   &mut game_state.room_list {
            let room_name = room.name().await;

            if room_name == start_location {
                room.add_anim(anim.clone()).await;
                game_state.anims.push(anim.clone());
                drop(game_state);
                return self.clone();
            }
        }

        println!("\n[ERROR] ANIM NOT ADDED {:#?}", anim);
        return self.clone();

    }

    async fn game_started(&self) -> bool {
        self.lock().await.game_start
    }
    
    async fn toggle_game_active(&self) {
        let mut game_state_lock = self.lock().await;
        
        game_state_lock.game_start = !game_state_lock.game_start; 
    }

    async fn night_exit_win(&mut self) {
        

        self.stop_all_sounds().await;

        let mut game_state_tmp = self.clone();
        
        let welcome_text = fs::read_to_string("./assets/welcome.txt").unwrap();

        let home_menu_text = fs::read_to_string("./assets/home_menu.txt").unwrap();
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

            game_state_tmp.play_sound("./assets/sound/victory.wav", false).await;
            
            game_state_tmp.add_log(format!("{:?} COMPLETED !", night), Color::Cyan, Color::LightRed).await;
            let mut game_state = game_state_tmp.lock().await;
            
            let next_night = game_state.night as usize + 1;

            if let Some(night_tmp) = Night::from_usize(next_night) {
                game_state.night = night_tmp;
            } 

            drop(game_state);

        });
        self.play_sound("./assets/sound/night_finish_alarm.wav", false).await;
        self.cleanup_night().await;
    }

    async fn cleanup_night(&mut self) {
        let mut game_state = self.lock().await;
        game_state.game_start = false;
        game_state.room_list = vec![];
        game_state.anims = vec![];
    }

    async fn start_clock(&mut self) {
        self.reset_clock().await;
        let mut game_state_arc = self.clone();
        tokio::spawn(async move {
            loop {
                thread::sleep(Duration::from_millis(1000));
                let mut game_state = game_state_arc.lock().await;
                if game_state.exit {
                    drop(game_state);
                    break;
                }
                game_state.time.1 += 1;
                if game_state.time.1 >= 60 {
                    game_state.time.1 -= 60;
                    game_state.time.0 += 1;
                }

                if game_state.time.0 >= 6 {
                    drop(game_state);
                    game_state_arc.night_exit_win().await;
                    break;
                }

                for anim_arc in &game_state.anims {
                    let mut anim = anim_arc.lock().await;
                    anim.cooldown = anim.cooldown.saturating_sub(1);
                    if anim.cooldown <= 0 {
                        anim.can_move = true;
                    }
                }

                drop(game_state);
                tokio::task::yield_now().await; // Force context switch
            }
        });
    }

    async fn input(&mut self) -> String {
        let game_state = self.lock().await;
        game_state.input.clone()
    }


    async fn set_scroll(&mut self, amount: usize) {
        let mut game_state = self.lock().await;
        game_state.scroll = amount;
        drop(game_state);
    }
    async fn inc_scroll(&mut self, amount: usize) {
        let mut game_state = self.lock().await;
        game_state.scroll += amount;
        drop(game_state);
    }

    async fn night(&self) -> Night {
        let mut game_state = self.lock().await; 
        game_state.night.clone()
    }

    async fn exit(&self) -> bool {
        let mut state = self.lock().await;
        return state.exit;
    }


    async fn add_room(&self, target:Arc<Mutex<DestNode>>) -> Arc<Mutex<GameState>> {
        let mut state = self.lock().await;
        state.room_list.push(target);
        drop(state);

        return Arc::clone(self);
    }    
}
