use game::board::Neighbors;
use game::player::Player;
use game::vertex::Vertex;
use std::collections::HashSet;


/// A connected sub-graph with nodes of all the same type.
#[derive(Clone, Debug)]
pub struct Chain {
    /// The state all of the verticies of the chain are in.
    pub player: Player,
    /// The set of verticies in the chain.
    pub verts: HashSet<Vertex>,
    /// The set of neighboring verticies that are empty.
    pub libs: HashSet<Vertex>,
    /// The set of neighboring verticies that are filled (by the opponent).
    pub filled_libs: HashSet<Vertex>,
}

impl Chain {
    /// Create a new chain initialized with a vertex and its neighbors.
    pub fn new(player: Player, vertex: Vertex, neighbors: &Neighbors) -> Self {
        let mut verts = HashSet::new();
        let mut libs = HashSet::new();
        let mut filled_libs = HashSet::new();

        verts.insert(vertex);
        libs.extend(&neighbors.empty);
        filled_libs.extend(&neighbors.evil);

        Chain {
            player: player,
            verts: verts,
            libs: libs,
            filled_libs: filled_libs,
        }
    }

    /// Update a chain with the consumed union of another.
    pub fn eat(&mut self, chain: Chain) {
        self.verts.extend(chain.verts);
        self.libs.extend(chain.libs);
        self.filled_libs.extend(chain.filled_libs);
    }
}
