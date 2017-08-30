//! A generic Matrix module specilized for holding Go Board state.

use std::collections::{HashMap, hash_set, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Index, IndexMut};
use std::slice;

use game::vertex::{self, Vertex};

/// A matrix holding the state of type T for each vertex on the board.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Matrix<T: Clone + Debug + Default + Hash + Eq + PartialEq> {
    order: usize,
    cells: Vec<T>,
}

fn vertex_from_index(index: usize, board_size: usize) -> Vertex {
    let x = index % board_size;
    let y = index / board_size;
    Vertex { x: x, y: y }
}

fn index_from_vertex(vertex: Vertex, board_size: usize) -> usize {
    vertex.y * board_size + vertex.x
}

impl<T: Clone + Debug + Default + Eq + Hash + PartialEq> Matrix<T> {
    /// Returns all indicies adjacent to index.
    pub fn adjacencies(&self, index: usize) -> Vec<usize> {
        let mut adjacencies = Vec::with_capacity(4);

        if let Some(index) = self.left_of(index) {
            adjacencies.push(index);
        }
        if let Some(index) = self.below(index) {
            adjacencies.push(index);
        }
        if let Some(index) = self.right_of(index) {
            adjacencies.push(index);
        }
        if let Some(index) = self.above(index) {
            adjacencies.push(index);
        }

        adjacencies
    }

    /// Returns the index above _index_ if it exists.
    pub fn above(&self, index: usize) -> Option<usize> {
        if index + self.order < self.order * self.order {
            Some(index + self.order)
        } else {
            None
        }
    }

    /// Returns the index below _index_ if it exists.
    pub fn below(&self, index: usize) -> Option<usize> {
        if index >= self.order {
            Some(index - self.order)
        } else {
            None
        }
    }

    /// Returns the index left of _index_ if it exists.
    pub fn left_of(&self, index: usize) -> Option<usize> {
        if index % self.order > 0 {
            Some(index - 1)
        } else {
            None
        }
    }

    /// Returns the index right of _index_ if it exists.
    pub fn right_of(&self, index: usize) -> Option<usize> {
        if (index + 1) % self.order > 0 {
            Some(index + 1)
        } else {
            None
        }
    }

    /// Converts a vertex into index in the matrix. Returns None if the vertex is not in the matrix.
    pub fn index_from_vertex(&self, vertex: Vertex) -> Option<usize> {
        if vertex.x < self.order && vertex.y < self.order {
            Some(index_from_vertex(vertex, self.order))
        } else {
            None
        }
    }

    /// Returns the vertex of a index.
    pub fn vertex_from_index(&self, index: usize) -> Vertex {
        vertex_from_index(index, self.order)
    }

    /// Returns a set of all of the empty vertices on the board.
    pub fn verts_in_state(&self, in_state: T) -> Vec<Vertex> {
        self.cells
            .iter()
            .enumerate()
            .filter_map(|(index, state)| if *state == in_state {
                Some(vertex_from_index(index, self.order))
            } else {
                None
            })
            .collect()
    }

    /// Returns the cell state at a given vertex or none if the vertex is not in the matrix.
    pub fn get(&self, vertex: Vertex) -> Option<&T> {
        self.cells.get(index_from_vertex(vertex, self.order))
    }

    /// Returns a new empty matrix.
    pub fn with_order(order: usize) -> Self {
        Matrix {
            order: order,
            cells: vec![T::default(); order * order],
        }
    }

    /// Returns the order the matrix.
    pub fn order(&self) -> usize {
        self.order
    }

    /// Returns the matrix to all default values.
    pub fn reset(&mut self) {
        for vertex in &mut self.cells {
            *vertex = T::default();
        }
    }

    /// Returns all of the values stored in the matrix.
    pub fn values(&self) -> slice::Iter<T> {
        self.cells.iter()
    }

    /// Returns an iterator over all of the vertices in the matrix.
    pub fn vertices(&self) -> vertex::Iter {
        vertex::Iter::new(self.order)
    }
}

impl<T: Clone + Debug + Default + Eq + Hash + PartialEq> Index<Vertex> for Matrix<T> {
    type Output = T;
    fn index(&self, vertex: Vertex) -> &Self::Output {
        self.cells
            .get(index_from_vertex(vertex, self.order))
            .expect("vertex not in the matrix")
    }
}

impl<T: Clone + Debug + Default + Eq + Hash + PartialEq> Index<usize> for Matrix<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.cells[index]
    }
}

impl<T: Clone + Debug + Default + Eq + Hash + PartialEq> IndexMut<Vertex> for Matrix<T> {
    fn index_mut(&mut self, vertex: Vertex) -> &mut T {
        self.cells
            .get_mut(index_from_vertex(vertex, self.order))
            .expect("vertex not in the matrix")
    }
}

impl<T: Clone + Debug + Default + Eq + Hash + PartialEq> IndexMut<usize> for Matrix<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.cells[index]
    }
}

impl<T: Clone + Debug + Default + Eq + Hash + PartialEq> From<Vec<T>> for Matrix<T> {
    fn from(cells: Vec<T>) -> Self {
        let order = (cells.len() as f64).sqrt() as usize;
        Matrix { order, cells }
    }
}
