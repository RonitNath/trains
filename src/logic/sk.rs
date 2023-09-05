use std::collections::{ VecDeque, BTreeMap, BinaryHeap };

use crate::{ prelude::*, world::EntityKind };

#[derive(Component, Clone, Debug)]
pub struct SpatialKnowledge {
    // used to stored what is on which tiles
    tiles: HashMap<Grid, HashSet<Entity>>,
    // used to point to where an entity is
    entities: HashMap<Entity, HashSet<Grid>>,
    // used to indicate that an entity which moves is being stored, so it can be removed when not being looked at
    pub temp: HashSet<Entity>,
    // used to remember information about entities
    details: HashMap<Entity, EntityKind>,
    // used to not duplicate information requests
    pub querying: HashSet<Entity>,
    pub tile_size: f32,
}

impl SpatialKnowledge {
    pub fn new(radius: f32) -> Self {
        Self {
            tiles: HashMap::new(),
            entities: HashMap::new(),
            temp: HashSet::new(),
            details: HashMap::new(),
            querying: HashSet::new(),
            tile_size: radius * 2.0,
        }
    }

    pub fn details(&self) -> Vec<Entity> {
        self.details
            .keys()
            .into_iter()
            .map(|k| *k)
            .collect::<Vec<Entity>>()
    }

    pub fn query_in_progress(&mut self, entity: Entity) {
        self.querying.insert(entity);
    }

    pub fn get_occupied(&self) -> Vec<Grid> {
        self.tiles.keys().cloned().collect()
    }

    pub fn tile(&self, grid: &Grid) -> Option<&HashSet<Entity>> {
        self.tiles.get(grid)
    }

    pub fn queried(&self, entity: &Entity) -> bool {
        // either we have information about it or we don't and we're not querying it right now
        self.details.contains_key(entity) || self.querying.contains(entity)
    }

    pub fn obtains_info_about(&mut self, entity: Entity, kind: EntityKind, pos: Vec2) {
        self.querying.remove(&entity);
        self.details.insert(entity, kind);

        dbg!(entity);

        self.full_report(entity, pos, RADIUS);

        dbg!(&self.temp);
        if kind == EntityKind::Base && self.temp.contains(&entity) {
            self.temp.remove(&entity);
        }
    }

    pub fn has_seen(&self) -> bool {
        !self.entities.is_empty()
    }

    /// Computes A Star
    /// Returns a path and whether the path encoutered an obstruction
    /// Path is constructed by pushing the target tile until the start tile, but then is reversed
    /// so is actually start tile to target tile
    pub fn path(&self, my_pos: Vec2, target: Vec2) -> (Vec<Vec2>, bool) {
        let start_tile: Grid = self.grid(my_pos);
        let mut current_tile = start_tile;
        let target_tile = self.grid(target);
        let max_check_dist = start_tile.distance(target_tile) * 2;

        let mut points_to: bevy::utils::hashbrown::HashMap<Grid, (Grid, AStar)> = HashMap::new();
        let mut open_map = BTreeMap::new();
        let mut visited = HashSet::new();

        let mut obstruction = false;

        // until we reach the target tile
        while target_tile != current_tile {
            // look at all the neighbors
            for neighbor in current_tile.neighbors() {
                // if the neighbor is too far away, skip it
                if neighbor.distance(target_tile) > max_check_dist {
                    // eventually this will lead to open_map ending
                    continue;
                }

                // if the tile has no entities
                if visited.contains(&neighbor) {
                    continue;
                }

                if let Some(entities) = self.tiles.get(&neighbor) {
                    if !entities.is_empty() {
                        // there is some entity on this tile
                        obstruction = true;
                        continue;
                    }
                }

                // then set the neighbor to point at yourself. If the neighbor is already pointing,
                // have it point at whichever path is faster
                let score = AStar::calc(0, current_tile, target_tile, neighbor);
                points_to
                    .entry(neighbor)
                    .and_modify(|(stored_tile, astar)| {
                        if score.origin_score < astar.origin_score {
                            *stored_tile = current_tile;
                            *astar = score;
                        }
                    })
                    .or_insert((current_tile, score));

                // if the neighbor is not in the open map, add it
                if !open_map.contains_key(&score) {
                    open_map.insert(score, neighbor);
                }
            }

            // set current_tile to the next tile with the lowest score. Remove that tile from the open map
            if let Some((score, tile)) = open_map.clone().iter().next() {
                current_tile = *tile;
                open_map.remove(score);
                visited.insert(current_tile);
            } else {
                // there are no tiles left in the open_map. We are in an enclosed space with no path to the target,
                // or we've reached max search distance
                warn!("No path to target");
                return (vec![], false);
            }
        }

        // we have reached the target tile. Now we need to backtrack to the start tile
        let mut path = vec![self.pos(current_tile)];

        while current_tile != start_tile {
            if let Some((prev_tile, _)) = points_to.get(&current_tile) {
                current_tile = *prev_tile;
                path.push(self.pos(current_tile));
            } else {
                // The current tile doesn't point anywhere. This should only happen to the start tile,
                // which should have existed us out already. If it happens to any other tile, we're in trouble
                warn!("Tile doesn't point back");
                break;
            }
        }

        (path, obstruction)
    }

    pub fn clear_seen(&mut self) {
        // clear entities and remove references to tiles they're on
        let entities = self.temp.iter().cloned().collect::<Vec<_>>();
        entities.iter().for_each(|e| {
            self.delete_entity(e);
        });
    }

    pub fn delete_entity(&mut self, entity: &Entity) {
        if let Some(tiles) = self.entities.remove(entity) {
            tiles.iter().for_each(|t| {
                if let Some(entities) = self.tiles.get_mut(t) {
                    // if this tile only contains this entity, delete this tile, otherwise remove this entity from the tile
                    if entities.len() == 1 {
                        self.tiles.remove(t);
                    } else {
                        entities.remove(entity);
                    }
                }
            });
        }
    }

    pub fn report(&mut self, map: &HashMap<Entity, Vec<Vec2>>) {
        map.iter().for_each(|(e, poses)| {
            // find the current tiles this entity occupies
            let mut new_tiles = poses
                .iter()
                .map(|p| (self.grid(*p), false))
                .collect::<HashMap<_, bool>>();

            // if this is something which is not temp, don't overwrite previous tiles
            if self.is_static(e) {
                for (new_tile, _b) in new_tiles.iter() {
                    self.tiles.entry(*new_tile).or_default().insert(*e);
                    self.entities.entry(*e).or_default().insert(*new_tile);
                }
                return;
            }

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
            if !self.known(e) {
                self.temp.insert(*e);
            }
        });
    }

    pub fn is_static(&self, entity: &Entity) -> bool {
        if let Some(kind) = self.details.get(entity) {
            if *kind == EntityKind::Base {
                return true;
            }
        }
        false
    }

    /// returns whether an entity has a details entry which is not unknown
    pub fn known(&self, entity: &Entity) -> bool {
        self.details.get(entity).map_or(false, |k| *k != EntityKind::Unknown)
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

/// Distance to target is calculated on an absolute basis
/// Distance to origin is calculated on a relative basis of the path followed
///
/// So, if the origin is 0,0
///     and the target is 2, 1
///     0, 1 has cost of 10 + 14
///     1, 1 has cost of 14 + 10
///     As origin_cost, target_cost
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd)]
struct AStar {
    origin_score: i32,
    target_score: i32,
    score: i32,
}

impl Ord for AStar {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // score, otherwise g_score
        if self.score == other.score {
            if self.origin_score == other.origin_score {
                self.target_score.cmp(&other.target_score)
            } else {
                self.origin_score.cmp(&other.origin_score)
            }
        } else {
            self.score.cmp(&other.score)
        }
    }
}

impl AStar {
    pub fn new(origin_score: i32, target_score: i32) -> Self {
        Self { origin_score, target_score, score: origin_score + target_score }
    }

    pub fn calc(accumulating_cost: i32, prev_grid: Grid, target: Grid, scoring: Grid) -> Self {
        let origin_score = scoring.distance(prev_grid) + accumulating_cost;
        let target_score = scoring.distance(target);

        Self::new(origin_score, target_score)
    }
}

#[test]
fn test_path() {
    let mut sk = SpatialKnowledge::new(0.5);
    // insert entity::PLACEHOLDER at (3,0)
    sk.full_report(Entity::PLACEHOLDER, Vec2::new(3.0, 0.0), 0.5);
    let path = sk.path(Vec2::ZERO, Vec2::new(10.0, 0.0));
    dbg!(path);
}
