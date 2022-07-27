use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    utils::tracing::Instrument,
};

use crate::{
    components::{OriginPoint, PressedButton},
    WinParams,
};

const degrees_90: f32 = 1.5708;

pub struct Hud;
impl Plugin for Hud {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_hud)
            .add_system(origin_update_system);
        // .add_system(ammo_update_system.system());
    }
}

fn origin_update_system(win_param: Res<WinParams>, mut query: Query<&mut Text, With<OriginPoint>>) {
    for mut text in query.iter_mut() {
        // println!("{:?}", text.sections[0].value);
        text.sections[0].value = format!("Origin: {}, {}", win_param.origin.x, win_param.origin.y);
    }
}

// TODO: get weapon ammo
// fn ammo_update_system(mut query: Query<&mut Text, With<AmmoCount>>) {
//     for mut text in query.iter_mut() {
//         if let ammo = 30 {
//             text.value = format!("III: {:.0}", ammo);
//         }
//     }
// }

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("setup is running");
    commands
        // 2d camera
        .spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(TextBundle {
            style: Style { ..default() },
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "hello\nbevy!",
                TextStyle {
                    font: asset_server.load("Karmatic-Arcade.ttf"),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..default()
                },
            ),
            ..default()
        })
        .insert(OriginPoint);

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(100.), Val::Px(60.)),

                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    right: Val::Px(15.0),
                    ..default()
                },

                ..Default::default()
            },
            color: UiColor(Color::rgb(0., 0., 0.)),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            top: Val::Px(5.0),
                            right: Val::Px(40.0),
                            ..default()
                        },
                        ..Default::default()
                    },

                    transform: Transform {
                        rotation: Quat::from_rotation_z(degrees_90),
                        ..Default::default()
                    },
                    image: asset_server.load("arrow.png").into(),
                    ..Default::default()
                })
                .insert(PressedButton { button: 1 });

            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            bottom: Val::Px(5.0),
                            left: Val::Px(10.0),
                            ..default()
                        },
                        ..Default::default()
                    },
                    transform: Transform {
                        rotation: Quat::from_rotation_z(degrees_90 * 2.0),
                        ..Default::default()
                    },
                    image: asset_server.load("arrow.png").into(),
                    ..Default::default()
                })
                .insert(PressedButton { button: 2 });

            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            bottom: Val::Px(5.0),
                            left: Val::Px(40.0),
                            ..default()
                        },
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(0., 0., 0.),
                        rotation: Quat::from_rotation_z(degrees_90 * 3.0),
                        ..Default::default()
                    },
                    image: asset_server.load("arrow.png").into(),
                    ..Default::default()
                })
                .insert(PressedButton { button: 3 });

            parent
                .spawn_bundle(ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(20.0), Val::Px(20.0)),
                        position_type: PositionType::Absolute,
                        position: Rect {
                            bottom: Val::Px(5.0),
                            right: Val::Px(10.0),
                            ..default()
                        },
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(10., 0., 0.),
                        ..Default::default()
                    },
                    image: asset_server.load("arrow.png").into(),
                    ..Default::default()
                })
                .insert(PressedButton { button: 4 });

            // parent
            //     .spawn_bundle(ImageBundle {
            //         style: Style {
            //             size: Size::new(Val::Px(20.0), Val::Px(20.0)),
            //             position_type: PositionType::Absolute,
            //             position: Rect {
            //                 left: Val::Px(10.0),
            //                 bottom: Val::Px(10.0),
            //                 ..Default::default()
            //             },
            //             ..Default::default()
            //         },
            //         image: asset_server.load("arrow.png").into(),
            //         ..Default::default()
            //     })
            //     .insert(PressedButton { button: 2 });

            // parent
            //     .spawn_bundle(ImageBundle {
            //         style: Style {
            //             size: Size::new(Val::Px(20.0), Val::Px(20.0)),
            //             position_type: PositionType::Absolute,
            //             position: Rect {
            //                 left: Val::Px(10.0),
            //                 bottom: Val::Px(10.0),
            //                 ..Default::default()
            //             },
            //             ..Default::default()
            //         },
            //         image: asset_server.load("arrow.png").into(),
            //         ..Default::default()
            //     })
            //     .insert(PressedButton { button: 3 });

            // parent
            //     .spawn_bundle(ImageBundle {
            //         style: Style {
            //             size: Size::new(Val::Px(20.0), Val::Px(20.0)),
            //             position_type: PositionType::Absolute,
            //             position: Rect {
            //                 left: Val::Px(10.0),
            //                 bottom: Val::Px(10.0),
            //                 ..Default::default()
            //             },
            //             ..Default::default()
            //         },
            //         image: asset_server.load("arrow.png").into(),
            //         ..Default::default()
            //     })
            //     .insert(PressedButton { button: 4 });
        });
}
// commands
//     .spawn_bundle(NodeBundle {
//         style: Style {
//             size: Size::new(Val::Px(200.0), Val::Px(200.0)),
//             position_type: PositionType::Absolute,
//             position: Rect {
//                 left: Val::Px(210.0),
//                 bottom: Val::Px(10.0),
//                 ..default()
//             },
//             ..default()
//         },
//         color: Color::NONE.into(),
//         ..default()
//     })
//     .with_children(|parent| {
//         // left vertical fill (border)

//         parent.spawn_bundle(SpriteBundle {
//             texture: asset_server.load("arrow.png"),
//             transform: Transform {
//                 translation: Vec3::new(0., 10., 0.),
//                 scale: Vec3::new(0.005, 0.005, 0.),
//                 ..Default::default()
//             },
//             ..Default::default()
//         });

//         parent.spawn_bundle(SpriteBundle {
//             texture: asset_server.load("arrow.png"),
//             transform: Transform {
//                 translation: Vec3::new(-10., 0., 0.),
//                 scale: Vec3::new(0.005, 0.005, 0.),
//                 ..Default::default()
//             },
//             ..Default::default()
//         });
//         parent.spawn_bundle(SpriteBundle {
//             texture: asset_server.load("arrow.png"),
// transform: Transform {
//     translation: Vec3::new(0., 0., 0.),
//     scale: Vec3::new(0.005, 0.005, 0.),
//     ..Default::default()
// },
//             ..Default::default()
//         });

//         parent.spawn_bundle(SpriteBundle {
//             texture: asset_server.load("arrow.png"),
//             transform: Transform {
//                 translation: Vec3::new(10., 0., 0.),
//                 scale: Vec3::new(0.005, 0.005, 0.),
//                 ..Default::default()
//             },
//             ..Default::default()
//         });
//     })
//     .insert(PressedButton { button: 1 });
