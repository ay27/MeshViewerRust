use bevy::prelude::*;
use bevy::mesh::VertexAttributeValues;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin, TrackpadBehavior};

use crate::plugins::model_loader::LoadedModel;
use crate::resources::UiAction;

/// Plugin that sets up the 3D camera with Blender-like orbit controls.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_systems(Startup, setup_camera)
            .add_systems(Update, (handle_fit_to_view, handle_reset_camera));
    }
}

/// Marker component for the main camera so we can query it later.
#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    // AmbientLight is attached to the Camera3d entity because in Bevy 0.18
    // AmbientLight #[require(Camera)].  Spawning it separately would create a
    // bare Camera that bevy_egui mistakes for the primary context target.
    // Scene units are centimeters (1 unit = 1cm), so default camera
    // distance and zoom must be scaled accordingly.
    //
    // The three DirectionalLights below are spawned as CHILDREN of the camera
    // entity. Because Bevy composes GlobalTransform = parent_global * local,
    // the lights inherit the camera's rotation automatically — i.e. they live
    // in the camera's local frame, which is exactly how Blender's Solid mode
    // Studio Lighting works (view-space anchored, not world-space). When you
    // orbit the camera, the lit side of the model follows you, so shape stays
    // legible from every angle.
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(500.0, 300.0, 500.0).looking_at(Vec3::ZERO, Vec3::Y),
            PanOrbitCamera {
                focus: Vec3::ZERO,
                radius: Some(800.0),
                // Left-click drag = orbit
                button_orbit: MouseButton::Left,
                // Shift + left-click drag = pan
                button_pan: MouseButton::Left,
                modifier_pan: Some(KeyCode::ShiftLeft),
                // Scroll wheel = zoom (scaled for cm units)
                zoom_sensitivity: 1.0,
                // Allow full 360-degree rotation (no pitch clamping)
                allow_upside_down: true,
                // Trackpad: Default mode — all scroll events = zoom
                trackpad_behavior: TrackpadBehavior::Default,
                trackpad_sensitivity: 1.0,
                trackpad_pinch_to_zoom_enabled: true,
                // Smooth movement
                orbit_smoothness: 0.2,
                pan_smoothness: 0.2,
                zoom_smoothness: 0.2,
                ..default()
            },
            // Gentle ambient — keeps crevices that face away from all three
            // child lights from going pitch black.
            AmbientLight {
                color: Color::WHITE,
                brightness: 100.0,
                affects_lightmapped_meshes: true,
            },
            MainCamera,
        ))
        .with_children(|parent| {
            // === Camera-space Studio lights (Blender Solid-style) ===
            //
            // DirectionalLight shines along its entity's local -Z. The camera
            // itself looks down its local -Z, so a child light with identity
            // rotation would be a pure headlight (light goes the way camera
            // looks). We offset each light's rotation to position it relative
            // to the camera frame:
            //   - Key:  upper-right-front  → -Z tilted down-left
            //   - Fill: upper-left-back    → -Z tilted up-right-back
            //   - Rim:  lower-back         → -Z tilted up-back

            // Key — warm, dominant, from upper-right of viewer
            parent.spawn((
                DirectionalLight {
                    illuminance: 4_500.0,
                    shadows_enabled: false,
                    color: Color::srgb(1.0, 0.98, 0.95),
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(
                    EulerRot::YXZ,
                    0.6,   // yaw: tilt light's forward to the LEFT in cam frame
                    -0.5,  // pitch: tilt DOWN so light comes from above
                    0.0,
                )),
            ));

            // Fill — cool, weaker, from upper-left of viewer
            parent.spawn((
                DirectionalLight {
                    illuminance: 2_800.0,
                    shadows_enabled: false,
                    color: Color::srgb(0.88, 0.92, 1.0),
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(
                    EulerRot::YXZ,
                    -0.8,  // yaw: tilt forward to the RIGHT in cam frame
                    -0.3,
                    0.0,
                )),
            ));

            // Rim — neutral, weakest, from behind viewer (front-lights model's
            // back side, creating a subtle edge separation from dark bg).
            parent.spawn((
                DirectionalLight {
                    illuminance: 1_500.0,
                    shadows_enabled: false,
                    color: Color::srgb(0.95, 0.95, 1.0),
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(
                    EulerRot::YXZ,
                    std::f32::consts::PI, // 180° — flip so it shines toward +Z (behind cam)
                    0.3,                  // slight pitch up
                    0.0,
                )),
            ));
        });
}

/// Handle `UiAction::FitToView` — reposition camera to see all loaded meshes.
fn handle_fit_to_view(
    mut ui_actions: MessageReader<UiAction>,
    mesh_handles: Query<(&Mesh3d, &GlobalTransform), With<LoadedModel>>,
    meshes: Res<Assets<Mesh>>,
    mut camera_query: Query<(&mut PanOrbitCamera, &mut Transform), With<MainCamera>>,
) {
    let mut should_fit = false;
    for action in ui_actions.read() {
        if matches!(action, UiAction::FitToView) {
            should_fit = true;
        }
    }

    if !should_fit {
        return;
    }

    // Calculate bounding box of all loaded model entities from mesh position data
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    let mut has_any = false;

    for (mesh_handle, global_transform) in mesh_handles.iter() {
        if let Some(mesh) = meshes.get(&mesh_handle.0) {
            if let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            {
                for pos in positions {
                    let world_pos =
                        global_transform.transform_point(Vec3::new(pos[0], pos[1], pos[2]));
                    min = min.min(world_pos);
                    max = max.max(world_pos);
                    has_any = true;
                }
            }
        }
    }

    if !has_any {
        return;
    }

    let center = (min + max) * 0.5;
    let size = max - min;
    let radius = size.length() * 0.8;
    let radius = radius.max(100.0); // min 100cm = 1m distance

    if let Ok((mut orbit, _transform)) = camera_query.single_mut() {
        orbit.target_focus = center;
        orbit.target_radius = radius;
        orbit.force_update = true;
    }
}

/// Handle `UiAction::ResetCamera` — return camera to default position.
fn handle_reset_camera(
    mut ui_actions: MessageReader<UiAction>,
    mut camera_query: Query<(&mut PanOrbitCamera, &mut Transform), With<MainCamera>>,
) {
    let mut should_reset = false;
    for action in ui_actions.read() {
        if matches!(action, UiAction::ResetCamera) {
            should_reset = true;
        }
    }

    if !should_reset {
        return;
    }

    if let Ok((mut orbit, _transform)) = camera_query.single_mut() {
        orbit.target_focus = Vec3::ZERO;
        orbit.target_radius = 800.0; // 800cm = 8m default distance
        orbit.target_yaw = std::f32::consts::FRAC_PI_4;
        orbit.target_pitch = std::f32::consts::FRAC_PI_6;
        orbit.force_update = true;
    }
}
