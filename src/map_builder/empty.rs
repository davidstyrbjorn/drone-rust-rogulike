use crate::prelude::*;

use super::MapArchitect;

pub struct EmptyArchitect {}

impl MapArchitect for EmptyArchitect {
    fn new(&mut self, rng: &mut RandomNumberGenerator) -> MapBuilder {
        let mut mb = MapBuilder {
            map: Map::new(),
            monster_spawns: Vec::new(),
            guaranteed_monster_spawns: Vec::new(),
            player_start: Point::zero(),
            teleportation_crystal_start: Point::zero(),
            theme: super::themes::DungeonTheme::new(),
        };

        mb.fill(TileType::Floor);
        mb.player_start = Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);
        mb.teleportation_crystal_start = mb.find_most_distant();
        for _ in 0..50 {
            mb.monster_spawns.push(Point::new(
                rng.range(1, SCREEN_WIDTH),
                rng.range(1, SCREEN_HEIGHT),
            ));
        }
        mb
    }
}
