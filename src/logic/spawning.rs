use rand::Rng;

use crate::prelude::*;

/// Return whether this location is open for spawning
pub fn spawn(radius: f32, pos: Vec2, replacing: Option<Entity>, rc: &Res<RapierContext>) -> bool {
    let mut space_open = true;

    let rot = 0.0;
    let mut filter = QueryFilter::default();
    if let Some(r) = replacing {
        filter.exclude_collider(r);
    }
    rc.intersections_with_shape(pos, rot, &Collider::ball(radius), filter, |ix| {
        // if there is an intersection, and it's not one of the bodies we're replacing
        space_open = false;
        false
    });

    space_open
}

// let wobble =
// (Vec3::new(
//     rand::thread_rng().gen_range(-1.0..1.0),
//     rand::thread_rng().gen_range(-1.0..1.0),
//     0.0
// ) *
//     (i as f32)) /
// (bodies.len() as f32);
// let mut body = body.clone();
// body.render.transform.translation += wobble;

// /// Spawns a single body in, as long that body's spawn location is empty.
// pub fn spawn(
//     body: PhysicsBody,
//     replacing: Vec<Entity>,
//     rc: &Res<RapierContext>,
//     commands: &mut Commands
// ) -> Option<Entity> {
//     let pos = body.render.transform.translation.truncate();
//     let facing = body.render.transform.local_z().truncate();
//     let rot = Vec2::Y.angle_between(facing);
//     let mut filter = QueryFilter::default();
//     for entity in replacing.iter() {
//         filter.exclude_collider(*entity);
//     }
//     if let Some(ix) = rc.intersection_with_shape(pos, rot, &body.collider, filter) {
//         None
//     } else {
//         for entity in replacing {
//             if let Some(ec) = commands.get_entity(entity) {
//                 ec.despawn_recursive();
//             } else {
//                 error!("Was passed an entity to replace, but the entity could not be queried.");
//             }
//         }
//         let e = commands.spawn(body).id();
//         Some(e)
//     }
// }
