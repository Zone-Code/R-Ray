use bevy::prelude::{Changed, Commands, Entity, GlobalTransform, Query, ResMut, Transform, With, World};
use bevy_inspector_egui::bevy_inspector::hierarchy::{SelectedEntities, SelectionMode};
use bevy_render::camera::Projection;
use transform_gizmo_bevy::GizmoTarget;
use crate::{GizmoMode, MainCamera, UiState};

pub fn draw_gizmo(
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
    query: Query<(Entity, Option<&mut GizmoTarget>)>,
) {

    let selected_entities = &mut ui_state.selected_entities;
    if let Some((action, entity)) = selected_entities.last_action() {
        match action {
            SelectionMode::Replace => {
                for (e, _) in query.iter() {
                    commands.entity(e).remove::<GizmoTarget>();
                }
                commands.entity(entity).insert(GizmoTarget::default());
            }
            _ => {}
        }
    }
}
