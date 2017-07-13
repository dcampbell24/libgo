use game::chain::Chain;
use game::player::Player;
use game::vertex::Vertex;

// All empty cells have ID 0; white and black chains have some ID > 0.
/// A structure that maps verticies to chains.
#[derive(Clone, Debug)]
pub struct Chains {
    chains: Vec<Chain>,
}

impl Chains {
    /// Adds a stone to the board updating the chains accordingly.
    pub fn add_stone(&mut self, player: Player, vertex: &Vertex) {
        for chain in &mut self.chains {
            if chain.libs.remove(vertex) && chain.player != player {
                chain.filled_libs.insert(*vertex);
            }
        }
    }

    /// Removes all of the Black and White chains from the board.
    pub fn clear(&mut self) {
        self.chains.truncate(0);
    }

    /// Are there any Black or White chains on the board?
    pub fn is_empty(&self) -> bool {
        self.chains.is_empty()
    }

    /// Add a chain to the set of chains.
    pub fn push(&mut self, chain: Chain) {
        self.chains.push(chain)
    }

    /// Returns an empty board of Chains.
    pub fn new() -> Self {
        Chains { chains: Vec::new() }
    }

    /// Removes the chain that contains vertex from the set of chains.
    pub fn remove_chain(&mut self, vertex: &Vertex) -> Option<Chain> {
        let mut idx = None;
        for (i, chain) in self.chains.iter().enumerate() {
            if chain.verts.contains(vertex) {
                idx = Some(i);
                break;
            }
        }
        if let Some(idx) = idx {
            Some(self.chains.swap_remove(idx))
        } else {
            None
        }
    }

    /// Removes all chains with zero liberties of a chosen player and returns their verticies.
    pub fn remove_dead_chains(&mut self, player: Player) -> Vec<Vertex> {
        let mut empty_verts = Vec::new();
        for chain in &self.chains {
            if chain.player == player && chain.libs.is_empty() {
                empty_verts.extend(&chain.verts);
            }
        }
        // Remove the dead chains before updating liberties to avoid updating dead chains.
        self.chains
            .retain(|chain| chain.player != player || !chain.libs.is_empty());
        for vertex in &empty_verts {
            for chain in &mut self.chains {
                if chain.player != player && chain.filled_libs.remove(vertex) {
                    chain.libs.insert(*vertex);
                }
            }
        }
        empty_verts
    }

    /// Returns an iterable struct.
    pub fn iter(&self) -> &Vec<Chain> {
        &self.chains
    }
}
