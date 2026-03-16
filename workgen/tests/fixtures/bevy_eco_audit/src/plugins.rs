use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

fn play_sound(audio: Res<Audio>, assets: Res<AssetServer>) {
    let handle = assets.load("sounds/hit.ogg");
    audio.play(handle);
}

fn spawn_particles(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    for _ in 0..100 {
        let x = rng.gen_range(-10.0..10.0);
        let y = rng.gen_range(-10.0..10.0);
        commands.spawn(SpriteBundle {
            transform: Transform::from_xyz(x, y, 0.0),
            ..default()
        });
    }
}

fn apply_physics(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.linvel.x;
        transform.translation.y += velocity.linvel.y;
    }
}

fn setup_ui(mut contexts: EguiContexts) {
    egui::Window::new("Debug").show(contexts.ctx_mut(), |ui| {
        ui.label("FPS: 60");
    });
}
