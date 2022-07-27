use crate::components::Ball;
use crate::components::Velocity;
// use bevy::app::AppExit;
use bevy::prelude::*;
use components::CustomID;
use components::Predator;

use components::MainCamera;
use components::PressedButton;
use hud::Hud;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use std::io::Read;
// use std::io::Write;
// use std::net::Shutdown;
// use std::net::TcpStream;
use std::net::UdpSocket;

mod components;
mod hud;

const BALL_SPRITE: &str = "ball.png";
const PREDATOR_SPRITE: &str = "wolf.png";
// const SERVER_ADDRESS: &str = "localhost:8000";

struct WinParams {
    origin: Vec3,
    w: f32,
    h: f32,
}

struct MapDimensions {
    origin: Vec3,
    w: f32,
    h: f32,
}
struct GameTextures {
    ball: Handle<Image>,
    predator: Handle<Image>,
}

struct Connection {
    // stream: TcpStream,
    socket: UdpSocket,
}
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
    TerminateConnection,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GameState {
    entity_state: EntityState,
    predator_state: PredatorState,
    boundary_state: Option<(Vec3, f32, f32)>,
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
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Client".to_string(),
            width: 700.0,
            height: 700.0,
            resizable: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system)
        .add_startup_system_to_stage(StartupStage::PostStartup, ball_spawn_system)
        .add_system(movement_update_system)
        .add_system(window_movement)
        // .add_system_to_stage(CoreStage::Last, exit_system)
        .add_plugin(Hud)
        .run();
}

fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut windows: ResMut<Windows>,
) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(MainCamera);

    let window = windows.get_primary_mut().unwrap();
    let (win_h, win_w) = (window.width(), window.height());

    //add window size resource
    let win_size = WinParams {
        origin: Vec3::new(0., 0., 0.),
        w: win_w,
        h: win_h,
    };
    commands.insert_resource(win_size);

    let map_dimensions = MapDimensions {
        origin: Vec3::new(0., 0., 0.),
        w: 0.,
        h: 0.,
    };
    commands.insert_resource(map_dimensions);

    let game_textures = GameTextures {
        ball: asset_server.load(BALL_SPRITE),
        predator: asset_server.load(PREDATOR_SPRITE),
    };

    commands.insert_resource(game_textures);

    //Add connection as resource
    let connection = Connection {
        // stream: TcpStream::connect(SERVER_ADDRESS).unwrap(),
        socket: UdpSocket::bind("127.0.0.1:8000").expect("Could not bind client socket")
    };
    connection.socket.connect("127.0.0.1:8888").expect("Could not connect to server");


    commands.insert_resource(connection);
}

fn ball_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,

    mut map_dimensions: ResMut<MapDimensions>,
    connection: ResMut<Connection>,
) {
    // Connect to server and notify that client is connected
    let message: ConnectionParams = ConnectionParams {
        status: "Connected".to_string(),
        connection_type: ConnectionType::Init,
        data: None,
        boundary: None,
    };
    let serialized = serde_json::to_string(&message).unwrap();
    // let _ = connection.stream.write(serialized.as_bytes());
    let _ = connection.socket.send(serialized.as_bytes());

    //Read entities from Server
    let mut buffer = [1; 80000];
    let len = connection.socket.recv(&mut buffer).unwrap();
    // let len = connection.stream.read(&mut buffer).unwrap();
    let message = String::from_utf8_lossy(&mut buffer[..len]);
    let deserialized_entity_state: ConnectionParams = serde_json::from_str(&message).unwrap();

    let (map_origin, map_width, map_height) = deserialized_entity_state.boundary.clone().unwrap();

    map_dimensions.origin = map_origin;
    map_dimensions.h = map_height;
    map_dimensions.w = map_width;

    let server_state = GameState {
        entity_state: deserialized_entity_state.data.clone().unwrap().entity_state,
        predator_state: deserialized_entity_state
            .data
            .clone()
            .unwrap()
            .predator_state,
        boundary_state: None,
    };

    let server_entity = server_state.entity_state;
    let server_predator = server_state.predator_state;

    for (
        server_entity,
        EntityPositionParams {
            translation,
            velocity:_,
            acceleration:_,
        },
    ) in server_entity.entity_atrib.iter()
    {
        commands
            .spawn_bundle(SpriteBundle {
                texture: game_textures.ball.clone(),
                transform: Transform {
                    translation: *translation,
                    scale: Vec3::new(0.05, 0.05, 0.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Velocity {
                x: 0.,
                y: 0.,
                z: 0.,
            })
            .insert(Ball)
            .insert(CustomID(*server_entity));
    }

    for (
        server_entity,
        PredatorPositionParams {
            translation,
            // velocity,
            // acceleration,
            velocity:_,
            acceleration:_,
        },
    ) in server_predator.predator_atrib.iter()
    {
        commands
            .spawn_bundle(SpriteBundle {
                texture: game_textures.predator.clone(),
                transform: Transform {
                    translation: *translation,
                    scale: Vec3::new(0.5, 0.5, 0.),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Velocity {
                x: 0.,
                y: 0.,
                z: 0.,
            })
            .insert(Predator)
            .insert(CustomID(*server_entity));
    }
}

fn movement_update_system(
    window_params: Res<WinParams>,
    connection: ResMut<Connection>,
    mut query: Query<(Entity, &mut CustomID, &mut Transform), (With<Ball>, Without<Predator>)>,
    mut query_predator: Query<
        (Entity, &mut CustomID, &mut Transform),
        (With<Predator>, Without<Ball>),
    >,
) {
    let boundary = (window_params.origin, window_params.w, window_params.h);
    let boundary_request = ConnectionParams {
        connection_type: ConnectionType::GetMovement,
        status: "Connected".to_string(),
        data: None,
        boundary: Some(boundary),
    };

    // ================= CLIENT SERVER COMMUNICATION ==================

    //Send the boundary to server here
    let serialized_boundary = serde_json::to_string(&boundary_request).unwrap();
    let _ = connection.socket.send(serialized_boundary.as_bytes());
    // let _ = connection.stream.write(serialized_boundary.as_bytes());

    //Read entities from Server
    let mut buffer = [1; 80000];
    let len = connection.socket.recv(&mut buffer).unwrap();
    // let len = connection.stream.read(&mut buffer).unwrap();
    let message = String::from_utf8_lossy(&mut buffer[..len]);
    // ================ END:CLIENT SERVER COMMUNICATION ==================

    if message.len() > 3 {
        let entities_within_bounds: ConnectionParams = serde_json::from_str(&message).unwrap();

        let server_state = GameState {
            entity_state: entities_within_bounds.data.clone().unwrap().entity_state,
            predator_state: entities_within_bounds.data.clone().unwrap().predator_state,
            boundary_state: None,
        };

        println!("{}", server_state.entity_state.entity_atrib.keys().len());

        // ================ PREY MOVEMENT  ====================

        for (_, customid, mut transform) in query.iter_mut() {
            if !server_state
                .entity_state
                .entity_atrib
                .contains_key(&customid.0)
            {
                transform.translation.x = 5000.;
            }
        }

        for (
            server_entity,
            EntityPositionParams {
                translation,
                // velocity,
                // acceleration,
                velocity:_,
                acceleration:_,
            },
        ) in server_state.entity_state.entity_atrib.iter()
        {
            for (_, customid, mut transform) in query.iter_mut() {
                //Client current entities
                if server_entity == &customid.0 {
                    transform.translation = *translation;
                }
            }
        }

        // ================== PREDATOR MOVEMENT  ====================

        for (_, customid, mut transform) in query_predator.iter_mut() {
            if !server_state
                .predator_state
                .predator_atrib
                .contains_key(&customid.0)
            {
                transform.translation.x = 5000.;
            }
        }

        for (
            server_predator,
            PredatorPositionParams {
                translation,
                // velocity,
                // acceleration,
                velocity:_,
                acceleration:_,
            },
        ) in server_state.predator_state.predator_atrib.iter()
        {
            for (_, customid, mut transform) in query_predator.iter_mut() {
                //Client current entities
                if server_predator == &customid.0 {
                    transform.translation = *translation;
                }
            }
        }
    }
}

fn window_movement(
    keyboard: Res<Input<KeyCode>>,
    map_dimensions: Res<MapDimensions>,
    asset_server: Res<AssetServer>,
    mut win_param: ResMut<WinParams>,
    mut cameras: Query<(&Camera, &mut Transform), With<MainCamera>>,
    mut arrows: Query<(&mut Style, &mut UiImage, &PressedButton), With<PressedButton>>,
) {
    let mut y_delta = 0.0;
    if keyboard.pressed(KeyCode::W) {
        y_delta += 10.;
    }
    if keyboard.pressed(KeyCode::S) {
        y_delta -= 10.;
    }

    let mut x_delta = 0.0;
    if keyboard.pressed(KeyCode::A) {
        x_delta -= 10.;
    }
    if keyboard.pressed(KeyCode::D) {
        x_delta += 10.;
    }

    //Uncomment when removing the current logic **DO NOT REMOVE**
    // win_param.origin.y += y_delta;
    // win_param.origin.x += x_delta;

    let arrow_enable_button: Handle<bevy::prelude::Image> = asset_server.load("arrow.png").into();
    let arrow_disable_button: Handle<bevy::prelude::Image> =
        asset_server.load("arrow_disabled.png").into();

    for (mut style, mut image, pressed_button) in arrows.iter_mut() {
        if y_delta > 0. {
            if pressed_button.button == 1 {
                if win_param.origin.y + y_delta <= (map_dimensions.h / 2. - win_param.h / 2.) {
                    image.0 = arrow_enable_button.clone();
                    win_param.origin.y += y_delta;
                } else {
                    image.0 = arrow_disable_button.clone();
                    win_param.origin.y += 0.;
                    y_delta = 0.;
                }
                style.size = Size::new(Val::Px(30.), Val::Px(30.));
            } else {
                image.0 = arrow_enable_button.clone();
                style.size = Size::new(Val::Px(20.), Val::Px(20.));
            }
        }

        if y_delta < 0. {
            if pressed_button.button == 3 {
                if win_param.origin.y + y_delta >= -(map_dimensions.h / 2. - win_param.h / 2.) {
                    image.0 = arrow_enable_button.clone();
                    win_param.origin.y += y_delta;
                } else {
                    image.0 = arrow_disable_button.clone();
                    win_param.origin.y += 0.;
                    y_delta = 0.;
                }
                style.size = Size::new(Val::Px(30.), Val::Px(30.));
            } else {
                image.0 = arrow_enable_button.clone();
                style.size = Size::new(Val::Px(20.), Val::Px(20.));
            }
        }

        if x_delta < 0. {
            if pressed_button.button == 2 {
                if win_param.origin.x + x_delta >= -(map_dimensions.w / 2. - win_param.w / 2.) {
                    image.0 = arrow_enable_button.clone();
                    win_param.origin.x += x_delta;
                } else {
                    image.0 = arrow_disable_button.clone();
                    win_param.origin.x += 0.;
                    x_delta = 0.;
                }
                style.size = Size::new(Val::Px(30.), Val::Px(30.));
            } else {
                image.0 = arrow_enable_button.clone();
                style.size = Size::new(Val::Px(20.), Val::Px(20.));
            }
        }

        if x_delta > 0. {
            if pressed_button.button == 4 {
                if win_param.origin.x + x_delta <= (map_dimensions.w / 2. - win_param.w / 2.) {
                    image.0 = arrow_enable_button.clone();
                    win_param.origin.x += x_delta;
                } else {
                    image.0 = arrow_disable_button.clone();
                    win_param.origin.x += 0.;
                    x_delta = 0.;
                }
                style.size = Size::new(Val::Px(30.), Val::Px(30.));
            } else {
                image.0 = arrow_enable_button.clone();
                style.size = Size::new(Val::Px(20.), Val::Px(20.));
            }
        }
    }

    for (_camera, mut pos) in cameras.iter_mut() {
        pos.translation += Vec3::new(x_delta, y_delta, 0.0);
    }
}

// fn exit_system(mut connection: ResMut<Connection>, mut events: EventReader<AppExit>) {
//     // ================= CLIENT SERVER COMMUNICATION ==================

//     for _ in events.iter() {
//         // === Write to Server console to close down connection
//         let close_connection_request = ConnectionParams {
//             connection_type: ConnectionType::TerminateConnection,
//             status: "Disconnect".to_string(),
//             data: None,
//             boundary: None,
//         };

//         //Send the boundary to server here
//         let serialized_boundary = serde_json::to_string(&close_connection_request).unwrap();
//         let _ = connection.socket.send(serialized_boundary.as_bytes());

//         // connection
//         //     .stream
//         //     .shutdown(Shutdown::Both)
//         //     .expect("Shutdown failed");
//     }
// }
