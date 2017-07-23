//! A generic Matrix module specilized for holding Go Board state.

use std::collections::{hash_set, HashSet};
use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use std::slice;

use game::vertex::Vertex;

/// A matrix holding the state of type T for each vertex on the board.
#[derive(Clone, Debug, PartialEq)]
pub struct Matrix<T: Clone + Debug + Default + PartialEq> {
    size: usize,
    vec: Vec<T>,
}

/// A reference to a location in a Matrix.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Node(usize);

fn vertex_from_index(index: usize, board_size: usize) -> Vertex {
    let x = index % board_size;
    let y = index / board_size;
    Vertex { x: x, y: y }
}

fn index_from_vertex(vertex: Vertex, board_size: usize) -> usize {
    vertex.y * board_size + vertex.x
}

impl<T: Clone + Debug + Default + PartialEq> Matrix<T> {
    /// Returns the node above _node_ if it exists.
    pub fn above(&self, node: Node) -> Option<Node> {
        if node.0 + self.size < self.size * self.size {
            Some(Node(node.0 + self.size))
        } else {
            None
        }
    }

    /// Returns the node below _node_ if it exists.
    pub fn below(&self, node: Node) -> Option<Node> {
        if node.0 >= self.size {
            Some(Node(node.0 - self.size))
        } else {
            None
        }
    }

    /// Returns the node left of _node_ if it exists.
    pub fn left_of(&self, node: Node) -> Option<Node> {
        if node.0 % self.size > 0 {
            Some(Node(node.0 - 1))
        } else {
            None
        }
    }

    /// Returns the node right of _node_ if it exists.
    pub fn right_of(&self, node: Node) -> Option<Node> {
        if (node.0 + 1) % self.size > 0 {
            Some(Node(node.0 + 1))
        } else {
            None
        }
    }

    /// Converts a vertex into node in the matrix. Returns None if the vertex is not in the matrix.
    pub fn node_from_vertex(&self, vertex: Vertex) -> Option<Node> {
        if vertex.x < self.size && vertex.y < self.size {
            Some(Node(index_from_vertex(vertex, self.size)))
        } else {
            None
        }
    }

    /// Returns the vertex of a node.
    pub fn vertex_from_node(&self, node: Node) -> Vertex {
        vertex_from_index(node.0, self.size)
    }

    /// Returns a set of all of the empty verticies on the board.
    pub fn verts_in_state(&self, in_state: T) -> Vec<Vertex> {
        self.vec
            .iter()
            .enumerate()
            .filter_map(|(index, state)| if *state == in_state {
                Some(vertex_from_index(index, self.size))
            } else {
                None
            })
            .collect()
    }

    /// Returns all nodes adjacent to node.
    pub fn adjacencies(&self, node: Node) -> Vec<Node> {
        let mut adjacencies = Vec::with_capacity(4);

        if let Some(node) = self.left_of(node) {
            adjacencies.push(node);
        }
        if let Some(node) = self.below(node) {
            adjacencies.push(node);
        }
        if let Some(node) = self.right_of(node) {
            adjacencies.push(node);
        }
        if let Some(node) = self.above(node) {
            adjacencies.push(node);
        }

        adjacencies
    }

    /// Returns the cell state at a given vertex or none if the vertex is not in the matrix.
    pub fn get(&self, vertex: Vertex) -> Option<&T> {
        self.vec.get(index_from_vertex(vertex, self.size))
    }

    /// Returns a new empty matrix.
    pub fn with_size(size: usize) -> Self {
        Matrix {
            size: size,
            vec: vec![T::default(); size * size],
        }
    }

    /// Returns the matrix size.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the largest connected region of nodes for which the test function applied to
    /// each node returns true starting at `node`.
    fn get_region<F: Fn(&T) -> bool>(
        &self,
        node: Node,
        test: F,
        visited: &mut Vec<bool>,
    ) -> Region {
        assert!(visited.len() == self.size * self.size);

        let mut passed_test = HashSet::new();
        let mut adjacencies = HashSet::new();
        let mut queue = Vec::new();

        queue.push(node);
        visited[node.0] = true;

        while let Some(node) = queue.pop() {
            if test(&self[node]) {
                passed_test.insert(node);
                for n in self.adjacencies(node) {
                    if !visited[n.0] {
                        queue.push(n);
                        visited[n.0] = true;
                    }
                }
            } else {
                adjacencies.insert(node);
            }
        }

        if passed_test.is_empty() {
            adjacencies.clear();
        }

        Region { nodes: passed_test, adjacencies }
    }

    /// Returns all of the largest connected regions of verticies for which the test function
    /// applied to each vertex returns true.
    pub fn get_regions<F: Fn(&T) -> bool>(&self, test: F) -> Vec<Region> {
        let mut visited = vec![false; self.size * self.size];
        let mut regions = Vec::new();

        for i in 0..(self.size() * self.size()) {
            if visited[i] {
                continue;
            }

            let node = Node(i);
            if test(&self[node]) {
                let region = self.get_region(node, &test, &mut visited);
                regions.push(region)
            }
        }
        regions
    }

    /// Returns the matrix to all default values.
    pub fn reset(&mut self) {
        for vertex in &mut self.vec {
            *vertex = T::default();
        }
    }

    /// Returns all of the values stored in the Matrix.
    pub fn values(&self) -> slice::Iter<T> {
        self.vec.iter()
    }
}

impl<'a, T: Clone + Debug + Default + PartialEq> Index<&'a Vertex> for Matrix<T> {
    type Output = T;
    fn index(&self, vertex: &Vertex) -> &Self::Output {
        self.vec
            .get(index_from_vertex(*vertex, self.size))
            .expect("vertex not in the matrix")
    }
}

impl<T: Clone + Debug + Default + PartialEq> Index<Node> for Matrix<T> {
    type Output = T;
    fn index(&self, node: Node) -> &Self::Output {
        &self.vec[node.0]
    }
}

impl<'a, T: Clone + Debug + Default + PartialEq> IndexMut<&'a Vertex> for Matrix<T> {
    fn index_mut(&mut self, vertex: &Vertex) -> &mut T {
        self.vec
            .get_mut(index_from_vertex(*vertex, self.size))
            .expect("vertex not in the matrix")
    }
}

impl<T: Clone + Debug + Default + PartialEq> IndexMut<Node> for Matrix<T> {
    fn index_mut(&mut self, node: Node) -> &mut Self::Output {
        &mut self.vec[node.0]
    }
}


impl<T: Clone + Debug + Default + PartialEq> From<Vec<T>> for Matrix<T> {
    fn from(vec: Vec<T>) -> Self {
        let size = (vec.len() as f64).sqrt() as usize;
        Matrix { size, vec }
    }
}


/// A set of connected nodes in the matrix and their adjacencies.
#[derive(Debug, PartialEq)]
pub struct Region {
    nodes: HashSet<Node>,
    adjacencies: HashSet<Node>,
}

impl Region {
    pub fn nodes(&self) -> hash_set::Iter<Node> {
        self.nodes.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_MATRIX: [u32; 9] = [
        0, 0, 1,
        1, 1, 0,
        0, 0, 0,
    ];

    #[test]
    fn get_region() {
        let matrix = Matrix::from(TEST_MATRIX.to_vec());

        let mut visited = vec![false; 9];
        let region = matrix.get_region(Node(4), |&value| value == 1, &mut visited);
        assert_eq!(region.nodes, vec![Node(3), Node(4)].into_iter().collect());
        assert_eq!(region.adjacencies, vec![Node(0), Node(1), Node(5), Node(6), Node(7)].into_iter().collect());

        let mut visited = vec![false; 9];
        let region = matrix.get_region(Node(2), |&value| value == 1, &mut visited);
        assert_eq!(region.nodes, vec![Node(2)].into_iter().collect());
        assert_eq!(region.adjacencies, vec![Node(1), Node(5)].into_iter().collect());


        let mut visited = vec![false; 9];
        let region = matrix.get_region(Node(8), |&value| value == 1, &mut visited);
        assert_eq!(region.nodes, HashSet::new());
        assert_eq!(region.adjacencies, HashSet::new());
    }

    #[test]
    fn get_regions() {
        let matrix = Matrix::from(TEST_MATRIX.to_vec());

        let regions = matrix.get_regions(|&value| value == 1);
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].nodes, vec![Node(2)].into_iter().collect());
        assert_eq!(regions[1].nodes, vec![Node(3), Node(4)].into_iter().collect());
    }
}
