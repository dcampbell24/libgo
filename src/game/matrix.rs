//! A generic Matrix module specilized for holding Go Board state.

use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

use game::vertex::Vertex;

/// A matrix holding the state of type T for each vertex on the board.
#[derive(Clone, Debug, PartialEq)]
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
    /// Returns the vertex above _vertex_ if it exists.
    pub fn above(&self, vertex: Vertex) -> Option<Vertex> {
        if vertex.y + 1 < self.size {
            Some(Vertex {
                x: vertex.x,
                y: vertex.y + 1,
            })
        } else {
            None
        }
    }

    /// Returns the vertex below _vertex_ if it exists.
    pub fn below(&self, vertex: Vertex) -> Option<Vertex> {
        if vertex.y > 0 {
            Some(Vertex {
                x: vertex.x,
                y: vertex.y - 1,
            })
        } else {
            None
        }
    }

    /// Returns the vertex left of _vertex_ if it exists.
    pub fn left_of(&self, vertex: Vertex) -> Option<Vertex> {
        if vertex.x > 0 {
            Some(Vertex {
                x: vertex.x - 1,
                y: vertex.y,
            })
        } else {
            None
        }
    }

    /// Returns the vertex right of _vertex_ if it exists.
    pub fn right_of(&self, vertex: Vertex) -> Option<Vertex> {
        if vertex.x + 1 < self.size {
            Some(Vertex {
                x: vertex.x + 1,
                y: vertex.y,
            })
        } else {
            None
        }
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

    /// Returns all verticies adjacent to vertex.
    pub fn adjacencies(&self, vertex: Vertex) -> Vec<Vertex> {
        let mut adjacencies = Vec::with_capacity(4);

        if let Some(vertex) = self.left_of(vertex) {
            adjacencies.push(vertex);
        }
        if let Some(vertex) = self.below(vertex) {
            adjacencies.push(vertex);
        }
        if let Some(vertex) = self.right_of(vertex) {
            adjacencies.push(vertex);
        }
        if let Some(vertex) = self.above(vertex) {
            adjacencies.push(vertex);
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

    /// Returns the largest connected region of verticies for which the test function applied to
    /// each vertex returns true starting at `vertex`.
    fn get_region<F: Fn(&T) -> bool>(&self, vertex: Vertex, test: F) -> HashSet<Vertex> {
        let mut passed_test = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue = Vec::new();

        queue.push(vertex);
        visited.insert(vertex);

        while let Some(vertex) = queue.pop() {
            if test(&self[&vertex]) {
                passed_test.insert(vertex);
                for v in self.adjacencies(vertex) {
                    if !visited.contains(&v) {
                        queue.push(v);
                        visited.insert(v);
                    }
                }
            }
        }

        passed_test
    }

    /// Returns all of the largest connected regions of verticies for which the test function
    /// applied to each vertex returns true.
    pub fn get_regions<F: Fn(&T) -> bool>(&self, test: F) -> Vec<HashSet<Vertex>> {
        let mut regions = Vec::new();
        let mut visited = HashSet::new();

        for i in 0..(self.size() * self.size()) {
            let vertex = vertex_from_index(i, self.size);
            if visited.contains(&vertex) {
                continue;
            }
            visited.insert(vertex);

            if test(&self[&vertex]) {
                let region = self.get_region(vertex, &test);
                for v in &region {
                    visited.insert(*v);
                }
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
}

impl<'a, T: Clone + Debug + Default + PartialEq> Index<&'a Vertex> for Matrix<T> {
    type Output = T;
    fn index(&self, vertex: &Vertex) -> &Self::Output {
        self.vec
            .get(index_from_vertex(*vertex, self.size))
            .expect("vertex not in the matrix")
    }
}

impl<'a, T: Clone + Debug + Default + PartialEq> IndexMut<&'a Vertex> for Matrix<T> {
    fn index_mut(&mut self, vertex: &Vertex) -> &mut T {
        self.vec
            .get_mut(index_from_vertex(*vertex, self.size))
            .expect("vertex not in the matrix")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_region() {
        let mut matrix = Matrix::with_size(3);
        let center = Vertex { x: 1, y: 1 };
        let west = Vertex { x: 0, y: 1 };
        let north_east_corner = Vertex { x: 2, y: 2 };
        matrix[&center] = true;
        matrix[&west] = true;
        matrix[&north_east_corner] = true;

        let region = matrix.get_region(center, |&value| value);
        assert_eq!(region, vec![center, west].into_iter().collect());
        let region = matrix.get_region(north_east_corner, |&value| value);
        assert_eq!(region, vec![north_east_corner].into_iter().collect());
        let region = matrix.get_region(Vertex { x: 0, y: 0 }, |&value| value);
        assert_eq!(region, HashSet::new());
    }

    #[test]
    fn get_regions() {
        let mut matrix = Matrix::with_size(3);
        let center = Vertex { x: 1, y: 1 };
        let north_east_corner = Vertex { x: 2, y: 2 };
        matrix[&center] = true;
        matrix[&north_east_corner] = true;

        let region = matrix.get_regions(|&value| value);
        assert_eq!(
            region,
            vec![
                vec![center].into_iter().collect(),
                vec![north_east_corner].into_iter().collect(),
            ]
        );
    }
}
