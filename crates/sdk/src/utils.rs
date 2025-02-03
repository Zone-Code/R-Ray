use bevy::prelude::Srgba;
use egui::{Color32, Rgba};

pub struct SrgbaExt(pub Srgba);

impl From<Srgba> for SrgbaExt {
    fn from(value: Srgba) -> Self {
        SrgbaExt(value)
    }
}

impl From<SrgbaExt> for Color32 {
    fn from(value: SrgbaExt) -> Self {
        Color32::from_hex(value.0.to_hex().as_str()).unwrap()
    }
}