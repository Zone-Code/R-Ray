use crate::bevy_inspector::{EntityFilter, Filter};
use crate::domain::SdkEntityIcon;
use crate::ui::utils;
use crate::utils::SdkColor;
use bevy::color::Color;
use bevy::color::Color::Srgba;
use bevy::ecs::query::QueryFilter;
use bevy::prelude::{AppTypeRegistry, Children, Entity, Parent, Resource, Without, World};
use bevy_inspector_egui::bevy_inspector::hierarchy::{SelectedEntities, SelectionMode};
use bevy_reflect::TypeRegistry;
use egui::collapsing_header::CollapsingState;
use egui::{include_image, pos2, vec2, CollapsingHeader, Color32, Id, Image, ImageSource, RichText, Rounding, Sense, Stroke, TextEdit, Vec2};
use egui_lucide_icons::icons;
use std::borrow::Cow;
use std::collections::HashSet;
use std::ops::Div;
use bevy::color::palettes::tailwind;
use bevy::core::Name;
use itertools::Itertools;
use smart_default::SmartDefault;
use crate::ui::utils::guess_entity_name::guess_entity_name;

#[derive(Resource, SmartDefault, Clone)]
struct HierarchyExtraState {
    #[default("")]
    pub input: String,
    pub is_regex: bool
}

/// Display UI of the entity hierarchy.
///
/// Returns `true` if a new entity was selected.
pub fn hierarchy_ui(world: &mut World, ui: &mut egui::Ui, selected: &mut SelectedEntities) -> bool {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    world.init_resource::<HierarchyExtraState>();

    Hierarchy {
        world,
        type_registry: &type_registry,
        selected,
        context_menu: None,
        shortcircuit_entity: None,
        extra_state: &mut (),
    }
    .show::<()>(ui)
}

pub struct Hierarchy<'a, T = ()> {
    pub world: &'a mut World,
    pub type_registry: &'a TypeRegistry,
    pub selected: &'a mut SelectedEntities,
    pub context_menu: Option<&'a mut dyn FnMut(&mut egui::Ui, Entity, &mut World, &mut T)>,
    pub shortcircuit_entity:
        Option<&'a mut dyn FnMut(&mut egui::Ui, Entity, &mut World, &mut T) -> bool>,
    pub extra_state: &'a mut T,
}

impl<T> Hierarchy<'_, T> {
    pub fn show<QF>(&mut self, ui: &mut egui::Ui) -> bool
    where
        QF: QueryFilter,
    {
        let filter: Filter = Filter::all();
        self._show::<QF, _>(ui, filter)
    }
    pub fn show_with_default_filter<QF>(&mut self, ui: &mut egui::Ui) -> bool
    where
        QF: QueryFilter,
    {
        let filter: Filter = Filter::from_ui(ui, egui::Id::new("default_hierarchy_filter"));
        self._show::<QF, _>(ui, filter)
    }
    pub fn show_with_filter<QF, F>(&mut self, ui: &mut egui::Ui, filter: F) -> bool
    where
        QF: QueryFilter,
        F: EntityFilter,
    {
        self._show::<QF, F>(ui, filter)
    }
    fn _show<QF, F>(&mut self, ui: &mut egui::Ui, filter: F) -> bool
    where
        QF: QueryFilter,
        F: EntityFilter,
    {
        let mut root_query = self.world.query_filtered::<Entity, (Without<Parent>, QF)>();

        let always_open: HashSet<Entity> = self
            .selected
            .iter()
            .flat_map(|selected| {
                std::iter::successors(Some(selected), |&entity| {
                    self.world.get::<Parent>(entity).map(|parent| parent.get())
                })
                .skip(1)
            })
            .collect();

        let mut entities: Vec<_> = root_query.iter(self.world).collect();
        filter.filter_entities(self.world, &mut entities);
        entities.sort();

        let hierarchy_extra_state = &mut self.world.get_resource_mut::<HierarchyExtraState>().unwrap();

        ui.add(
            TextEdit::singleline(&mut hierarchy_extra_state.input)
                .hint_text(
                    RichText::new("Search").color(
                        SdkColor::Bevy(Srgba(tailwind::GRAY_400))
                    )
                )
        );

        ui.horizontal(|ui|{
            ui.checkbox(&mut hierarchy_extra_state.is_regex, "Regex");
        });
        ui.separator();

        let mut selected = false;
        for &entity in &entities {
            selected |= self.entity_ui(ui, entity, &always_open, &entities, &filter);
        }
        selected
    }

    fn entity_ui<F>(
        &mut self,
        ui: &mut egui::Ui,
        entity: Entity,
        always_open: &HashSet<Entity>,
        at_same_level: &[Entity],
        filter: &F,
    ) -> bool
    where
        F: EntityFilter,
    {
        let mut new_selection = false;
        let selected = self.selected.contains(entity);

        let entity_name = utils::guess_entity_name::guess_entity_name(self.world, entity);
        let mut name = RichText::new(entity_name);
        if selected {
            name = name.strong();
        }

        let has_children = self
            .world
            .get::<Children>(entity)
            .is_some_and(|children| children.len() > 0);

        let open = if !has_children {
            Some(false)
        } else if always_open.contains(&entity) {
            Some(true)
        } else {
            None
        };

        if let Some(shortcircuit_entity) = self.shortcircuit_entity.as_mut() {
            if shortcircuit_entity(ui, entity, self.world, self.extra_state) {
                return false;
            }
        }

        let id = Id::new(entity);
        let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, false);

        let children = self.world.get::<Children>(entity);

        let header_response = ui.horizontal(|ui| {
            if let Some(_) = children {
                state.show_toggle_button(ui, paint_expand_icon);
            } else {
                let size = vec2(ui.spacing().indent, ui.spacing().icon_width);
                let (_id, rect) = ui.allocate_space(size);
                let response = ui.interact(rect, state.id(), Sense::click());

                let (mut icon_rect, _) = ui.spacing().icon_rectangles(response.rect);
                icon_rect.set_center(pos2(
                    response.rect.left() + ui.spacing().indent / 2.0,
                    response.rect.center().y,
                ));
            }

            let result = self.world.get_entity(entity).unwrap();

            let sdk_entity_icon = result
                .get::<SdkEntityIcon>()
                .map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Owned(SdkEntityIcon::default()));

            ui.add(
                Image::new(format!("file://{}", sdk_entity_icon.path))
                    .tint(SdkColor::from(sdk_entity_icon.color))
                    .fit_to_exact_size(Vec2::splat(18.0)),
            );

            ui.selectable_label(false, name)
        });

        state.show_body_unindented(ui, |ui| {
            let children = self.world.get::<Children>(entity);
            if let Some(children) = children {
                let mut children = children.to_vec();
                filter.filter_entities(self.world, &mut children);
                for &child in &children {
                    // ui.spacing_mut().indent += ui.spacing().icon_width;
                    new_selection |= ui
                        .indent(id, |ui| {
                            self.entity_ui(ui, child, always_open, &children, filter)
                        })
                        .inner;
                }
            }
        });

        if header_response.inner.clicked() {
            let selection_mode = ui.input(|input| {
                SelectionMode::from_ctrl_shift(input.modifiers.ctrl, input.modifiers.shift)
            });
            let extend_with = |from, to| {
                // PERF: this could be done in one scan
                let from_position = at_same_level.iter().position(|&entity| entity == from);
                let to_position = at_same_level.iter().position(|&entity| entity == to);
                from_position
                    .zip(to_position)
                    .map(|(from, to)| {
                        let (min, max) = if from < to { (from, to) } else { (to, from) };
                        at_same_level[min..=max].iter().copied()
                    })
                    .into_iter()
                    .flatten()
            };
            self.selected.select(selection_mode, entity, extend_with);
            new_selection = true;
        }

        /*if let Some(context_menu) = self.context_menu.as_mut() {
            stat
                .context_menu(|ui| context_menu(ui, entity, self.world, self.extra_state));
        }*/

        new_selection
    }
}

fn paint_expand_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response) {
    let visuals = ui.style().interact(response);
    let stroke = visuals.fg_stroke;

    let rect = response.rect;

    Image::new(include_image!(icons::lucide::icon_chevron_down!()))
        .rotate(
            egui::lerp(0.0..=-180f32.to_radians(), openness),
            Vec2::splat(0.5),
        )
        .paint_at(ui, rect.scale_from_center(2.0));
}

fn paint_empty_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response) {
    let visuals = ui.style().interact(response);
    let stroke = visuals.fg_stroke;

    let rect = response.rect;

    ui.painter()
        .rect(rect, Rounding::ZERO, Color32::TRANSPARENT, Stroke::NONE);
}
