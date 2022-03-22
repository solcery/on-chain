use std::cmp::Ord;
use std::cmp::Ordering;

struct Node<K, V> {
    value: V,
    key: K,
    left: Option<u32>,
    right: Option<u32>,
    size: u32,
    is_red: bool,
    parent: Option<u32>,
}

struct RBTree<'a, K, V>
where
    K: Ord,
{
    root: Option<u32>,
    nodes: &'a mut [Node<K, V>],
}
impl<'a, K, V> RBTree<'a, K, V>
where
    K: Ord,
{
    pub fn init(slice: &'a mut [Node<K, V>]) -> Self {
        Self {
            root: None,
            nodes: slice,
        }
    }

    // TODO: rewrite it with Index trait
    pub fn get(&self, key: &K) -> Option<&V> {
        let mut maybe_id = self.root;
        while let Some(id) = maybe_id {
            let node = &self.nodes[id as usize];
            match key.cmp(&node.key) {
                Ordering::Equal => return Some(&node.value),
                Ordering::Less => maybe_id = node.left,
                Ordering::Greater => maybe_id = node.right,
            }
        }
        None
    }

    // TODO: rewrite it with IndexMut trait
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let mut maybe_id = self.root;
        while let Some(id) = maybe_id {
            let node = &self.nodes[id as usize];
            match key.cmp(&node.key) {
                Ordering::Equal => return Some(&mut self.nodes[id as usize].value),
                Ordering::Less => maybe_id = node.left,
                Ordering::Greater => maybe_id = node.right,
            }
        }
        None
    }
}
