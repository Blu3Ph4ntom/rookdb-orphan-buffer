use crate::quant::SQ8Quantizer;
use crate::io::{AlignedBuffer, DirectReader};
use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;
use std::io::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct Neighbor {
    pub id: u32,
    pub distance: f32,
}

impl Eq for Neighbor {}

impl Ord for Neighbor {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.partial_cmp(&self.distance).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Neighbor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct HNSWIndex {
    pub levels: Vec<Vec<Node>>,
    pub quantizer: SQ8Quantizer,
    pub entry_point: u32,
    pub max_level: usize,
}

#[derive(Debug, Clone)]
pub struct Node {
    pub id: u32,
    pub vector: Vec<u8>,
    pub neighbors: Vec<u32>,
}

pub const BLOCK_SIZE: usize = 4096;

impl HNSWIndex {
    pub fn new(quantizer: SQ8Quantizer) -> Self {
        HNSWIndex {
            levels: Vec::new(),
            quantizer,
            entry_point: 0,
            max_level: 0,
        }
    }

    pub fn route_to_layer0(&self, query: &[f32]) -> Neighbor {
        let q_vec = self.quantizer.quantize(query);
        let mut current_node_id = self.entry_point;
        let mut current_dist = self.quantizer.l2_distance_sq(&q_vec, &self.get_node(self.max_level, current_node_id).vector);

        for level in (1..=self.max_level).rev() {
            let mut changed = true;
            while changed {
                changed = false;
                let node = self.get_node(level, current_node_id);
                for &neighbor_id in &node.neighbors {
                    let neighbor_node = self.get_node(level, neighbor_id);
                    let dist = self.quantizer.l2_distance_sq(&q_vec, &neighbor_node.vector);
                    if dist < current_dist {
                        current_dist = dist;
                        current_node_id = neighbor_id;
                        changed = true;
                    }
                }
            }
        }

        Neighbor { id: current_node_id, distance: current_dist }
    }

    fn get_node(&self, level: usize, id: u32) -> &Node {
        &self.levels[level - 1][id as usize]
    }
}
