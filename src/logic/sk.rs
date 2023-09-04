use crate::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct SpatialKnowledge {
    tiles: HashMap<Grid, HashSet<Entity>>,
    entities: HashMap<Entity, HashSet<Grid>>,
    pub tile_size: f32,
}

impl SpatialKnowledge {
    pub fn new(radius: f32) -> Self {
        Self {
            tiles: HashMap::new(),
            entities: HashMap::new(),
            tile_size: radius * 2.0,
        }
    }

    pub fn get_occupied(&self) -> Vec<Grid> {
        self.tiles.keys().cloned().collect()
    }

    pub fn tile(&self, grid: &Grid) -> Option<&HashSet<Entity>> {
        self.tiles.get(grid)
    }

    pub fn report(&mut self, map: &HashMap<Entity, Vec<Vec2>>) {
        map.iter().for_each(|(e, poses)| {
            // find the current tiles this entity occupies
            let mut new_tiles = poses
                .iter()
                .map(|p| (self.grid(*p), false))
                .collect::<HashMap<_, bool>>();

            // remove all previous tiles this entity occupied, if not in the new set
            if let Some(old_tiles) = self.entities.get(e) {
                old_tiles.iter().for_each(|t| {
                    // if the tile is not in the new set,
                    // if this tile only contains this entity,
                    // delete this tile
                    // otherwise remove this entity from the tile

                    if !new_tiles.contains_key(t) {
                        if let Some(entities) = self.tiles.get_mut(t) {
                            if entities.len() == 1 {
                                self.tiles.remove(t);
                            } else {
                                entities.remove(e);
                            }
                        }
                    } else {
                        new_tiles.entry(*t).and_modify(|b| {
                            *b = true;
                        });
                    }
                });
            }

            let mut tiles = HashSet::new();
            for (tile, b) in new_tiles {
                if !b {
                    self.tiles.entry(tile).or_default().insert(*e);
                }

                tiles.insert(tile);
            }

            self.entities.insert(*e, tiles);

            dbg!(&self);
        });
    }

    pub fn full_report(&mut self, entity: Entity, pos: Vec2, radius: f32) {
        let grid = self.grid(pos);
        let points = (0..8).map(|i| {
            let angle = ((i as f32) * std::f32::consts::PI) / 4.0;
            let x = angle.cos() * radius;
            let y = angle.sin() * radius;
            Vec2::new(x, y) + pos
        });

        let mut grids = HashSet::from([grid]);
        points.for_each(|p| {
            grids.insert(self.grid(p));
        });

        self.entities.insert(entity, grids.clone());
        grids.into_iter().for_each(|g| {
            self.tiles.entry(g).or_default().insert(entity);
        });
    }

    pub fn grid(&self, pos: Vec2) -> Grid {
        grid(self.tile_size, pos)
    }

    pub fn pos(&self, grid: Grid) -> Vec2 {
        pos(self.tile_size, grid)
    }
}
