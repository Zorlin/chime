use bevy::prelude::*;
use bevy::input::gamepad::{GamepadButton, GamepadAxis};
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::window::CursorMoved;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::render::mesh::PrimitiveTopology;

#[derive(Component)]
struct RotatableCube;

#[derive(Component)]
struct VoidPlane;

#[derive(Resource)]
struct VoidState {
    instability_heat: f32,
    mouse_pos: Vec2,
}

#[derive(Resource)]
struct ScreenshotTimer {
    elapsed: f32,
    screenshot_5s_taken: bool,
    screenshot_10s_taken: bool,
}

impl Default for VoidState {
    fn default() -> Self {
        Self {
            instability_heat: 0.0,
            mouse_pos: Vec2::new(0.5, 0.5),
        }
    }
}

impl Default for ScreenshotTimer {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            screenshot_5s_taken: false,
            screenshot_10s_taken: false,
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct VoidMaterial {
    #[uniform(0)]
    time: f32,
    #[uniform(1)]
    mouse_pos: Vec2,
    #[uniform(2)]
    instability_heat: f32,
}

impl Material for VoidMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/void.wgsl".into()
    }
    
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
    
    // Make it unlit
    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MaterialPlugin::<VoidMaterial>::default(),
        ))
        .init_resource::<VoidState>()
        .init_resource::<ScreenshotTimer>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            gamepad_system,
            rotate_cube_system,
            update_void_material,
            cursor_moved_system,
            update_instability,
            screenshot_system,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut void_materials: ResMut<Assets<VoidMaterial>>,
    _asset_server: Res<AssetServer>,
) {
    // Create the void background - large inverted sphere
    let mut sphere_mesh = Sphere::new(100.0).mesh().build();
    // Flip the normals to make it render from inside
    if let Some(normals) = sphere_mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        if let bevy::render::mesh::VertexAttributeValues::Float32x3(normals) = normals {
            for normal in normals.iter_mut() {
                normal[0] = -normal[0];
                normal[1] = -normal[1];
                normal[2] = -normal[2];
            }
        }
    }
    
    commands.spawn((
        Mesh3d(meshes.add(sphere_mesh)),
        MeshMaterial3d(void_materials.add(VoidMaterial {
            time: 0.0,
            mouse_pos: Vec2::new(0.5, 0.5),
            instability_heat: 0.0,
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        VoidPlane,
    ));
    
    // Spawn the cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.8),
            emissive: LinearRgba::rgb(0.2, 0.05, 0.2),
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
        RotatableCube,
    ));

    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Camera {
            hdr: true,
            ..default()
        },
    ));

    // Spawn minimal lighting (dimmer for void atmosphere)
    commands.spawn((
        DirectionalLight {
            illuminance: 2000.0,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 1000.0,
            ..default()
        },
        Transform::from_xyz(-5.0, 5.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn update_void_material(
    mut void_materials: ResMut<Assets<VoidMaterial>>,
    void_query: Query<&MeshMaterial3d<VoidMaterial>, With<VoidPlane>>,
    time: Res<Time>,
    void_state: Res<VoidState>,
) {
    for material_handle in void_query.iter() {
        if let Some(material) = void_materials.get_mut(&material_handle.0) {
            material.time = time.elapsed_secs();
            material.mouse_pos = void_state.mouse_pos;
            material.instability_heat = void_state.instability_heat;
        }
    }
}

fn cursor_moved_system(
    mut events: EventReader<CursorMoved>,
    windows: Query<&Window>,
    mut void_state: ResMut<VoidState>,
) {
    for event in events.read() {
        if let Ok(window) = windows.get(event.window) {
            // Normalize cursor position to 0-1 range
            void_state.mouse_pos = Vec2::new(
                event.position.x / window.width(),
                1.0 - (event.position.y / window.height()),
            );
        }
    }
}

fn update_instability(
    mut void_state: ResMut<VoidState>,
    time: Res<Time>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    // Slowly increase instability over time
    void_state.instability_heat += time.delta_secs() * 0.01;
    
    // Click to add instability
    if mouse_buttons.just_pressed(MouseButton::Left) {
        void_state.instability_heat += 0.1;
    }
    
    // Clamp instability
    void_state.instability_heat = void_state.instability_heat.clamp(0.0, 1.0);
}

fn gamepad_system(gamepads: Query<(Entity, &Gamepad)>) {
    for (entity, gamepad) in &gamepads {
        // Check button presses
        if gamepad.just_pressed(GamepadButton::South) {
            println!("{:?} just pressed South (A/X)", entity);
        }
        if gamepad.just_pressed(GamepadButton::East) {
            println!("{:?} just pressed East (B/Circle)", entity);
        }
        if gamepad.just_pressed(GamepadButton::North) {
            println!("{:?} just pressed North (Y/Triangle)", entity);
        }
        if gamepad.just_pressed(GamepadButton::West) {
            println!("{:?} just pressed West (X/Square)", entity);
        }
    }
}

fn rotate_cube_system(
    time: Res<Time>,
    gamepads: Query<&Gamepad>,
    mut cubes: Query<&mut Transform, With<RotatableCube>>,
) {
    for gamepad in &gamepads {
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        let right_stick_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);

        for mut transform in &mut cubes {
            let rotation_speed = 2.0;
            let dt = time.delta_secs();
            
            let deadzone = 0.1;
            
            let mut rotation = Vec3::ZERO;
            
            if left_stick_x.abs() > deadzone {
                rotation.y += left_stick_x * rotation_speed * dt;
            }
            
            if left_stick_y.abs() > deadzone {
                rotation.x += -left_stick_y * rotation_speed * dt;
            }
            
            if right_stick_x.abs() > deadzone {
                rotation.z += right_stick_x * rotation_speed * dt;
            }
            
            transform.rotate_x(rotation.x);
            transform.rotate_y(rotation.y);
            transform.rotate_z(rotation.z);
        }
    }
}

fn screenshot_system(
    mut commands: Commands,
    mut screenshot_timer: ResMut<ScreenshotTimer>,
    time: Res<Time>,
) {
    screenshot_timer.elapsed += time.delta_secs();
    
    // Take screenshot at 5 seconds
    if !screenshot_timer.screenshot_5s_taken && screenshot_timer.elapsed >= 5.0 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("screenshots/chime_5s.png"));
        screenshot_timer.screenshot_5s_taken = true;
        println!("Screenshot taken at 5 seconds");
    }
    
    // Take screenshot at 10 seconds
    if !screenshot_timer.screenshot_10s_taken && screenshot_timer.elapsed >= 10.0 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk("screenshots/chime_10s.png"));
        screenshot_timer.screenshot_10s_taken = true;
        println!("Screenshot taken at 10 seconds");
    }
}