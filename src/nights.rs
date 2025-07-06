use crate::*;

use rodio::OutputStreamHandle;
use std::{fs, time::Duration};
use std::sync::Arc;
use tokio::time::sleep;
use tokio::sync::Mutex;
use ratatui::Terminal;
use ratatui::style::Color;

use crate::gamestate::GameStateAsync;
use crate::gamestate::{self, Night};

pub async fn start_game(
    game_state: &mut Arc<Mutex<gamestate::GameState>>
) {
    let mut game_state_for_spawn = Arc::clone(&game_state);
    /*tokio::spawn(async move {
        game_state_for_spawn.stop_all_sounds().await;
         
        let mut delay: usize = 100;
        let mut factor: usize = 1;
        
        game_state_for_spawn.clear_logs().await;
        game_state_for_spawn.play_sound("./assets/sound/pc-start-63725(1).wav", false).await;
        
        let night_text = match game_state_for_spawn.night().await {
            Night::First =>  fs::read_to_string("./assets/welcome.txt").unwrap(),
            _ => "Not implemented".to_string(),
        }; 

        for line in night_text.lines() {
            game_state_for_spawn.add_log(line.to_string(), Color::Green, Color::Reset).await;
            sleep(Duration::from_millis(delay as u64)).await;

            if delay > 2 {
                delay = delay.saturating_sub(factor);
                factor += 1;
            }
        }        
        game_state_for_spawn.clear_logs().await;


        let night_text = match fs::read_to_string("./assets/night1.txt") {
            Ok(text) => text,
            Err(e) => {
                eprintln!("Error reading night1.txt: {}", e);
                return;
            }
        };

        for line in night_text.lines() {
            let line = line.replace("\t", "    ");
            game_state_for_spawn.add_log(line, Color::Green, Color::Black).await;
        }

        game_state_for_spawn.set_scroll(0).await;
        game_state_for_spawn.play_sound("./assets/sound/fnaf1-ambience.wav", true).await;

    });*/
    game_state.reset_clock().await;
    match game_state.night().await {
        Night::First => {
            nights::start_night1(&mut Arc::clone(&game_state)).await;
        },
        Night::Second => {
            nights::start_night2(&mut Arc::clone(&game_state)).await;
        },
        _ => todo!("haven't added this night yet"),
    }
}

pub async fn start_night2(game_state: &mut Arc<Mutex<gamestate::GameState>>) {

    let main_floor_name = "mainFloor";

    let mut officeNode = destNode::DestNode::new("office", "X", 0,'&',true);
    let mut utilsNode = destNode::DestNode::new("utils", "0x3f5", 2, '*', false);
    let mut mainFloor = destNode::DestNode::new(main_floor_name, "0x4a1", 4,'!', false);
    let mut kitchenNode = destNode::DestNode::new("kitchen", "0x7b2", 3,'$', false);
    let mut playRoom = destNode::DestNode::new("playplace", "0x2z2", 3,'#', false);
    let mut entranceRoom = destNode::DestNode::new("entrance", "0x14e", 2,'-', false);
    game_state
            .add_room(Arc::clone(&officeNode)).await
            .add_room(Arc::clone(&utilsNode)).await
            .add_room(Arc::clone(&mainFloor)).await
            .add_room(Arc::clone(&kitchenNode)).await
            .add_room(Arc::clone(&playRoom)).await
            .add_room(Arc::clone(&entranceRoom)).await;

    //set up the nodes
    //set up distances mainually
    


    officeNode
        .add_connection(&entranceRoom).await
        .add_connection(&utilsNode).await;
            
    utilsNode.add_connection(&officeNode).await
             .add_connection(&mainFloor).await;

    mainFloor.add_connection(&utilsNode).await 
             .add_connection(&kitchenNode).await 
             .add_connection(&playRoom).await;    

    playRoom.add_connection(&mainFloor).await
            .add_connection(&entranceRoom).await
            .add_connection(&utilsNode).await;

    
    kitchenNode
        .add_connection(&mainFloor).await
        .add_connection(&entranceRoom).await;
    
    entranceRoom
        .add_connection(&kitchenNode).await
        .add_connection(&officeNode).await
        .add_connection(&playRoom).await;
    /*
    mainFloor.add_anim(game_state.freddy().await).await;
    mainFloor.add_anim(game_state.bonnie().await).await;
    mainFloor.add_anim(game_state.chica().await).await;

    playRoom.add_anim(game_state.foxy().await).await;
    playRoom.add_anim(game_state.puppet().await).await;
    */

    game_state.add_anim(anim::AnimType::Freddy, "freddy", "284cx", main_floor_name, 1, 1).await     
              .add_anim(anim::AnimType::Chica, "chica", "1231ss", main_floor_name, 1, 1).await    
              .add_anim(anim::AnimType::Bonnie, "bonnie", "la201", main_floor_name, 1, 1).await   
              .add_anim(anim::AnimType::Foxy,"foxy", "lmmm10", main_floor_name, 1, 1).await     
              .add_anim(anim::AnimType::Puppet, "puppet", "balls12", main_floor_name, 1, 1).await; 





    let game_state_lock = game_state.lock().await;
    drop(game_state_lock);
    game_state.start_clock().await;
    game_state.toggle_game_active().await;   
    println!("STARTED NIGHT 2");
}

pub async fn start_night1(game_state: &mut Arc<Mutex<gamestate::GameState>>) {


    let main_floor_name = "mainFloor";

    let mut officeNode = destNode::DestNode::new("office", "X", 0,'&',true);
    let mut utilsNode = destNode::DestNode::new("utils", "0x3f5", 2, '*', false);
    let mut mainFloor = destNode::DestNode::new(main_floor_name, "0x4a1", 4,'!', false);
    let mut kitchenNode = destNode::DestNode::new("kitchen", "0x7b2", 3,'$', false);
    let mut playRoom = destNode::DestNode::new("playplace", "0x2z2", 3,'#', false);
    let mut entranceRoom = destNode::DestNode::new("entrance", "0x14e", 2,'-', false);
    game_state
            .add_room(Arc::clone(&officeNode)).await
            .add_room(Arc::clone(&utilsNode)).await
            .add_room(Arc::clone(&mainFloor)).await
            .add_room(Arc::clone(&kitchenNode)).await
            .add_room(Arc::clone(&playRoom)).await
            .add_room(Arc::clone(&entranceRoom)).await;

    //set up the nodes
    //set up distances mainually
    


    officeNode
        .add_connection(&entranceRoom).await
        .add_connection(&utilsNode).await;
            
    utilsNode.add_connection(&officeNode).await
             .add_connection(&mainFloor).await;

    mainFloor.add_connection(&utilsNode).await 
             .add_connection(&kitchenNode).await 
             .add_connection(&playRoom).await;    

    playRoom.add_connection(&mainFloor).await
            .add_connection(&entranceRoom).await
            .add_connection(&utilsNode).await;

    
    kitchenNode
        .add_connection(&mainFloor).await
        .add_connection(&entranceRoom).await;
    
    entranceRoom
        .add_connection(&kitchenNode).await
        .add_connection(&officeNode).await
        .add_connection(&playRoom).await;
    /*
    mainFloor.add_anim(game_state.freddy().await).await;
    mainFloor.add_anim(game_state.bonnie().await).await;
    mainFloor.add_anim(game_state.chica().await).await;

    playRoom.add_anim(game_state.foxy().await).await;
    playRoom.add_anim(game_state.puppet().await).await;
    */

    game_state.add_anim(anim::AnimType::Freddy, "freddy", "284cx", main_floor_name, 1, 1).await     
              .add_anim(anim::AnimType::Chica, "chica", "1231ss", main_floor_name, 1, 1).await    
              .add_anim(anim::AnimType::Bonnie, "bonnie", "la201", main_floor_name, 1, 1).await   
              .add_anim(anim::AnimType::Foxy,"foxy", "lmmm10", main_floor_name, 1, 90).await     
              .add_anim(anim::AnimType::Puppet, "puppet", "balls12", main_floor_name, 1, 1).await; 
    game_state.start_clock().await;
    game_state.toggle_game_active().await;   
}
