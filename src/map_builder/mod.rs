use crate::prelude::*;
use themes::*;

use self::prefab::PREFAB_LIST;

mod automata;
mod drunkard;
mod empty;
mod prefab;
mod rooms;
mod themes;

trait MapArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder;
}

// A large part of how Rust enforced safe concurrency between threads is through these new types 'Sync' and 'Send'
// Sync lets us safe access this object from multipile threads
// Send lets us safe share this object between threads
pub trait MapTheme: Sync + Send {
    fn tile_to_render(&self, tile_type: TileType) -> FontCharType;
}

pub struct MapBuilder {
    pub map: Map,
    pub player_start: Point,
    pub teleportation_crystal_start: Point,
    pub monster_spawns: Vec<Point>,
    pub guaranteed_monster_spawns: Vec<Point>,
    pub theme: Box<dyn MapTheme>,
}

impl MapBuilder {
    pub fn new(rng: &mut RandomNumberGenerator, map_level: u32) -> Self {
        let theme_name: &str;
        // Build map and select appropiate theme
        let mut architect: Box<dyn MapArchitect> = match map_level {
            0 | 1 => {
                theme_name = "forest";
                Box::new(automata::CellularAutomataArchitect {})
            }
            2 => {
                theme_name = "cave";
                Box::new(drunkard::DrunkardsWalkArchitect {})
            }
            _ => {
                // theme_name = "dungeon";
                theme_name = "dungeon";
                Box::new(rooms::RoomArchitect { rooms: Vec::new() })
                // Box::new(drunkard::DrunkardsWalkArchitect {})
            }
        };

        let mut mb = architect.new(rng);
        // Randomly select a prefab variant
        let prefab = PREFAB_LIST[rng.range(0, PREFAB_LIST.len())];
        prefab::apply_prefab(&mut mb, rng, prefab);

        // Decide on theme
        match theme_name {
            "dungeon" => mb.theme = DungeonTheme::new(),
            "forest" => mb.theme = ForestTheme::new(),
            "cave" => mb.theme = CaveTheme::new(),
            _ => mb.theme = DungeonTheme::new(),
        }

        mb
    }

    fn fill(&mut self, tile: TileType) {
        // Lambda function passed to for each which operatoes on the mutable itertor
        self.map.tiles.iter_mut().for_each(|t| *t = tile);
    }

    fn find_most_distant(&self) -> Point {
        let djikstra_map = DijkstraMap::new(
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
            &vec![self.map.point2d_to_index(self.player_start)],
            &self.map,
            1024.0,
        );
        const UNREACHABLE: &f32 = &f32::MAX;
        // TODO: I don't really get this, try to figure this out later
        self.map.index_to_point2d(
            djikstra_map
                .map
                .iter()
                .enumerate() // Returns (index, distance)
                .filter(|(_, dist)| *dist < UNREACHABLE)
                // we use max_by because we have tuples, but we want to compare e.1, and then grab e.0 (index)
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0,
        )
    }
}

// Returns a vector of spawn points atleast 10 points away from the player
pub fn spawn_monsters(
    mb: &MapBuilder,
    start: &Point,
    rng: &mut RandomNumberGenerator,
) -> Vec<Point> {
    let mut available_spawns: Vec<Point> = mb
        .map
        .tiles
        .iter()
        .enumerate()
        .filter(|(i, t)| {
            **t == TileType::Floor
                && DistanceAlg::Pythagoras.distance2d(*start, mb.map.index_to_point2d(*i)) > 5.0
        })
        .map(|(i, _)| mb.map.index_to_point2d(i))
        .collect();

    const NUM_MONSTERS: usize = 50;
    let mut spawns = Vec::new();
    for _ in 0..NUM_MONSTERS {
        if available_spawns.is_empty() {
            break;
        }
        let target_index = rng.random_slice_index(&available_spawns).unwrap();
        spawns.push(available_spawns[target_index].clone());
        available_spawns.remove(target_index);
    }

    spawns
}
