use bevy::{
    prelude::*,
    picking::pointer::{PointerInteraction, PointerPress},
};
use transform_gizmo_bevy::GizmoTarget;

/// Інтеграція picking з гізмо.
pub struct GizmoPickingPlugin;

impl Plugin for GizmoPickingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MeshPickingPlugin)
            .add_systems(PreUpdate, toggle_picking_enabled)
            .add_systems(Update, manage_selection);
    }
}

/// Вимикає picking, коли будь-який з гізмо (GizmoTarget) знаходиться у фокусі або активний.
fn toggle_picking_enabled(
    gizmo_targets: Query<&GizmoTarget>,
    mut picking_settings: ResMut<PickingPlugin>,
) {
    picking_settings.is_enabled = gizmo_targets
        .iter()
        .all(|target| !target.is_focused() && !target.is_active());
}

/// Обробляє логіку вибору/зніття вибору об’єктів через мишку.
///
/// Якщо ліву кнопку відпустили, то:
/// - Якщо не натиснуто Shift, знімаємо вибір з усіх об’єктів;
/// - Далі для клікаємого об’єкта перевіряємо, чи має він вже `GizmoTarget`.
///   Якщо так – знімаємо вибір (видаляємо `GizmoTarget`), інакше – додаємо його.
pub fn manage_selection(
    pointers: Query<&PointerInteraction, Changed<PointerPress>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    // Запит по всіх об'єктах, на яких встановлено GizmoTarget (тобто вибраних)
    gizmo_query: Query<Entity, With<GizmoTarget>>,
    mut commands: Commands,
) {
    if !mouse.just_released(MouseButton::Left) {
        return;
    }

    // Отримуємо єдиний джерело pointer (демо припускає, що його лише один)
    let pointer = match pointers.get_single() {
        Ok(pointer) => pointer,
        Err(err) => match err {
            bevy::ecs::query::QuerySingleError::NoEntities(_) => return,
            bevy::ecs::query::QuerySingleError::MultipleEntities(_) => {
                warn!("Демо працює лише з одним джерелом pointer. Видаліть зайві!");
                return;
            }
        },
    };

    if let Some((entity, _)) = pointer.first() {
        // Заздалегідь перевіряємо, чи був об’єкт вибраним
        let was_selected = gizmo_query.get(*entity).is_ok();

        // Якщо Shift не натиснуто, знімаємо вибір з усіх об’єктів
        if !keys.pressed(KeyCode::ShiftLeft) {
            for selected_entity in gizmo_query.iter() {
                commands.entity(selected_entity).remove::<GizmoTarget>();
            }
        }

        // Перемикаємо стан вибору для клікаємого об’єкта
        if was_selected {
            commands.entity(*entity).remove::<GizmoTarget>();
        } else {
            commands.entity(*entity).insert(GizmoTarget::default());
        }
    }
}
