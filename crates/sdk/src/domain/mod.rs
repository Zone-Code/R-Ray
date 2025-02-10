use std::borrow::Cow;
use crate::utils;
use bevy::color::palettes::tailwind;
use bevy::color::{Color, Srgba};
use bevy::prelude::Component;
use bevy_reflect::Reflect;
use egui::Color32;
use egui_lucide_icons::icons;

#[derive(Component, Reflect, Clone, Debug)]
pub struct SdkEntityIcon {
    pub path: Cow<'static, str>,
    pub color: Color,
}

impl Default for SdkEntityIcon {
    fn default() -> Self {
        Self {
            path: Cow::from(icons::lucide::HEXAGON),
            color: utils::SdkColor::Bevy(Color::from(tailwind::GRAY_400)).into(),
        }
    }
}
impl SdkEntityIcon {
    #[inline(always)]
    pub fn new(path: impl Into<Cow<'static, str>>, color: Color) -> Self {
        fn new(path: Cow<'static, str>, color: Color) -> SdkEntityIcon {
            SdkEntityIcon { path, color }
        }
        new(path.into(), color)
    }

    pub fn icon(path: impl Into<Cow<'static, str>>) -> Self {
        Self::new(path, utils::SdkColor::Bevy(Color::from(tailwind::GREEN_400)).into())
    }
}
