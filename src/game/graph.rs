use std::collections::{HashMap, hash_set, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Index;
use std::slice;

use game::matrix::Matrix;
use game::vertex::{self, Vertex};

/// A generic structure that contains a square matrix and tracks all of the regions of cells that
/// are adjacent and contain the same value.
#[derive(Clone, Debug)]
pub struct Graph<T: Clone + Debug + Default + Hash + Eq + PartialEq> {
    matrix: Matrix<T>,
    blocks: HashMap<T, Vec<Region>>,
}

impl<T: Clone + Debug + Default + Hash + Eq> PartialEq for Graph<T> {
    fn eq(&self, other: &Self) -> bool {
        self.matrix == other.matrix
    }
}

/// A reference to a node in the graph.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Node(usize);

impl<T: Clone + Debug + Default + Hash + Eq + PartialEq> Graph<T> {
    /// Creates a new graph with all default values backed by a square matrix of order `order`.
    pub fn with_matrix_order(order: usize) -> Self {
        let mut blocks = HashMap::new();
        blocks.insert(T::default(), vec![Region::with_matrix_order(order)]);

        Graph { matrix: Matrix::with_order(order), blocks }
    }

    /// Returns the value at a given vertex or none if the vertex is not in the graph.
    pub fn get(&self, vertex: Vertex) -> Option<&T> {
        self.matrix.get(vertex)
    }

    /// Sets the node in the graph to the new value.
    pub fn set(&mut self, vertex: Vertex, value: T) {
        let node = self.matrix.index_from_vertex(vertex).expect("invalid vertex");
        let old_value = self.matrix[node].clone();
        if value == old_value {
            return;
        }
        self.matrix[node] = value.clone();

        // Calculate the nodes that may have been part of a split or joined region.
        let mut maybe_split = Vec::new();
        let mut maybe_joined = Vec::new();
        for node in self.matrix.adjacencies(node).into_iter() {
            let ref adjacency = self.matrix[node];
            if *adjacency == old_value {
                maybe_split.push(node);
            } else if *adjacency == value {
                maybe_joined.push(node);
            }
        }

        // Remove the region containing the old value and create new regions until we cover all the
        // points that may have been part of a split region.
        self.blocks.get_mut(&old_value).unwrap().retain(|region| !region.nodes.contains(&Node(node)));
        while !maybe_split.is_empty() {
            let node = Node(maybe_split.pop().unwrap());
            let region = self.get_region(node, |v| *v == old_value);
            maybe_split.retain(|&n| !region.nodes.contains(&Node(n)));
            self.blocks.get_mut(&old_value).unwrap().push(region);
        }

        // Create a new region at the point of the updated node and remove any possibly joined
        // regions.
        let region = self.get_region(Node(node), |v| *v == value);
        if let Some(blocks) = self.blocks.get_mut(&value) {
            blocks.retain(|region| !maybe_split.iter().any(|&node| {
                region.nodes.contains(&Node(node))
            }));
        }
        self.blocks.entry(value).or_insert(Vec::new()).push(region);
    }

    /// Returns the underlying matrix order.
    pub fn grid_length(&self) -> usize {
        self.matrix.order()
    }

    /// Set all regions of value _from_ to _to_ where the region does not touch any regions with
    /// value _to_. Returns true if any regions are shifted.
    pub fn shift_regions(&mut self, from: T, to: T) -> bool {
        let mut was_shifted = false;

        if let Some(from_blocks) = self.blocks.remove(&from) {
            let (shifted, not_shifted): (Vec<_>, Vec<_>) = from_blocks.into_iter().partition(|region| {
                region.adjacencies.iter().all(|node| {
                    self.matrix[node.0] != to
                })
            });

            if !shifted.is_empty() {
                was_shifted = true;
            }

            self.blocks.insert(from, not_shifted);
            if let Some(mut blocks) = self.blocks.remove(&to) {
                blocks.extend(shifted);
                self.blocks.insert(to, blocks);
            } else {
                self.blocks.insert(to, shifted);
            }
        }
        was_shifted
    }

    /// Returns the largest connected region of nodes for which the test function applied to
    /// each node returns true starting at `node`.
    fn get_region<F: Fn(&T) -> bool>(&self, node: Node, test: F) -> Region {
        let mut passed_test = HashSet::new();
        let mut adjacencies = HashSet::new();
        let mut queue = Vec::new();
        let mut visited = vec![false; self.matrix.order() * self.matrix.order()];
        let index = node.0;

        queue.push(index);
        visited[index] = true;

        while let Some(index) = queue.pop() {
            if test(&self.matrix[index]) {
                passed_test.insert(Node(index));
                for i in self.matrix.adjacencies(index) {
                    if !visited[i] {
                        queue.push(i);
                        visited[i] = true;
                    }
                }
            } else {
                adjacencies.insert(Node(index));
            }
        }

        if passed_test.is_empty() {
            adjacencies.clear();
        }

        Region { nodes: passed_test, adjacencies }
    }

    /// Returns all of the largest connected regions of vertices for which the test function
    /// applied to each vertex returns true.
    pub fn get_regions<F: Fn(&T) -> bool>(&self, test: F) -> Vec<Region> {
        let matrix_length = self.matrix.order() * self.matrix.order();
        let mut visited = vec![false; matrix_length];
        let mut regions = Vec::new();

        for i in 0..matrix_length {
            if visited[i] {
                continue;
            }

            if test(&self.matrix[i]) {
                let region = self.get_region(Node(i), &test);
                for n in &region.nodes {
                    visited[n.0] = true;
                }
                regions.push(region)
            }
        }
        regions
    }

    // FIXME: Can't there be more than one region with the same value?
    /// Returns all of the largest connected regions of vertices that are equal to each other.
    pub fn get_regions_by_value(&self) -> HashMap<T, Region> {
        let matrix_length = self.matrix.order() * self.matrix.order();
        let mut visited = vec![false; matrix_length];
        let mut regions = HashMap::new();

        for i in 0..matrix_length {
            if visited[i] {
                continue;
            }

            let region = self.get_region(Node(i), |value| value == &self.matrix[i]);
            for n in &region.nodes {
                visited[n.0] = true;
            }
            regions.insert(self.matrix[i].clone(), region);
        }
        regions
    }

    /// Updates the graph to its default initial state.
    pub fn reset(&mut self) {
        self.matrix.reset();
        self.blocks.clear();
        self.blocks.insert(T::default(), vec![Region::with_matrix_order(self.matrix.order())]);
    }

    /// Returns an iterator over all of the vertices in the graph.
    pub fn vertices(&self) -> vertex::Iter {
        self.matrix.vertices()
    }

    /// Returns an iterator over the values in the graph.
    pub fn values(&self) -> slice::Iter<T> {
        self.matrix.values()
    }

    /// Returns all regions with value `value`.
    pub fn regions(&self, value: T) -> Option<&Vec<Region>> {
        self.blocks.get(&value)
    }
}

impl<T: Clone + Debug + Default + Eq + Hash + PartialEq> Index<Vertex> for Graph<T> {
    type Output = T;
    fn index(&self, vertex: Vertex) -> &Self::Output {
        &self.matrix[vertex]
    }
}

/// A set of connected nodes in the matrix and their adjacencies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Region {
    nodes: HashSet<Node>,
    adjacencies: HashSet<Node>,
}

impl Region {
    /// Returns an iterator over all of the nodes in the region.
    pub fn nodes(&self) -> hash_set::Iter<Node> {
        self.nodes.iter()
    }

    /// Returns a square region anchored at the origin with no adjacencies.
    fn with_matrix_order(order: usize) -> Self {
        let nodes = (0..(order * order)).map(|index| Node(index)).collect();
        Region { nodes, adjacencies: HashSet::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    static TEST_MATRIX_2: [u32; 4] = [
        0, 0,
        1, 1,
    */

    /*
    static TEST_MATRIX_3: [u32; 9] = [
        0, 0, 1,
        1, 1, 0,
        0, 0, 0,
    ];
    */

    #[test]
    fn get_region() {
        let mut graph = Graph::with_matrix_order(3);
        graph.set(Vertex {x: 2, y: 2}, 1);
        graph.set(Vertex {x: 1, y: 1}, 1);
        graph.set(Vertex {x: 0, y: 1}, 1);

        let region = graph.get_region(Node(4), |&value| value == 1);
        assert_eq!(region.nodes, vec![Node(3), Node(4)].into_iter().collect());
        assert_eq!(region.adjacencies, vec![Node(0), Node(1), Node(5), Node(6), Node(7)].into_iter().collect());

        let region = graph.get_region(Node(8), |&value| value == 1);
        assert_eq!(region.nodes, vec![Node(8)].into_iter().collect());
        assert_eq!(region.adjacencies, vec![Node(5), Node(7)].into_iter().collect());

        let region = graph.get_region(Node(2), |&value| value == 1);
        assert_eq!(region.nodes, HashSet::new());
        assert_eq!(region.adjacencies, HashSet::new());
    }

    #[test]
    fn get_regions() {
        let mut graph = Graph::with_matrix_order(3);
        graph.set(Vertex {x: 2, y: 2}, 1);
        graph.set(Vertex {x: 1, y: 1}, 1);
        graph.set(Vertex {x: 0, y: 1}, 1);

        let regions = graph.get_regions(|&value| value == 1);
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].nodes, vec![Node(3), Node(4)].into_iter().collect());
        assert_eq!(regions[1].nodes, vec![Node(8)].into_iter().collect());
    }

    #[test]
    fn partition_by_value() {
        let mut graph = Graph::with_matrix_order(2);
        graph.set(Vertex {x: 0, y: 0}, 1);
        graph.set(Vertex {x: 1, y: 0}, 1);

        let regions = graph.get_regions_by_value();
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[&0].nodes, vec![Node(2), Node(3)].into_iter().collect());
        assert_eq!(regions[&0].adjacencies, vec![Node(0), Node(1)].into_iter().collect());
        assert_eq!(regions[&1].nodes, vec![Node(0), Node(1)].into_iter().collect());
        assert_eq!(regions[&1].adjacencies, vec![Node(2), Node(3)].into_iter().collect());
    }
}
