use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct Ball;

#[derive(Component)]
pub struct Predator;

#[derive(Component, Debug)]
pub struct CustomID(pub u32);

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component)]
pub struct OriginPoint;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct PressedButton {
    pub button: u8,
}
