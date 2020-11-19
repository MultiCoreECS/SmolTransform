use std::time;
use SmolECS::{
    component::*,
    entity::*,
    system::*,
    rayon::*,
    world::*,
};
use std::sync::{Arc, Mutex};

#[derive(Copy, Clone)]
pub struct Position {
    x: f32,
    y: f32
}

//#[derive(Copy, Clone)]
pub struct Vertices(Vec<Position>);

#[derive(Copy, Clone)]
pub struct Rotation(f32);

#[derive(Copy, Clone)]
pub struct RotationalVelocity(f32);

#[derive(Copy, Clone)]
pub struct Time {
    beginning: time::Instant,
    last: time::Instant,
    total: f64,
    delta: f64
}

pub struct UpdateTime;
impl<'d, 'w: 'd> System<'d, 'w, World> for UpdateTime {
    type SystemData = (
        Write<'d, Time>
    );

    fn run(&self, (mut time): Self::SystemData) {
        let current = std::time::Instant::now();
        time.delta = current.duration_since(time.last).as_secs_f64();
        time.total = current.duration_since(time.beginning).as_secs_f64();
        time.last = current;
    }
}

pub struct ApplyRotationalVelocities;
impl<'d, 'w: 'd> System<'d, 'w, World> for ApplyRotationalVelocities{
    type SystemData = (
        ReadComp<'d, RotationalVelocity>,
        Read<'d, Time>,
        WriteComp<'d, Rotation>
    );

    fn run(&self, (vels, time, mut rots): Self::SystemData) {
        for (vel, rot) in (&vels, &mut rots).join(){
            rot.0 += vel.0 * time.delta as f32;
            rot.0 = rot.0.signum() * rot.0.abs() % 360.0;
        }
    }
}

fn main() {
    println!("Hello, world!");

    let mut world = World::new();
    world.register_comp::<Position>();
    world.register_comp::<Rotation>();
    world.register_comp::<RotationalVelocity>();

    world.insert(Time{
        beginning: std::time::Instant::now(),
        last: std::time::Instant::now(),
        total: 0.0,
        delta: 0.0,
    });
    world.insert(EntityStorage::new());

    let mut ents = Write::<EntityStorage>::get_data(&world);
    let mut positions = WriteComp::<Position>::get_data(&world);
    let mut angles = WriteComp::<Rotation>::get_data(&world);
    let mut angle_vel = WriteComp::<RotationalVelocity>::get_data(&world);

    ents.create_entity()
        .add(&mut positions, Position{x: 1.0, y: 2.0})
        .add(&mut angles, Rotation(100.0))
        .add(&mut angle_vel, RotationalVelocity(1.2));
    
    ents.create_entity()
        .add(&mut positions, Position{x: 0.0, y: -5.0})
        .add(&mut angles, Rotation(359.9))
        .add(&mut angle_vel, RotationalVelocity(10.0));
    
    let mut scheduler = SystemScheduler::new(Arc::new(ThreadPoolBuilder::new().num_threads(4).build().unwrap()));
    scheduler.add(UpdateTime{}, "update_time", vec![]);
    scheduler.add(ApplyRotationalVelocities{}, "update_angles", vec!["update_time"]);

    drop(ents);
    drop(positions);
    drop(angles);
    drop(angle_vel);

    for _ in 0..100 {
        scheduler.run(&world);
    }

    let positions = ReadComp::<Position>::get_data(&world);
    let angles = ReadComp::<Rotation>::get_data(&world);
    let angle_vels = ReadComp::<RotationalVelocity>::get_data(&world);

    for (position, angle, angle_vel) in (&positions, &angles, &angle_vels).join() {
        println!("({}, {}) : {} degs : {} per second", position.x, position.y, angle.0, angle_vel.0);
    }

    drop(positions);
    drop(angles);
    drop(angle_vels);
}
