use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            move_player,
            check_collisions,
            update_score,
            spawn_enemies,
            despawn_dead,
            update_health_bar,
            sync_transforms,
            handle_input,
            play_sounds,
            update_ui_text,
            animate_sprites,
            check_boundaries,
            apply_gravity,
            update_camera,
            process_powerups,
            handle_pause,
        ).chain())
        .add_systems(Update, render_debug.run_if(in_state(GameState::Playing)))
        .add_systems(Update, update_menu.run_if(in_state(GameState::Playing)))
        .add_systems(Update, save_progress.run_if(in_state(GameState::Playing)))
        .insert_resource(GameConfig::default());
    }
}

#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct Speed(pub f32);

#[derive(Resource, Default)]
struct GameConfig {
    difficulty: f32,
    volume: f32,
}

fn move_player(query: Query<(&Speed, &mut Transform)>, time: Res<Time>) {}
fn check_collisions() {}
fn update_score() {}
fn spawn_enemies() {}
fn despawn_dead() {}
fn update_health_bar() {}
fn sync_transforms() {}
fn handle_input() {}
fn play_sounds() {}
fn update_ui_text() {}
fn animate_sprites() {}
fn check_boundaries() {}
fn apply_gravity() {}
fn update_camera() {}
fn process_powerups() {}
fn handle_pause() {}
fn render_debug() {}
fn update_menu() {}
fn save_progress() {}
