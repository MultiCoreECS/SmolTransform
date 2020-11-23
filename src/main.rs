use std::time;
use SmolECS::{
    component::*,
    entity::*,
    system::*,
    rayon::*,
    world::*,
};
use std::sync::{Arc, Mutex};
use clap::*;
use rand::*;

#[derive(Copy, Clone)]
pub struct Position {
    x: f32,
    y: f32
}

//pub struct Vertices(Vec<Position>);

#[derive(Copy, Clone)]
pub struct Rotation(f32);

#[derive(Copy, Clone)]
pub struct RotationalVelocity(f32);

#[derive(Copy, Clone)]
pub struct Scale(f32);

#[derive(Copy, Clone)]
pub struct Id(usize);

#[derive(Copy, Clone)]
pub struct Parent(usize);

pub struct Children<'a>(Vec<&'a Entity>);

#[derive(Copy, Clone)]
pub struct Time {
    beginning: time::Instant,
    last: time::Instant,
    total: f64,
    delta: f64
}

/*pub struct UpdateTime;
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
}*/

pub struct ApplyRotationalVelocities;
impl<'d, 'w: 'd> System<'d, 'w, World> for ApplyRotationalVelocities{
    type SystemData = (
        ReadComp<'d, RotationalVelocity>,
        //Read<'d, Time>,
        WriteComp<'d, Rotation>
    );

    fn run(&self, (vels, mut rots): Self::SystemData) {
        for (vel, rot) in (&vels, &mut rots).join(){
            rot.0 += vel.0;// * time.delta as f32;
            rot.0 = rot.0.signum() * rot.0.abs() % 360.0;
        }
    }
}

pub struct ApplyScaleAdjustment;
impl<'d, 'w: 'd> System<'d, 'w, World> for ApplyScaleAdjustment{
    type SystemData = (
        WriteComp<'d, Scale>
    );

    fn run(&self, mut scales: Self::SystemData) {
        let mut rng = rand::thread_rng();

        for (s) in (&mut scales).join() {
            s.0 += rng.gen_range(-0.5, 0.5);
        }
    }
}

pub struct ApplyTranslationAdjustment;
impl<'d, 'w: 'd> System<'d, 'w, World> for ApplyTranslationAdjustment{
    type SystemData = (
        WriteComp<'d, Position>
    );

    fn run(&self, mut positions: Self::SystemData) {
        let mut rng = rand::thread_rng();

        for (p) in (&mut positions).join() {
            p.x += rng.gen_range(-5.0, 5.0);
            p.y += rng.gen_range(-5.0, 5.0);
        }
    }
}

fn main() {
    let matches = App::new("SmolTransform")
        .version("1.0")
        .author("Blake Wyatt")
        .about("A transform hierarchy experiment")
        .arg(Arg::with_name("object_count")
            .short("c")
            .long("object_count")
            .help("Sets the number of objects to generate")
            .takes_value(true))
        .arg(Arg::with_name("update_iterations")
            .short("i")
            .long("update_iterations")
            .help("Sets the number of transform update iterations to perform")
            .takes_value(true))
        .arg(Arg::with_name("transform_type")
            .short("t")
            .long("transform_type")
            .help("Sets the type of transform. 0=all, 1=rotation, 2=scale, 3=translation")
            .takes_value(true))
        .get_matches();

    let object_count = matches.value_of("object_count").unwrap_or("10").parse::<i32>().unwrap_or(10);
    let update_iterations = matches.value_of("update_iterations").unwrap_or("100000").parse::<i32>().unwrap_or(100000);
    let transform_type = matches.value_of("transform_type").unwrap_or("0").parse::<i32>().unwrap_or(0);

    let mut world = World::new();
    world.register_comp::<Position>();
    world.register_comp::<Rotation>();
    world.register_comp::<RotationalVelocity>();
    //world.register_comp::<Children>();
    world.register_comp::<Parent>();
    world.register_comp::<Id>();

    world.insert(Time {
        beginning: std::time::Instant::now(),
        last: std::time::Instant::now(),
        total: 0.0,
        delta: 0.0,
    });
    world.insert(EntityStorage::new());

    let mut ents = Write::<EntityStorage>::get_data(&world);
    let mut positions = WriteComp::<Position>::get_data(&world);
    let mut scales = WriteComp::<Scale>::get_data(&world);
    let mut angles = WriteComp::<Rotation>::get_data(&world);
    let mut angle_vels = WriteComp::<RotationalVelocity>::get_data(&world);
    //let mut children_vecs = WriteComp::<Children>::get_data(&world);
    let mut parents = WriteComp::<Parent>::get_data(&world);
    let mut ids = WriteComp::<Id>::get_data(&world);
    
    ents.create_entity()
        .add(&mut ids, Id(0));

    let mut rng = rand::thread_rng();
    for id in 0..object_count {
        ents.create_entity()
            .add(&mut ids, Id(id as usize+1))
            .add(&mut positions, Position{x: rng.gen_range(0.0, 100.0), y: rng.gen_range(0.0, 100.0)})
            .add(&mut scales, Scale(rng.gen_range(0.0, 2.0)))
            .add(&mut angles, Rotation(rng.gen_range(0.0, 360.0)))
            .add(&mut angle_vels, RotationalVelocity(rng.gen_range(0.0, 1.0)))
            //.add(&mut children_vecs, Children(Vec::from([last_entity])));
            .add(&mut parents, Parent(id as usize));
    }
    
    let mut scheduler = SystemScheduler::new(Arc::new(ThreadPoolBuilder::new().num_threads(4).build().unwrap()));
    //scheduler.add(UpdateTime{}, "update_time", vec![]);
    match transform_type {
        0 => {
            scheduler.add(ApplyRotationalVelocities{}, "update_angles", vec![]);
            scheduler.add(ApplyScaleAdjustment{}, "update_scales", vec![]);
            scheduler.add(ApplyTranslationAdjustment{}, "update_position", vec![]);
        }
        1 => scheduler.add(ApplyRotationalVelocities{}, "update_angles", vec![]),
        2 => scheduler.add(ApplyScaleAdjustment{}, "update_scales", vec![]),
        3 => scheduler.add(ApplyTranslationAdjustment{}, "update_position", vec![]),
        _ => {
            println!("Invalid transform type. Action cancelled.");
            return;
        }
    }
    
    drop(ents);
    drop(ids);
    drop(parents);
    drop(positions);
    drop(scales);
    drop(angles);
    drop(angle_vels);

    for _ in 0..update_iterations {
        scheduler.run(&world);
    }

    let ids = ReadComp::<Id>::get_data(&world);
    let parents = ReadComp::<Parent>::get_data(&world);
    let positions = ReadComp::<Position>::get_data(&world);
    let scales = ReadComp::<Scale>::get_data(&world);
    let angles = ReadComp::<Rotation>::get_data(&world);
    let angle_vels = ReadComp::<RotationalVelocity>::get_data(&world);

    for (id, parent, position, scale, angle, angle_vel) in (&ids, &parents, &positions, &scales, &angles, &angle_vels).join() {
        println!("ID {} : Parent {} : ({}, {}) : Scaled {} units : {} degs : {} per second", id.0, parent.0, position.x, position.y, scale.0, angle.0, angle_vel.0);
    }

    drop(ids);
    drop(parents);
    drop(positions);
    drop(angles);
    drop(angle_vels);
}
