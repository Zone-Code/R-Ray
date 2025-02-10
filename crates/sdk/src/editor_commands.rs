use bevy::ecs::system::SystemState;
use bevy::input::ButtonInput;
use bevy::prelude::{Commands, Component, Entity, KeyCode, Res, ResMut, Resource, Transform, World};
use crate::ui::{EguiWindow, UiState};

#[derive(Resource)]
pub struct HistoryManager {
    undo_stack: Vec<Box<dyn EditorCommand>>,
    redo_stack: Vec<Box<dyn EditorCommand>>,
}

impl HistoryManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, mut command: Box<dyn EditorCommand>, commands: &mut Commands) {
        command.execute(commands);
        self.undo_stack.push(command);
        self.redo_stack.clear(); // Очистка redo після нової дії
    }

    pub fn undo(&mut self, commands: &mut Commands) {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo(commands);
            self.redo_stack.push(command);
        }
    }

    pub fn redo(&mut self, commands: &mut Commands) {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute(commands);
            self.undo_stack.push(command);
        }
    }
}

pub trait EditorCommand: Send + Sync {
    fn execute(&mut self, commands: &mut Commands);
    fn undo(&mut self, commands: &mut Commands);
}

pub fn handle_input(
    world: &mut World,
) {
    
    let mut state: SystemState<(
        Res<ButtonInput<KeyCode>>,
        ResMut<HistoryManager>,
        Commands,
        ResMut<UiState>
    )> = SystemState::new(world);

    let (
        keyboard_input, 
        mut history, 
        mut commands,
        mut ui_state
    ) = state.get_mut(world);

    if let Some((_, window)) = ui_state.state.find_active_focused() {
        match window {
            EguiWindow::GameView => {}
            _ => {return;}
        }
    }

    if !keyboard_input.pressed(KeyCode::KeyZ) {
        return;
    }
    if keyboard_input.pressed(KeyCode::KeyZ) {
        history.undo(&mut commands);
    } else if keyboard_input.pressed(KeyCode::KeyY) {
        history.redo(&mut commands);
    }

    state.apply(world);
}

struct TransformChange {
    entity: Entity,
    from: Transform,
    to: Transform
}
impl EditorCommand for TransformChange {
    fn execute(&mut self, commands: &mut Commands) {
        /*if let Some(mut transform) = world.get_mut::<Transform>(self.entity) {
            *transform = self.to;
        }*/
        commands.entity(self.entity).remove::<Transform>();
        commands.entity(self.entity).insert(self.to);
    }

    fn undo(&mut self, commands: &mut Commands) {
        commands.entity(self.entity).remove::<Transform>();
        commands.entity(self.entity).insert(self.from);
    }
}