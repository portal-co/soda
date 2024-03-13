use std::{collections::BTreeMap, sync::{Arc, Mutex}};

use stdweb::web::{MutationObserver, MutationObserverHandle, MutationObserverInit, MutationRecord, Node};
use once_cell::sync::Lazy;
use bimap::*;
#[derive(Eq,PartialEq,Clone)]
struct NodeWrapper(Node);
impl PartialOrd for NodeWrapper{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.as_ref().as_raw().partial_cmp(&other.0.as_ref().as_raw())
    }
}
impl Ord for NodeWrapper{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.partial_cmp(other).unwrap();
    }
}
#[derive(Clone)]
struct State{
    observer: Arc<MutationObserverHandle>
}
static M: Lazy<Mutex<BTreeMap<u32,State>>> = Lazy::new(||Mutex::new(BTreeMap::new()));
static IDS: Lazy<Mutex<BiBTreeMap<NodeWrapper,u32>>> = Lazy::new(||Mutex::new(BiBTreeMap::new()));
static HOOKS: Lazy<Mutex<Vec<Box<dyn Hook>>>> = Lazy::new(||Mutex::new(vec![]));
pub fn add_hook(h: Box<dyn Hook>){
    let mut l = HOOKS.lock().unwrap();
    l.push(h);
}
pub trait Hook: Send{
    fn poke(&mut self, n: &Node, r: &MutationRecord);
}
fn id_of(n: NodeWrapper) -> u32{
    let mut l = IDS.lock().unwrap();
    if let Some(b) = l.get_by_left(&n){
        return *b;
    }
    let mut i = 0;
    while l.contains_right(&i){
        i += 1
    }
    l.insert(n, i);
    return i;
}
pub fn id(n: Node) -> u32{
    return id_of(NodeWrapper(n));
}
fn cleanup(n: &Node){
    let mut ml = M.lock().unwrap();
    ml.remove(&id_of(NodeWrapper(n.clone())));
    let mut l = IDS.lock().unwrap();
    l.remove_by_left(&NodeWrapper(n.clone()));
}
fn observer(n: &Node) -> State{
    let mut l = M.lock().unwrap();
    match l.get(&id_of(NodeWrapper(n.clone()))){
        Some(a) => return a.clone(),
        None => {},
    }
    let o = n.clone();
    let mut mn = MutationObserver::new(move|v,_|{
        for w in v{
            let mut hl = HOOKS.lock().unwrap();
            for m in hl.iter_mut(){
                m.poke(&o, &w);
            }
            match w{
                stdweb::web::MutationRecord::Attribute { target, name, namespace, old_value } => {},
                stdweb::web::MutationRecord::CharacterData { target, old_data } => {},
                stdweb::web::MutationRecord::ChildList { target, inserted_nodes, removed_nodes, previous_sibling, next_sibling } => {
                    for r in removed_nodes{
                        cleanup(&r);
                    }
                    for a in inserted_nodes{
                        observer(&a);
                    }
                },
            }
        }
    });
    mn.observe(n,MutationObserverInit{child_list: true, character_data: true, attributes: true, subtree: false, attribute_old_value: false, character_data_old_value: false, attribute_filter: None });
    let mn = Arc::new(mn);
    let s = State{
        observer: mn,
    };
    l.insert(id_of(NodeWrapper(n.clone())), s.clone());
    return s;
}
pub fn start(n: Node){
    observer(&n);
}
#[cfg(test)]
mod tests {
    use super::*;

}
