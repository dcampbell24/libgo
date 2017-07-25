use std::collections::{HashMap, hash_set, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Index;

use game::matrix::Matrix;
use game::vertex::{self, Vertex};

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

// A reference to a node in the graph.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Node(usize);

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
        let old_value = self.matrix[node];
        if value == old_value {
            return;
        }
        self.matrix[node] = value;

        // Calculate the nodes that may have been part of a split or joined region.
        let maybe_split = Vec::new();
        let maybe_joined = Vec::new();
        for node in self.matrix.adjacencies(node).into_iter() {
            let adjacency = self.matrix[node];
            if adjacency == old_value {
                maybe_split.push(node);
            } else if adjacency == value {
                maybe_joined.push(node);
            }
        }

        // Remove the region containing the old value and create new regions until we cover all the
        // points that may have been part of a split region.
        self.blocks.get_mut(&old_value).unwrap().retain(|region| !region.nodes.contains(&Node(node)));
        while !maybe_split.is_empty() {
            let node = Node(maybe_split.pop().unwrap());
            let region = self.get_region(node, |&v| v == old_value);
            maybe_split.retain(|&n| !region.nodes.contains(&Node(n)));
            self.blocks.get_mut(&old_value).unwrap().push(region);
        }

        // Create a new region at the point of the updated node and remove any possibly joined
        // regions.
        let region = self.get_region(Node(node), |&v| v == value);
        if let Some(blocks) = self.blocks.get_mut(&value) {
            blocks.retain(|region| !maybe_split.iter().any(|&node| {
                region.nodes.contains(&Node(node))
            }));
        }
        self.blocks.entry(value).or_insert(vec![region]).push(region);
    }

    /// Set all of the values in a region to `value` if predicate F is true.
    // TODO pub set_region_to_if

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
            regions.insert(self.matrix[i], region);
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

    /// Returns all regions with value `value`.
    pub fn regions(&self, value: T) -> &Vec<Region> {
        self.blocks.get(&value).unwrap_or(&Vec::new())
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

    static TEST_MATRIX_2: [u32; 4] = [
        0, 0,
        1, 1,
    ];

    static TEST_MATRIX_3: [u32; 9] = [
        0, 0, 1,
        1, 1, 0,
        0, 0, 0,
    ];

    #[test]
    fn get_region() {
        let matrix = Matrix::from(TEST_MATRIX_3.to_vec());

        let region = matrix.get_region(Node(4), |&value| value == 1);
        assert_eq!(region.nodes, vec![Node(3), Node(4)].into_iter().collect());
        assert_eq!(region.adjacencies, vec![Node(0), Node(1), Node(5), Node(6), Node(7)].into_iter().collect());

        let region = matrix.get_region(Node(2), |&value| value == 1);
        assert_eq!(region.nodes, vec![Node(2)].into_iter().collect());
        assert_eq!(region.adjacencies, vec![Node(1), Node(5)].into_iter().collect());


        let region = matrix.get_region(Node(8), |&value| value == 1);
        assert_eq!(region.nodes, HashSet::new());
        assert_eq!(region.adjacencies, HashSet::new());
    }

    #[test]
    fn get_regions() {
        let matrix = Matrix::from(TEST_MATRIX_3.to_vec());

        let regions = matrix.get_regions(|&value| value == 1);
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].nodes, vec![Node(2)].into_iter().collect());
        assert_eq!(regions[1].nodes, vec![Node(3), Node(4)].into_iter().collect());
    }

    #[test]
    fn partition_by_value() {
        let matrix = Matrix::from(TEST_MATRIX_2.to_vec());
        let regions = matrix.get_regions_by_value();
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].nodes, vec![Node(0), Node(1)].into_iter().collect());
        assert_eq!(regions[1].nodes, vec![Node(2), Node(3)].into_iter().collect());
        assert_eq!(regions[0].adjacencies, vec![Node(2), Node(3)].into_iter().collect());
        assert_eq!(regions[1].adjacencies, vec![Node(0), Node(1)].into_iter().collect());
    }
}
