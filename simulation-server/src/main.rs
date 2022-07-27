use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use spmc::{Receiver, Sender};
use std::collections::HashMap;
use std::io::{Error, Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::ops::{Add, Div, Mul, Sub};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{thread, time};

const NUMBEROFBEADS: u32 = 20;
const PREDATOR_MAX_COUNT: u32 = 0;
const MAP_DIMENSION: (f32, f32) = (1000., 1000.);
const MAX_SPEED: f32 = 2.;
const ALIGNMENT_RADIUS: f32 = 50.;
const COHESION_RADIUS: f32 = 50.;
const DESIRED_SEPARATION: f32 = 25.;
const EVADING_RADIUS: f32 = 150.;
const PERCEPTION_RADIUS: f32 = 100.;

#[derive(Serialize, Deserialize, Debug)]
struct ConnectionParams {
    status: String,
    connection_type: ConnectionType,
    data: Option<GameState>,
    boundary: Option<(Vec3, f32, f32)>,
}

#[derive(Serialize, Deserialize, Debug)]
enum ConnectionType {
    Init,
    GetMovement,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameState {
    entity_state: EntityState,
    predator_state: PredatorState,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EntityState {
    entity_atrib: HashMap<u32, EntityPositionParams>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PredatorState {
    predator_atrib: HashMap<u32, PredatorPositionParams>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EntityPositionParams {
    translation: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PredatorPositionParams {
    translation: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    chasing: bool,
}

struct Scalar {
    value: f32,
}

#[derive(Debug, PartialEq)]
struct Vector {
    value: Vec3,
}

// Trait to divide a vector by a scalar
impl Div<Scalar> for Vector {
    type Output = Self;

    fn div(self, divider: Scalar) -> Vector {
        return Vector {
            value: Vec3::new(
                self.value.x / divider.value,
                self.value.y / divider.value,
                self.value.z / divider.value,
            ),
        };
    }
}

trait Send {
    fn send_data(&self, packet: &ConnectionParams, addr: &str);
}

impl Send for UdpSocket {
    fn send_data(&self, packet: &ConnectionParams, addr: &str) {

        let json_str = serde_json::to_string(&packet).unwrap();
        self.send_to(&json_str.as_bytes(), &addr).expect("Unable to send data!");
        
    }   
}

fn main() {
    // let listener = TcpListener::bind("127.0.0.1:8000").expect("Could not bind");
    let socket = UdpSocket::bind("0.0.0.0:8888").expect("Could not bind socket");

    // let (tx, rx) = spmc::channel();
    let mut handles = vec![];

    let mut beads: EntityState = EntityState {
        entity_atrib: HashMap::new(),
    };
    let mut predators: PredatorState = PredatorState {
        predator_atrib: HashMap::new(),
    };

    spawn_entities(&mut beads, &mut predators);

    let game_state = GameState {
        entity_state: beads,
        predator_state: predators,
    };

    let g_state = Arc::new(RwLock::new(game_state.clone()));
    let game_state_info = Arc::new(RwLock::new(game_state));
    
    let game_info = Arc::clone(&game_state_info);
    let game_info2 = Arc::clone(&game_state_info);

    let g_state2 = Arc::clone(&g_state);
    let g_state3 = Arc::clone(&g_state);

    handles.push(thread::spawn(move || {
        handle_all_movements(&game_info);
    }));

    handles.push(thread::spawn(move || {
        simulaiton_tick(&game_info2, &g_state2);  
    }));

    handles.push(thread::spawn(move || {
        handle_connection(socket, &g_state3)
        .unwrap_or_else(|error| eprintln!("{:?}", error));
    }));

    for handle in handles {
        handle.join().unwrap();
    }

    // let lock = Arc::new(RwLock::new(game_state));
    // let c_lock = Arc::clone(&lock);

    // handles.push(thread::spawn(move || {
    //     handle_all_movements(&c_lock);
    // }));

    // let c_lock = Arc::clone(&lock);
    // handles.push(thread::spawn(move || {
    //     handle_connection(socket, &c_lock)
    //     .unwrap_or_else(|error| eprintln!("{:?}", error));
    // }));
    // // for stream in listener.incoming()
    // //  {
    // //     // let rx = rx.clone();
    // //     match stream {
    // //         Err(e) => {
    // //             eprintln!("failed: {}", e)
    // //         }

    // //         Ok(stream) => {
    // //             let c_lock = Arc::clone(&lock);
    // //             clients.push(thread::spawn(move || {
    // //                 handle_connection(stream, &c_lock)
    // //                     .unwrap_or_else(|error| eprintln!("{:?}", error));
    // //             }));
    // //         }
    // //     }
    // // }
    // for handle in handles {
    //     handle.join().unwrap();
    // }
}

fn spawn_entities(beads: &mut EntityState, predators: &mut PredatorState) {
    let mut count: u32 = 0;
    let mut rng = thread_rng();

    while count < NUMBEROFBEADS {
        // RANDOM Translation & Velocity
        let pos_x: f32 = rng.gen_range(-200.0..200.0);
        let pos_y: f32 = rng.gen_range(-200.0..200.0);
        let vel_x: f32 = rng.gen_range(-2.0..2.0);
        let vel_y: f32 = rng.gen_range(-2.0..2.0);

        let translation = Vec3::new(pos_x, pos_y, 0.);
        let velocity = Vec3::new(vel_x, vel_y, 0.);
        let acceleration = Vec3::new(0., 0., 0.);

        beads.entity_atrib.insert(
            count,
            EntityPositionParams {
                translation,
                velocity,
                acceleration,
            },
        );
        count += 1;
    }

    count = 0;

    while count < PREDATOR_MAX_COUNT {
        // RANDOM Translation & Velocity
        let pos_x: f32 = rng.gen_range(-200.0..200.0);
        let pos_y: f32 = rng.gen_range(-200.0..200.0);
        let vel_x: f32 = rng.gen_range(-2.0..2.0);
        let vel_y: f32 = rng.gen_range(-2.0..2.0);

        let translation = Vec3::new(pos_x, pos_y, 0.);
        let velocity = Vec3::new(vel_x, vel_y, 0.);
        let acceleration = Vec3::new(0., 0., 0.);
        let chasing = false;

        predators.predator_atrib.insert(
            count,
            PredatorPositionParams {
                translation,
                velocity,
                acceleration,
                chasing,
            },
        );
        count += 1;
    }
}

fn handle_all_movements(lock: &RwLock<GameState>) {
    loop {

        let ten_millis = Duration::from_millis(15);
        thread::sleep(ten_millis);
        {
            let mut w = lock.write().unwrap();

            let game_state = &mut *w;
            handle_bead_movement(game_state);
            handle_predator_movement_system(game_state);
            // tx.send(game_state.clone()).unwrap();
        }

    }
}

fn handle_bead_movement(game_state: &mut GameState) {
    let mut boids = Vec::new();

    //Iterate through the entities and it's component data and store them in a local vector
    for (
        entity,
        EntityPositionParams {
            translation,
            velocity,
            acceleration,
        },
    ) in game_state.entity_state.entity_atrib.iter_mut()
    {
        boids.push((entity.clone(), velocity.clone(), translation.clone()));
    }

    //Iterate through the entities
    for (
        entity,
        EntityPositionParams {
            translation,
            velocity,
            acceleration,
        },
    ) in game_state.entity_state.entity_atrib.iter_mut()
    {
        //Cohesion variables
        let mut cohesion_count = Scalar { value: 0. };
        let mut cohesion_sum = Vec3::new(0., 0., 0.);
        let mut cohesion_steer = Vec3::new(0., 0., 0.);

        //Separation variables
        let mut separation_count = Scalar { value: 0. };
        let mut separation_steer = Vec3::new(0., 0., 0.);

        //Alignment variables
        let mut alignment_count = Scalar { value: 0. };
        let mut alignment_sum = Vec3::new(0., 0., 0.);
        let mut alignment_steer = Vec3::new(0., 0., 0.);

        //Iterate through all entities again and compare the `entity` data with all other `boid_entity` data
        for (boids_entity, boids_velocity, boids_transform) in boids.iter_mut() {
            if entity != boids_entity {
                let distance = boids_transform.distance(*translation);

                // --- COHESION ---
                if distance > 0. && distance < COHESION_RADIUS {
                    cohesion_sum = cohesion_sum.add(*boids_transform);
                    cohesion_count.value += 1.;
                }

                // --- SEPARATION ---
                // Looks for boids within the Desired SEPERATION radius and makes a count of it
                if distance > 0. && distance < DESIRED_SEPARATION {
                    let mut difference = Vec3::new(
                        translation.x.sub(boids_transform.x),
                        translation.y.sub(boids_transform.y),
                        translation.z.sub(boids_transform.z),
                    );

                    difference = difference.normalize();
                    difference = difference.div(distance);
                    separation_steer = separation_steer.add(difference);
                    separation_count.value += 1.;
                }

                // --- ALIGNMENT ---
                // Looks for boids within the ALIGNMENT radius and makes a count of it
                if distance > 0. && distance < ALIGNMENT_RADIUS {
                    alignment_sum.x += boids_velocity.x;
                    alignment_sum.y += boids_velocity.y;
                    alignment_count.value += 1.;
                }
            }
        }

        // --- COHESION Contd. ---
        if cohesion_count.value > 0. {
            let average = cohesion_sum.div(cohesion_count.value);
            let mut desired = average.sub(*translation);
            desired = desired.normalize();
            let temp_steer = Vec3::new(desired.x - velocity.x, desired.y - velocity.y, 0.);
            cohesion_steer = temp_steer;
        }

        // --- SEPARATION Contd. ---
        if separation_count.value > 0. {
            separation_steer = separation_steer.div(separation_count.value);
        }

        let steer_mag = ((separation_steer.x * separation_steer.x)
            + (separation_steer.y * separation_steer.y)
            + (0.))
            .sqrt();
        if steer_mag > 0. {
            separation_steer = separation_steer.normalize();
            separation_steer = separation_steer.mul(MAX_SPEED);
            let temp_steer;
            temp_steer = Vec3::new(
                separation_steer.x - velocity.x,
                separation_steer.y - velocity.y,
                separation_steer.z - velocity.z,
            );
            separation_steer = temp_steer;
        }

        // --- AlIGNMENT Contd. ---
        if alignment_count.value > 0. {
            let mut average = alignment_sum.div(alignment_count.value);
            average = average.normalize();
            average = average.mul(MAX_SPEED);
            let temp_vec = Vec3::new(average.x.sub(velocity.x), average.y.sub(velocity.y), 0.);
            alignment_steer = temp_vec;
        }

        // --- ALIGNMENT END ---

        //Get all values after calculations
        let mut separation = separation_steer;
        let mut cohesion = cohesion_steer;
        let mut alignment = alignment_steer;

        // Mutliply it with coefficient - adjust values to
        separation = separation.mul(0.5);
        cohesion = cohesion.mul(0.05);
        alignment = alignment.mul(0.5);

        // Update the accelaration with the calculations made
        acceleration.x += cohesion.x;
        acceleration.y += cohesion.y;
        acceleration.z += cohesion.z;

        acceleration.x += separation.x;
        acceleration.y += separation.y;
        acceleration.z += separation.z;

        acceleration.x += alignment.x;
        acceleration.y += alignment.y;
        acceleration.z += alignment.z;

        velocity.x = velocity.x.add(acceleration.x);
        velocity.y = velocity.y.add(acceleration.y);
        translation.x += velocity.x;
        translation.y += velocity.y;

        // acceleration = acceleration.mul(0.);
        acceleration.x = 0.;
        acceleration.y = 0.;
        acceleration.z = 0.;

        // =============== APPEAR ON OPPOSITE SIDE OF BOUNDS =================

        let bounds_x: f32 = MAP_DIMENSION.0 / 2.;
        let bounds_y: f32 = MAP_DIMENSION.1 / 2.;

        if translation.x > bounds_x {
            translation.x = -bounds_x
        } else if translation.x < -bounds_x {
            translation.x = bounds_x;
        }
        if translation.y > bounds_y {
            translation.y = -bounds_y;
        } else if translation.y < -bounds_y {
            translation.y = bounds_y;
        }

        // ======================= EVASION LOGIC ==========================
        let prey_translation = translation;
        let prey_velocity = velocity;

        for (
            _,
            PredatorPositionParams {
                translation,
                velocity,
                acceleration,
                chasing,
            },
        ) in game_state.predator_state.predator_atrib.iter_mut()
        {
            let predator_translation = translation;
            let distance = predator_translation.distance(*prey_translation);

            if distance <= EVADING_RADIUS {
                let direction_to_move_x = (predator_translation.x - prey_translation.x) * -1.;
                let direction_to_move_y = (predator_translation.y - prey_translation.y) * -1.;

                let normalized_vector =
                    Vec3::new(direction_to_move_x, direction_to_move_y, 0.).normalize();

                prey_velocity.x = normalized_vector.x * 0.5;
                prey_velocity.y = normalized_vector.y * 0.5;
            }
        }
    }
}

fn handle_predator_movement_system(game_state: &mut GameState) {
    for (
        _,
        PredatorPositionParams {
            translation,
            velocity,
            acceleration,
            chasing,
        },
    ) in game_state.predator_state.predator_atrib.iter_mut()
    {
        let translation_pred = translation;
        let velocity_pred = velocity;
        let mut chasing_state = chasing;

        for (
            prey_entity,
            EntityPositionParams {
                translation,
                velocity,
                acceleration,
            },
        ) in game_state.entity_state.entity_atrib.clone().iter_mut()
        {
            // =============== PREDATOR CHASING LOGIC ===========================

            let translation_prey = translation;
            let distance = translation_prey.distance(translation_pred.clone());

            //Check if a predator is within the vicinity radius
            if distance > 0. && distance < PERCEPTION_RADIUS {
                let collision = collide(
                    *translation_pred,
                    Vec2::splat(20.),
                    *translation_prey,
                    Vec2::splat(20.),
                );

                // Implementation for predator catching a prey
                if let Some(_) = collision {
                    //Remove prey when predator captures it
                    game_state.entity_state.entity_atrib.remove(prey_entity);
                    //Predator back to random movement
                    chasing_state = &mut false;
                }

                // Run towards the prey
                velocity_pred.x = (translation_prey.x - translation_pred.x) * 0.05;
                velocity_pred.y = (translation_prey.y - translation_pred.y) * 0.05;
            }
        }

        translation_pred.x += velocity_pred.x;
        translation_pred.y += velocity_pred.y;

        // =============== APPEAR ON OPPOSITE SIDE OF BOUNDS =================

        let bounds_x: f32 = MAP_DIMENSION.0 / 2.;
        let bounds_y: f32 = MAP_DIMENSION.1 / 2.;

        if translation_pred.x > bounds_x {
            translation_pred.x = -bounds_x
        } else if translation_pred.x < -bounds_x {
            translation_pred.x = bounds_x;
        }
        if translation_pred.y > bounds_y {
            translation_pred.y = -bounds_y;
        } else if translation_pred.y < -bounds_y {
            translation_pred.y = bounds_y;
        }
    }
}

fn handle_connection(sock: UdpSocket, lock: &RwLock<GameState>) -> Result<(), Error> {
    loop {
        //100 beads & predator
        // let game_state = rx.recv();

        let game_state;
        {
            let r = lock.read().unwrap();
            let st = &*r;
            game_state = st.clone();
        }

        //Read from client
        let mut buffer = [1; 80000];
        let (len, addr) = sock.recv_from(&mut buffer).unwrap();
        // let len = stream.read(&mut buffer).unwrap();
        
        let message = String::from_utf8_lossy(&mut buffer[..len]);
        let connection_request: ConnectionParams = serde_json::from_str(&message).unwrap();

        let mut entities_in_frame: EntityState = EntityState {
            entity_atrib: HashMap::new(),
        };
        let mut predators_in_frame: PredatorState = PredatorState {
            predator_atrib: HashMap::new(),
        };

        match connection_request.connection_type {
            //If connection created - Spawn 100 predator and entities and pass IDs
            ConnectionType::Init => {
                //Write to client
                let game_data = ConnectionParams {
                    status: "Connected".to_string(),
                    connection_type: ConnectionType::Init,
                    data: Some(game_state),
                    boundary: Some((Vec3::new(0., 0., 0.), MAP_DIMENSION.0, MAP_DIMENSION.1)),
                };
                let serialized_entity_data = serde_json::to_string(&game_data).unwrap();
                // let _ = stream.write(serialized_entity_data.as_bytes());
                // let _ = stream.flush();
                let _ = sock.send_to(serialized_entity_data.as_bytes(), addr);
            }
            ConnectionType::GetMovement => {
                let (origin, width, height) = connection_request.boundary.unwrap();
                let half_width = width / 2.0;
                let half_height = height / 2.0;
                // println!("{:?}", origin);
                for (
                    entity,
                    EntityPositionParams {
                        translation,
                        velocity,
                        acceleration,
                    },
                ) in game_state.entity_state.entity_atrib
                {
                    if (translation.x > origin.x
                        && translation.x < origin.x + half_width
                        && translation.y > origin.y
                        && translation.y < origin.y + half_height)
                        || (translation.x < origin.x
                            && translation.x > origin.x - half_width
                            && translation.y < origin.y
                            && translation.y > origin.y - half_height)
                        || (translation.x < origin.x
                            && translation.x > origin.x - half_width
                            && translation.y > origin.y
                            && translation.y < origin.y + half_height)
                        || (translation.x > origin.x
                            && translation.x < origin.x + half_width
                            && translation.y < origin.y
                            && translation.y > origin.y - half_height)
                    {
                        entities_in_frame.entity_atrib.insert(
                            entity,
                            EntityPositionParams {
                                translation,
                                velocity,
                                acceleration,
                            },
                        );
                    }
                }

                for (
                    predator,
                    PredatorPositionParams {
                        translation,
                        velocity,
                        acceleration,
                        chasing,
                    },
                ) in game_state.predator_state.predator_atrib
                {
                    if (translation.x > origin.x
                        && translation.x < origin.x + half_width
                        && translation.y > origin.y
                        && translation.y < origin.y + half_height)
                        || (translation.x < origin.x
                            && translation.x > origin.x - half_width
                            && translation.y < origin.y
                            && translation.y > origin.y - half_height)
                        || (translation.x < origin.x
                            && translation.x > origin.x - half_width
                            && translation.y > origin.y
                            && translation.y < origin.y + half_height)
                        || (translation.x > origin.x
                            && translation.x < origin.x + half_width
                            && translation.y < origin.y
                            && translation.y > origin.y - half_height)
                    {
                        predators_in_frame.predator_atrib.insert(
                            predator,
                            PredatorPositionParams {
                                translation,
                                velocity,
                                acceleration,
                                chasing,
                            },
                        );
                    }
                }

                let filtered_game_state = GameState {
                    entity_state: entities_in_frame,
                    predator_state: predators_in_frame,
                };
                // println!("{:?}", filtered_game_state.entity_state);

                //Write to client
                let game_data = ConnectionParams {
                    status: "Connected".to_string(),
                    connection_type: ConnectionType::GetMovement,
                    data: Some(filtered_game_state),
                    boundary: None,
                };

                let serialized = serde_json::to_string(&game_data).unwrap();
                // stream.write(serialized.as_bytes())?;
                // let _ = stream.flush();
                sock.send_to(serialized.as_bytes(), addr)?;
            }
        }
    }
}

fn simulaiton_tick(info: &RwLock<GameState>, state: &RwLock<GameState>) {

    let time_step: f32 = 0.03333; // 1/30 aproximated upto 5 decimal places
    let mut tick = 0;
    
    loop {
        while tick != 30 {

            let time_instance = time::Instant::now(); //Reseting the stop-watch
                
                while time_instance.elapsed().as_secs_f32() <= time_step {

                    let mut w = state.write().unwrap();
                    let game_state = &mut *w;

                    let r = info.read().unwrap();
                    let st = &*r;
                    let game_info = st.clone();

                    *game_state = game_info;
                    
                    thread::sleep(Duration::from_secs((0.033  - time_instance.elapsed().as_secs_f32()) as u64 ));
                    println!("time elapsed {}", time_instance.elapsed().as_secs_f32());
                }
            tick += 1;
            println!("{}", tick);
        }
        tick = 0;
    }
}