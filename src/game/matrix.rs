//! A generic Matrix module specilized for holding Go Board state.

use game::vertex::Vertex;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

/// A matrix holding the state of type T for each vertex on the board.
#[derive(Clone, Debug)]
pub struct Matrix<T: Clone + Debug + Default + PartialEq> {
    size: usize,
    vec: Vec<T>,
}

fn vertex_from_index(index: usize, board_size: usize) -> Vertex {
    let x = index % board_size;
    let y = index / board_size;
    Vertex { x: x, y: y }
}

fn index_from_vertex(vertex: Vertex, board_size: usize) -> usize {
    vertex.y * board_size + vertex.x
}

impl<T: Clone + Debug + Default + PartialEq> Matrix<T> {
    /// Returns a set of all of the empty verticies on the board.
    pub fn verts_in_state(&self, in_state: T) -> Vec<Vertex> {
        self.vec.iter().enumerate().filter_map(|(index, state)| {
            if *state == in_state {
                Some(vertex_from_index(index, self.size))
            } else {
                None
            }
        }).collect()
    }

    /// Returns the neighboring exterior verticies of a vertex given a board size.
    pub fn exterior(&self, vertex: Vertex) -> Vec<Vertex> {
        let board_size = self.size;
        let mut adjacencies = Vec::with_capacity(4);
        if vertex.x > 0 {
            adjacencies.push(Vertex { x: vertex.x - 1, y: vertex.y });
        }
        if vertex.y > 0 {
            adjacencies.push(Vertex { x: vertex.x, y: vertex.y - 1 });
        }
        if vertex.x + 1 < board_size {
            adjacencies.push(Vertex { x: vertex.x + 1, y: vertex.y });
        }
        if vertex.y + 1 < board_size {
            adjacencies.push(Vertex { x: vertex.x, y: vertex.y + 1 });
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

    /// Returns an immutable reference to the underlying state of the matrix.
    pub fn vec(&self) -> &Vec<T> {
        &self.vec
    }

    /// Returns the largest connected region of verticies not in not_state that contains vertex.
    // This is a flood fill algorithm.
    fn get_not_state_region(&self, vertex: Vertex, not_state: &T) -> Vec<Vertex> {
        let mut processed = vec![false; self.size * self.size];
        let mut queue = Vec::new();
        if *not_state == self[&vertex] {
            return queue;
        }
        queue.push(vertex);

        while let Some(vertex) = queue.pop() {
            let index = index_from_vertex(vertex, self.size);
            if *not_state != self.vec[index] {
                processed[index] = true;
                if vertex.x > 0 && !processed[index - 1] {
                    queue.push(Vertex { x: vertex.x - 1, y: vertex.y });
                }
                if vertex.x + 1 < self.size && !processed[index + 1] {
                    queue.push(Vertex { x: vertex.x + 1, y: vertex.y });
                }
                if vertex.y > 0 && !processed[index - self.size] {
                    queue.push(Vertex { x: vertex.x, y: vertex.y - 1 });
                }
                if vertex.y + 1 < self.size && !processed[index + self.size] {
                    queue.push(Vertex { x: vertex.x, y: vertex.y + 1 });
                }
            }
        }

        for (i, in_region) in processed.into_iter().enumerate() {
            if in_region {
                queue.push(vertex_from_index(i, self.size));
            }
        }
        queue
    }

    /// Returns all largest connected regions of verticies not == state; Black's regions are
    /// connected sub-graphs of White or Empty verticies.
    pub fn split_by_state(&self, state: &T) -> Vec<Vec<Vertex>> {
        let mut regions = Vec::new();
        let mut processed = vec![false; self.size * self.size];
        for i in 0 .. processed.len() {
            if processed[i] || *state == self.vec[i] {
                continue;
            }
            let region = self.get_not_state_region(vertex_from_index(i, self.size), state);
            for v in &region {
                processed[index_from_vertex(*v, self.size)] = true;
            }
            regions.push(region);
        }
        regions
    }

    /// Returns the matrix to all default values.
    pub fn reset(&mut self) {
        for vertex in &mut self.vec {
            *vertex = T::default();
        }
    }

    /// Returns true if the vertex is on the board and empty, otherwise false.
    pub fn is_in_state(&self, vertex: Vertex, in_state: T) -> bool {
        let index = index_from_vertex(vertex, self.size);
        match self.vec.get(index) {
            Some(web) => web == &in_state,
            None => false,
        }
    }
}

impl<'a, T: Clone + Debug + Default + PartialEq> Index<&'a Vertex> for Matrix<T> {
    type Output = T;
    fn index(&self, vertex: &Vertex) -> &Self::Output {
        &self.vec.get(index_from_vertex(*vertex, self.size)).expect("vertex not in the matrix")
    }
}

impl<'a, T: Clone + Debug + Default + PartialEq> IndexMut<&'a Vertex> for Matrix<T> {
    fn index_mut(&mut self, vertex: &Vertex) -> &mut T {
        self.vec.get_mut(index_from_vertex(*vertex, self.size)).expect("vertex not in the matrix")
    }
}
