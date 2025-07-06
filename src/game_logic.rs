
use crate::*;
use crate::gamestate::GameState;

use crate::anim::AnimAsync;
pub trait GameLogicAsync { 
    async fn update(&self);
    async fn move_anims(&self);
}


impl GameLogicAsync for Arc<Mutex<GameState>> {
    async fn update(&self) {
        if !self.game_started().await { 
            return;
        }
        self.move_anims().await;
    }

    async fn move_anims(&self) { 
        let game_state = self.lock().await; 
        for anim_arc in &game_state.anims {
            if anim_arc.can_move().await {
                anim_arc.move_anim().await;
                let mut anim = anim_arc.lock().await;
                anim.can_move = false;
                anim.cooldown = anim.move_delay;
            }
        }
    }
}
