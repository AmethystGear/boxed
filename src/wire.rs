use bevy::{
    color::palettes::css::{BLUE, GRAY, RED},
    prelude::*,
    ui::widget,
    utils::HashSet,
};
use itertools::Itertools;

use crate::server::{Generator, Server};

const WIRE_Z: f32 = 0.0;

#[derive(Component)]
pub struct Wire {
    pub terminals: [Entity; 2],
    pub control_points: [Vec2; 2],
    pub segments: usize,
    pub cuts: HashSet<usize>,
    pub width: f32,
}

#[derive(Event)]
pub struct UpdateWire {
    operation: WireOperation,
    wire: Entity,
    pos: usize,
}

pub enum WireOperation {
    Cut,
    Join,
}

// get the perpendicular of the provided vector.
fn perp(v: Vec2) -> Vec2 {
    Vec2::new(-v.y, v.x)
}

// update wire meshes when we get a cut/join event.
fn update_wires(
    mut ev_wireupdate: EventReader<UpdateWire>,
    mut wires: Query<(Entity, &mut Wire)>,
    servers: Query<(Entity, &Transform), With<Server>>,
    generators: Query<(Entity, &Transform), With<Generator>>,
) {
    for ev in ev_wireupdate.read() {
        if let Some((_, mut wire)) = wires.iter_mut().filter(|(e, _)| *e == ev.wire).next() {
            match ev.operation {
                WireOperation::Cut => {
                    wire.cuts.replace(ev.pos);
                }
                WireOperation::Join => {
                    wire.cuts.remove(&ev.pos);
                }
            }

            fn find<'a, T: Component>(
                entity: &Entity,
                query: &'a Query<(Entity, &Transform), With<T>>,
            ) -> Option<&'a Transform> {
                query
                    .iter()
                    .filter(|(e, _)| e == entity)
                    .map(|(_, transform)| transform)
                    .next()
            }

            // wire is a powerline if either of the terminals is a generator
            let is_pwr = wire
                .terminals
                .iter()
                .any(|terminal| find(&terminal, &generators).is_some());

            // get the locations of the terminals
            let terminals = wire.terminals.map(|terminal| {
                find(&terminal, &servers)
                    .or(find(&terminal, &generators))
                    .map(|transform| transform.translation.truncate())
                    .expect("wire should be connecting server -> server or server -> generator")
            });

            // calculate bezier curve
            let diff = terminals[1] - terminals[0];
            let perpendicular = Vec2::new(0., 1.).rotate(diff);
            let control_points = wire
                .control_points
                .map(|pt| terminals[0] + diff * pt.x + perpendicular * pt.y);
            let curve = CubicBezier::new(vec![[
                terminals[0],
                control_points[0],
                control_points[1],
                terminals[1],
            ]])
            .to_curve();

            // wire is grey if there is at least 1 cut, red if PWR, blue if COM
            let color = if wire.cuts.len() > 0 {
                Vec4::new(0.35, 0.35, 0.35, 1.0)
            } else if is_pwr {
                Vec4::new(1.0, 0.0, 0.0, 1.0)
            } else {
                Vec4::new(0.0, 0.2, 0.9, 1.0)
            };

            // construct the lines by sampling the bezier curve, and breaking + skipping over cuts.
            let mut lines = vec![];
            let mut line = vec![];
            for (i, pos) in curve.iter_positions(wire.segments).enumerate() {
                if wire.cuts.contains(&(i + 1)) {
                    lines.push(line);
                    line = vec![];
                } else if wire.cuts.contains(&(i)) {
                    continue;
                } else {
                    line.push(pos);
                }
            }
            lines.push(line);

            // construct triangles - wgpu cannot draw lines portably. mucho sado :(
            let mut verts = vec![];
            let mut breaks = HashSet::new();
            for line in lines {
                let mut prev = line[0];
                let mut line = line
                    .iter()
                    .tuple_windows()
                    .flat_map(|(&a, &b)| {
                        let normal = (perp(a - prev) + perp(b - a)).normalize();
                        prev = a.clone();
                        [a + normal * wire.width, a - normal * wire.width]
                    })
                    .collect_vec();

                let normal = perp(line[line.len() - 1] - line[line.len() - 2]);
                line.push(line[line.len() - 1] + normal * wire.width);
                line.push(line[line.len() - 1] - normal * wire.width);
                verts.extend(line.into_iter().map(|pt| Vec3::new(pt.x, pt.y, WIRE_Z)));
                breaks.insert(verts.len());
            }

            let indices = (0..verts.len())
                .filter(|i| i % 2 == 0)
                .filter(|i| !breaks.contains(&(i + 2)))
                .flat_map(|i| [i, i + 3, i + 1, i, i + 2, i + 3]);
        }
    }
}

pub struct Lines {
    geometry: Vec<Vec<(Vec3, Vec4)>>,
    width: f32,
}

impl Lines {
    pub fn mesh(&self) -> Mesh {}
}

pub fn setup_env(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(LineList {
            lines: vec![
                (
                    Vec3::ZERO,
                    Vec3::new(100.0, 1.0, 0.0),
                    Vec4::new(1.0, 0.0, 0.0, 1.0),
                ),
                (
                    Vec3::new(1.0, 10.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec4::new(0.0, 1.0, 0.0, 1.0),
                ),
            ],
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: materials.add(LineMaterial {}),
        ..default()
    });
}
