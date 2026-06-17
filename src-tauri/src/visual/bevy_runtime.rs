use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};

use crate::visual::protocol::{
    AppToVisualMessage, AppToVisualPayload, VisualParameterBatchApplied, VisualProtocolError,
    VisualProtocolErrorCode, VisualReady, VisualRuntimeEvent, VisualRuntimeEventLevel,
    VisualSceneLoaded, VisualSceneSnapshot, VisualShutdownComplete, VisualToAppMessage,
    VisualToAppPayload, VISUAL_PROTOCOL_VERSION,
};
use crate::visual::sidecar::error_response;

pub const BEVY_RENDERER: &str = "scrysynth-bevy-visual";

#[derive(Resource)]
struct ProtocolInbox {
    receiver: Mutex<Receiver<Result<AppToVisualMessage, String>>>,
}

#[derive(Resource)]
struct ProtocolOutbox {
    sender: Sender<VisualToAppMessage>,
}

#[derive(Resource, Default)]
struct RenderedScene {
    scene: Option<VisualSceneSnapshot>,
    parameters: HashMap<(String, String), f64>,
}

#[derive(Component)]
struct VisualElementEntity {
    element_id: String,
}

pub fn run_visible_runtime() {
    let (inbox_sender, inbox_receiver) = mpsc::channel();
    let (outbox_sender, outbox_receiver) = mpsc::channel();

    spawn_stdin_reader(inbox_sender);
    spawn_stdout_writer(outbox_receiver);

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(ProtocolInbox {
            receiver: Mutex::new(inbox_receiver),
        })
        .insert_resource(ProtocolOutbox {
            sender: outbox_sender,
        })
        .insert_resource(RenderedScene::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Scrysynth Visual Runtime".to_string(),
                resolution: WindowResolution::new(960, 540),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, handle_protocol_messages)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn handle_protocol_messages(
    mut commands: Commands,
    inbox: Res<ProtocolInbox>,
    outbox: Res<ProtocolOutbox>,
    mut rendered_scene: ResMut<RenderedScene>,
    mut clear_color: ResMut<ClearColor>,
    mut query: Query<(Entity, &VisualElementEntity, &mut Transform, &mut Sprite)>,
) {
    let Ok(receiver) = inbox.receiver.lock() else {
        return;
    };

    while let Ok(message) = receiver.try_recv() {
        match message {
            Ok(message) => handle_protocol_message(
                message,
                &outbox,
                &mut commands,
                &mut rendered_scene,
                &mut clear_color,
                &mut query,
            ),
            Err(message) => send_message(&outbox, error_response(None, message)),
        }
    }
}

fn handle_protocol_message(
    message: AppToVisualMessage,
    outbox: &ProtocolOutbox,
    commands: &mut Commands,
    rendered_scene: &mut RenderedScene,
    clear_color: &mut ClearColor,
    query: &mut Query<(Entity, &VisualElementEntity, &mut Transform, &mut Sprite)>,
) {
    if message.protocol_version != VISUAL_PROTOCOL_VERSION {
        send_message(
            outbox,
            VisualToAppMessage::response(
                message.sequence_id,
                VisualToAppPayload::Error(VisualProtocolError {
                    code: VisualProtocolErrorCode::ProtocolMismatch,
                    message: format!(
                        "unsupported visual protocol version {}; expected {}",
                        message.protocol_version, VISUAL_PROTOCOL_VERSION
                    ),
                    recoverable: Some(false),
                }),
            ),
        );
        return;
    }

    match message.payload {
        AppToVisualPayload::Handshake(_) => {
            send_message(
                outbox,
                VisualToAppMessage::response(
                    message.sequence_id,
                    VisualToAppPayload::Ready(VisualReady {
                        renderer: BEVY_RENDERER.to_string(),
                        sidecar_version: env!("CARGO_PKG_VERSION").to_string(),
                        capabilities: vec![
                            "scene_load".to_string(),
                            "parameter_batch".to_string(),
                            "rendering_status".to_string(),
                            "shutdown".to_string(),
                        ],
                    }),
                ),
            );
            send_message(
                outbox,
                VisualToAppMessage::event(VisualToAppPayload::RuntimeEvent(VisualRuntimeEvent {
                    level: VisualRuntimeEventLevel::Info,
                    message: "bevy visual runtime ready and rendering window active".to_string(),
                    scene_id: rendered_scene
                        .scene
                        .as_ref()
                        .map(|scene| scene.scene_id.clone()),
                })),
            );
        }
        AppToVisualPayload::LoadScene(load) => {
            load_scene(
                commands,
                rendered_scene,
                clear_color,
                query,
                load.scene.clone(),
            );
            send_message(
                outbox,
                VisualToAppMessage::response(
                    message.sequence_id,
                    VisualToAppPayload::SceneLoaded(VisualSceneLoaded {
                        scene_id: load.scene.scene_id,
                        rendering: true,
                    }),
                ),
            );
        }
        AppToVisualPayload::UpdateParameters(batch) => {
            let mut applied_count = 0;
            for update in batch.updates {
                if rendered_scene
                    .parameters
                    .contains_key(&(update.element_id.clone(), update.parameter_id.clone()))
                {
                    rendered_scene.parameters.insert(
                        (update.element_id.clone(), update.parameter_id.clone()),
                        update.value,
                    );
                    applied_count += 1;

                    for (_, element, mut transform, mut sprite) in query.iter_mut() {
                        if element.element_id == update.element_id {
                            apply_parameter_to_entity(
                                &update.parameter_id,
                                update.value,
                                &mut transform,
                                &mut sprite,
                            );
                        }
                    }
                }
            }

            send_message(
                outbox,
                VisualToAppMessage::response(
                    message.sequence_id,
                    VisualToAppPayload::ParameterBatchApplied(VisualParameterBatchApplied {
                        applied_count,
                    }),
                ),
            );
        }
        AppToVisualPayload::Ping(ping) => send_message(
            outbox,
            VisualToAppMessage::response(
                message.sequence_id,
                VisualToAppPayload::Pong(crate::visual::protocol::VisualPong {
                    sent_at_unix_ms: ping.sent_at_unix_ms,
                }),
            ),
        ),
        AppToVisualPayload::Shutdown(shutdown) => {
            for (entity, _, _, _) in query.iter_mut() {
                commands.entity(entity).despawn();
            }
            rendered_scene.scene = None;
            rendered_scene.parameters.clear();
            send_message(
                outbox,
                VisualToAppMessage::response(
                    message.sequence_id,
                    VisualToAppPayload::ShutdownComplete(VisualShutdownComplete {
                        mode: shutdown.mode,
                    }),
                ),
            );
            std::process::exit(0);
        }
    }
}

fn load_scene(
    commands: &mut Commands,
    rendered_scene: &mut RenderedScene,
    clear_color: &mut ClearColor,
    query: &mut Query<(Entity, &VisualElementEntity, &mut Transform, &mut Sprite)>,
    scene: VisualSceneSnapshot,
) {
    for (entity, _, _, _) in query.iter_mut() {
        commands.entity(entity).despawn();
    }

    rendered_scene.parameters.clear();
    clear_color.0 = Color::srgba(
        scene.background_color[0],
        scene.background_color[1],
        scene.background_color[2],
        scene.background_color[3],
    );

    for (index, element) in scene.elements.iter().enumerate() {
        for parameter in &element.parameters {
            rendered_scene.parameters.insert(
                (element.element_id.clone(), parameter.parameter_id.clone()),
                parameter.value,
            );
        }

        let value = element.parameters.first().map(|p| p.value).unwrap_or(0.5);
        let size = visual_size(element.scale, value);
        let position = visual_position(element.position, index, scene.elements.len());
        let color = visual_color(&element.element_type, value);

        commands.spawn((
            Sprite::from_color(color, Vec2::splat(size)),
            Transform::from_xyz(position.x, position.y, index as f32),
            VisualElementEntity {
                element_id: element.element_id.clone(),
            },
        ));
    }

    rendered_scene.scene = Some(scene);
}

fn apply_parameter_to_entity(
    parameter_id: &str,
    value: f64,
    transform: &mut Transform,
    sprite: &mut Sprite,
) {
    let normalized = value.clamp(0.0, 1.0) as f32;
    match parameter_id {
        "gain" | "level" | "energy" | "amount" => {
            transform.scale = Vec3::splat(0.65 + normalized * 1.75);
            sprite.color.set_alpha(0.45 + normalized * 0.55);
        }
        "frequency" | "cutoff" | "rate" => {
            sprite.color = Color::srgb(0.25 + normalized * 0.7, 0.85, 1.0 - normalized * 0.45);
        }
        _ => {
            sprite.color.set_alpha(0.35 + normalized * 0.65);
        }
    }
}

fn visual_position(position: [f32; 2], index: usize, total: usize) -> Vec2 {
    if position != [0.0, 0.0] {
        return Vec2::new(position[0] * 180.0 - 360.0, 180.0 - position[1] * 120.0);
    }

    let count = total.max(1) as f32;
    let angle = index as f32 / count * std::f32::consts::TAU;
    Vec2::new(angle.cos() * 260.0, angle.sin() * 145.0)
}

fn visual_size(scale: f32, value: f64) -> f32 {
    let normalized = value.clamp(0.0, 1.0) as f32;
    (56.0 + normalized * 72.0) * scale.max(0.35)
}

fn visual_color(element_type: &str, value: f64) -> Color {
    let normalized = value.clamp(0.0, 1.0) as f32;
    match element_type {
        "sphere" => Color::srgb(0.3 + normalized * 0.45, 0.95, 0.85),
        "box" => Color::srgb(0.95, 0.55 + normalized * 0.3, 0.3),
        "ring" => Color::srgb(0.8, 0.65, 1.0),
        "plane" => Color::srgb(0.35, 0.55 + normalized * 0.35, 1.0),
        _ => Color::srgb(0.85, 0.9, 0.95),
    }
}

fn spawn_stdin_reader(sender: Sender<Result<AppToVisualMessage, String>>) {
    thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            let line = match line {
                Ok(line) => line,
                Err(err) => {
                    let _ = sender.send(Err(err.to_string()));
                    continue;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            let message =
                serde_json::from_str::<AppToVisualMessage>(&line).map_err(|err| err.to_string());
            if sender.send(message).is_err() {
                break;
            }
        }
    });
}

fn spawn_stdout_writer(receiver: Receiver<VisualToAppMessage>) {
    thread::spawn(move || {
        let mut stdout = io::stdout();
        while let Ok(message) = receiver.recv() {
            if serde_json::to_writer(&mut stdout, &message).is_ok() {
                let _ = stdout.write_all(b"\n");
                let _ = stdout.flush();
            }
        }
    });
}

fn send_message(outbox: &ProtocolOutbox, message: VisualToAppMessage) {
    let _ = outbox.sender.send(message);
}
