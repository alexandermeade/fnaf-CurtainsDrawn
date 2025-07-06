use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::{HashSet, VecDeque};

use crate::anim;

use crate::anim::AnimAsync;

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    LeftUp,
    RightUp,
    LeftDown,
    RightDown,
    None,
}

#[derive(Debug)]
pub struct DestNode {
    pub dist_to_office: u8,
    pub name: String,
    pub tag: String,
    pub connections: Vec<Arc<Mutex<DestNode>>>,
    pub anims: Vec<Arc<Mutex<anim::Anim>>>,
    pub target: char,
    pub warning: bool,
    is_office: bool,
}

impl DestNode {
    pub fn new(name: &str, tag: &str, dist_to_office: u8, target: char, is_office: bool) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(DestNode {
            dist_to_office,
            name: name.to_string(),
            tag: tag.to_string(),
            connections: vec![],
            anims: vec![],
            target,
            warning: false,
            is_office,
        }))
    }
}

//#[async_trait::async_trait]  // You'll need async_trait crate for async in traits
pub trait DestNodeFns {
    async fn add_connection(&self, other: &Arc<Mutex<DestNode>>) -> Arc<Mutex<DestNode>>;
    async fn set_dist(&self);
    async fn map_replacement(&self) -> (String, String);
    async fn add_anim(&mut self, target: Arc<Mutex<anim::Anim>>) -> Arc<Mutex<DestNode>>; 
    async fn remove_anim(&mut self, target: anim::AnimType);
    async fn name(&self) -> String;

    async fn intercom(&self);
}

//#[async_trait::async_trait]
impl DestNodeFns for Arc<Mutex<DestNode>> {
    
    async fn name(&self) -> String {
        self.lock().await.name.clone()
    }
    async fn remove_anim(&mut self, target: anim::AnimType) { 
        let mut node = self.lock().await;
        let mut indicies:Vec<usize> = vec![];
        for (i, anim) in node.anims.iter().enumerate() {
            if anim.lock().await.anim_type == target {
                indicies.push(i);
            }
        }
        for i in indicies.iter().rev() {
            node.anims.remove(*i);
        }
    }

    async fn add_anim(&mut self, target: Arc<Mutex<anim::Anim>>) -> Arc<Mutex<DestNode>> {
        
        //target.set_location(Arc::clone(self)).await;       
        let mut anim = target.lock().await;
        anim.location = Some(self.clone());
        drop(anim);
        let mut node = self.lock().await;
        node.anims.push(Arc::clone(&target));
        drop(node);

        return Arc::clone(self);
    }

    async fn add_connection(&self, other: &Arc<Mutex<DestNode>>) -> Arc<Mutex<DestNode>> {
        let mut node = self.lock().await;
        node.connections.push(Arc::clone(other));
        Arc::clone(self)
    }

    async fn intercom(&self) {
        
    }

    async fn set_dist(&self) {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();

        {
            let node = self.lock().await;
            queue.push_back((Arc::clone(self), 0));
            visited.insert(node.tag.clone());
        }

        while let Some((node_arc, dist)) = queue.pop_front() {
            let mut node = node_arc.lock().await;
            node.dist_to_office = dist;

            for conn_arc in &node.connections {
                let conn = conn_arc.lock().await;
                if !visited.contains(&conn.tag) {
                    visited.insert(conn.tag.clone());
                    drop(conn);
                    queue.push_back((Arc::clone(conn_arc), dist + 1));
                }
            }
        }
    }

    async fn map_replacement(&self) -> (String, String) {
        let node = self.lock().await;
        let mut result =  format!("{}{}", if node.warning {"⚠"} else {" "}, "     ");
        for anim_ in &node.anims {
            let anim = anim_.lock().await;
            let index:usize = anim.anim_type.clone() as usize;
            let mut chars: Vec<char> = result.chars().collect();
            chars[index] = '·';
            result = chars.into_iter().collect();
            drop(anim);
        }
        let target = std::iter::repeat(node.target).take(6).collect::<String>();
        return (target, result);
    }

}

