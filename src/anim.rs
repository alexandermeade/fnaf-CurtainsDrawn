
use crate::*;

use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub enum AnimType {
    Freddy = 3,
    Bonnie = 1,
    Chica = 2,
    Puppet = 4,
    Foxy = 5,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TravelMethod {
    Shortest = 1,
    MostEmpty = 2,
    None = 3,
}


#[derive(Debug)]
pub struct Anim {
    pub anim_type: AnimType,
    pub name: String,
    pub tag: String,
    pub start_location: String,
    pub location: Option<Arc<Mutex<DestNode>>>,
    status: String,
    aggression: u64,
    awareness: u64,
    hidden: bool,
    pub move_delay: u64,
    pub cooldown: u64,
    is_cooling_down: bool,
    pub can_move: bool,
    executing: bool,
}

impl Anim {
    pub fn new(anim_type: AnimType, name: &str, tag: &str, start_location: &str, aggression: u64, awareness: u64) -> Arc<Mutex<Anim>> {
        let delay = match anim_type.clone() {
                AnimType::Bonnie | AnimType::Chica => 30,
                AnimType::Freddy => 25,
                AnimType::Foxy => 60,
                AnimType::Puppet => 70,
        };
        Arc::new(Mutex::new(Anim {
            anim_type: anim_type.clone(),
            name:name.to_string(),
            tag:tag.to_string(),
            start_location: start_location.to_string(),
            location: None,
            status: "idk bruh".to_string(),
            aggression,
            awareness,
            hidden: false,
            move_delay:delay, 
            cooldown: delay,
            is_cooling_down: false,
            can_move: false,
            executing: false,
        }))
    }        
}

pub trait AnimAsync {
    async fn move_cooldown(&self);
    async fn set_location(&self, location: Arc<Mutex<destNode::DestNode>>);
    async fn move_anim(&self);
    async fn can_move(&self) -> bool;
    async fn on_cooldown(&self) -> bool;
    async fn root_into(&self, game_state: Arc<Mutex<gamestate::GameState>>);

    async fn execute(&self, game_state: Arc<Mutex<gamestate::GameState>>, args: Vec<String>);

    async fn roll(&self, chance_percent: f32) -> bool;
}

impl AnimAsync for Arc<Mutex<Anim>> {
    
    async fn roll(&self, chance_percent: f32) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen_range(0.0..100.0) < chance_percent
    }

    async fn root_into(&self, game_state: Arc<Mutex<gamestate::GameState>>) {

        let mut game_state = game_state.clone();
        let mut anim_arc = self.clone();

        let mut anim = anim_arc.lock().await;
        if anim.executing {
            drop(anim);
            return;
        }
        drop(anim);

        tokio::spawn(async move {

            let mut anim = anim_arc.lock().await;
            anim.executing = true;
            let awareness = anim.awareness;
            let name = anim.name.clone();
            let tag = anim.tag.clone();
            drop(anim);

            //100% chance of success minus anim's awareness value 

            let result = anim_arc.roll((100 - awareness) as f32).await;
            let cap = 20;
              
            for i in (0..=cap) { 

                if i > 0 {
                    game_state.pop_log().await;
                }
                game_state.add_log(format!("Rooting [{}{}]", "█".repeat(i), " ".repeat(cap - i)), Color::Green, Color::Reset).await;

                tokio::task::yield_now().await; // Force context switch
                sleep(Duration::from_millis((10 + awareness) as u64)).await;

            }

             
            if result {

                let mut anim = anim_arc.lock().await;
                anim.awareness += 5;

                anim.executing = false;
                drop(anim); //free up the lock on anim
                let mut game_state_lock = game_state.lock().await;
                game_state_lock.rooted = Some(anim_arc.clone());
                drop(game_state_lock); //free up the lock on game_state

                game_state.add_log(format!("[SUCCESS] rooted into {} {}", name, tag), Color::White, Color::Green).await;
                
                return;   
            } 
            
            let mut anim = anim_arc.lock().await;
            anim.executing = false;
            drop(anim);
            game_state.add_log(format!("[ERROR] Failed to root into {} {}", name, tag), Color::Black, Color::Red).await;
            game_state.play_sound("./assets/sound/error.wav", false).await;
        });

    }

    async fn on_cooldown(&self) -> bool {
        self.lock().await.is_cooling_down
    }

    async fn can_move(&self) -> bool {
        self.lock().await.can_move
    }

    async fn move_cooldown(&self) {
        let mut anim = self.lock().await;
        anim.cooldown = anim.move_delay;
    }

    async fn set_location(&self, location: Arc<Mutex<destNode::DestNode>>) {
        self.lock().await.location = Some(Arc::clone(&location));
    }


    async fn move_anim(&self) {
        // First: Lock the anim briefly to check its move state
        let should_start_cooldown = {
            let anim = self.lock().await;
            !anim.can_move && !anim.is_cooling_down
        };

        let should_stop = { 
            let anim = self.lock().await;
            !anim.can_move && anim.is_cooling_down
        };
        
        if should_stop {
            return;
        }

        if should_start_cooldown {
            self.move_cooldown().await;
            return;
        }
        

        let (mut location_arc, anim_type) = {
            let anim = self.lock().await;
            if !anim.can_move {
                return;
            }

            (
                anim.location.clone().expect("LOCATION IS NONE"),
                anim.anim_type.clone()
            )
        };

        // Lock the location
        let location = location_arc.lock().await;
        let anim_type =  {
            let anim = self.lock().await;
            anim.anim_type.clone()
        };
        let curr_dist = location.dist_to_office;
        let method = match anim_type {
                AnimType::Bonnie | AnimType::Chica => TravelMethod::Shortest,
                AnimType::Freddy => TravelMethod::MostEmpty, //TravelMethod::MostEmpty,
                AnimType::Foxy => TravelMethod::None,//TravelMethod::Shortest,
                AnimType::Puppet => TravelMethod::None,//TravelMethod::Shortest,
        };
        // Find valid nodes without holding other locks
        let mut possible_connections = vec![];
        for node_arc in &location.connections {
            let node = node_arc.lock().await;
            match method {
                TravelMethod::Shortest => if node.dist_to_office <= curr_dist {
                    possible_connections.push(Arc::clone(node_arc));
                },
                TravelMethod::MostEmpty => if node.anims.len() <= 1 {
                    possible_connections.push(Arc::clone(node_arc));
                },
                TravelMethod::None => {
                },

            }
        }

        drop(location); // done with the current location

        if possible_connections.is_empty() {
            return; // No valid moves
        }


        // get all names first (async)
        let names: Vec<String> = futures::future::join_all(
            possible_connections.iter().map(|node_arc| async {
                node_arc.name().await
            })
        ).await;

        // now create RNG and generate index (sync, no await)
        let index = {
            let mut rng = rand::thread_rng();
            rng.gen_range(0..possible_connections.len())
        };
        let mut new_location_arc = Arc::clone(&possible_connections[index]);

        // Call add_anim and remove_anim outside the anim lock
        new_location_arc.add_anim(Arc::clone(&self)).await;
        
        location_arc.remove_anim(anim_type.clone()).await;

        // Update anim location
        let mut anim = self.lock().await;
        anim.location = Some(new_location_arc);
        anim.can_move = false;
        drop(anim);
        self.move_cooldown().await;
    }


    async fn execute(&self, mut game_state: Arc<Mutex<gamestate::GameState>>, args: Vec<String>) {

        let mut anim = self.lock().await;
        if anim.executing { 
            game_state.add_log(format!("Already executing a command please wait..."), Color::White, Color::Green).await;
            return;
        }

        anim.executing = true;
        drop(anim);
        
        if let Some(command) = args.get(0) {
            match command.as_str() {
                "unroot" => {
                    let mut game_state_lock = game_state.lock().await;
                    game_state_lock.rooted = None;
                    drop(game_state_lock);
                    game_state.add_log(format!("unrooted successful"), Color::White, Color::Green).await;
                },
                "clear" => {
                    let mut game_state_lock = game_state.lock().await;
                    game_state_lock.logs = vec![];
                    game_state_lock.input = String::new();
                },
                "status" => {
                    let anim_arc = self.clone();
                    let mut game_state_arc = game_state.clone();
                    tokio::spawn(async move {

                        let mut anim = anim_arc.lock().await;
                        let awareness = anim.awareness;
                        drop(anim);
                        //100% chance of success minus anim's awareness value 
                        let cap = 20;
                        
                        for i in (0..=cap) { 
                            if i > 0 {
                                game_state_arc.pop_log().await;
                            }
                            game_state_arc.add_log(format!("Retreueveubg status [{}{}]", "█".repeat(i), " ".repeat(cap - i)), Color::Green, Color::Reset).await;
                            tokio::task::yield_now().await; // Force context switch
                            sleep(Duration::from_millis((25 + awareness) as u64)).await; 

                        }

                         
                        let mut anim = anim_arc.lock().await;
                        let name = anim.name.clone();
                        let tag = anim.tag.clone();
                        anim.awareness += 5;
                        drop(anim); //free up the lock on anim
                        
                        let mut game_state_arc_lock = game_state_arc.lock().await;
                        game_state_arc_lock.rooted = Some(anim_arc.clone());
                        drop(game_state_arc_lock); //free up the lock on game_state_arc
                        
                        game_state_arc.add_log(format!("[SUCCESS] Retrieved Status of {} {}", name, tag), Color::White, Color::Green).await;
                        game_state_arc.add_log(format!("internal errors:  {} {}", name, tag), Color::White, if awareness >= 50 {Color::Red} else if awareness >= 25 {Color::Yellow} else {Color::Green}).await;

                        game_state_arc.add_log(format!("internal errors:  {} {}", name, tag), Color::White, if awareness >= 50 {Color::Red} else if awareness >= 25 {Color::Yellow} else {Color::Green}).await;
                    }); 
                },
                "help" => {
                    let help_text = fs::read_to_string("./assets/rooted_help.txt").unwrap();

                    for line in help_text.lines() {
                        game_state.add_log(format!("{}", line), Color::Green, Color::Reset).await;
                    }

                },

                cmd => {
                    game_state.add_log(format!("unknown command {}", cmd), Color::White, Color::Red).await;
                },

            }
            
        }
        let mut game_state_lock = game_state.lock().await;
        game_state_lock.input = String::new();
        let mut anim = self.lock().await;

        anim.executing = false;
        drop(anim);
        drop(game_state_lock);
    }


}
