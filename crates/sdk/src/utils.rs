use bevy::color::{Color, ColorToPacked};
use bevy::prelude::Srgba;
use egui::Color32;

pub enum SdkColor {
    Bevy(Color),
    Egui(Color32),
}

impl From<Color> for SdkColor {
    fn from(val: Color) -> Self {
        Self::Bevy(val)
    }
}

impl From<Color32> for SdkColor {
    fn from(val: Color32) -> Self {
        Self::Egui(val)
    }
}

impl From<SdkColor> for Color32 {
    fn from(val: SdkColor) -> Self {
        match val {
            SdkColor::Bevy(val) => {
                let [r, g, b, a] = val.to_srgba().to_u8_array();
                Color32::from_rgba_premultiplied(r, g, b, a)
            }
            SdkColor::Egui(val) => val,
        }
    }
}

impl From<SdkColor> for Color {
    fn from(val: SdkColor) -> Self {
        match val {
            SdkColor::Bevy(val) => val,
            SdkColor::Egui(val) => {
                let [red, green, blue, alpha] = val.to_normalized_gamma_f32();
                Self::Srgba(Srgba {
                    red,
                    green,
                    blue,
                    alpha,
                })
            }
        }
    }
}
