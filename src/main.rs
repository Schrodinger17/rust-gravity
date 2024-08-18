//! Renders a 2D scene containing a single, moving sprite.

use bevy::{
    math::NormedVectorSpace,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use rand::random;

const UNIVERSE_WIDTH: f32 = 200.; // meters
const UNIVERSE_HEIGHT: f32 = 200.; // meters
const WINDOW_WIDTH: f32 = 800.; // pixels
const WINDOW_HEIGHT: f32 = 400.; // pixels
const SCALE: f32 = 2.; // ratio pixels/meter
const G: f32 = -9.81;
const FRICTION: f32 = 0.5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(GameState::Paused)
        .insert_resource(TimeProgress::default())
        .add_systems(Startup, setup)
        .add_systems(Update, time_progress)
        .add_systems(
            Update,
            update_balls
                .run_if(resource_exists::<GameState>.and_then(resource_equals(GameState::Running))),
        )
        .run();
}

#[derive(Resource, Debug, Clone, PartialEq)]
enum GameState {
    Running,
    Paused,
}

#[derive(Resource, Debug, Default, Clone)]
struct TimeProgress {
    time: f32,
    frame: u32,
    frame_forward: u32,
}

#[derive(Component, Debug, Clone)]
struct Ball {
    position: Vec3,
    speed: Vec3,
    acceleration: Vec3,
    mass: f32,
    size: f32,
    fixed: bool,
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            speed: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            mass: 1.,
            size: 10.,
            fixed: false,
        }
    }
}

impl Ball {
    pub fn new(position: Vec3, speed: Vec3, acceleration: Vec3, mass: f32, size: f32) -> Self {
        Self {
            position,
            speed,
            acceleration,
            mass,
            size,
            ..Default::default()
        }
    }
}

#[derive(Bundle, Default)]
struct BallBundle {
    ball: Ball,
    mesh: MaterialMesh2dBundle<ColorMaterial>,
}

impl BallBundle {
    pub fn new(
        position: Vec3,
        speed: Vec3,
        acceleration: Vec3,
        mass: f32,
        size: f32,
        materials: &mut ResMut<Assets<ColorMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) -> Self {
        Self {
            ball: Ball::new(position, speed, acceleration, mass, size),
            mesh: MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle { radius: size })),
                material: materials.add(Color::linear_rgb(0., 255., 0.)),
                transform: Transform::from_xyz(position.x * SCALE, position.y * SCALE, 0.0),
                ..Default::default()
            },
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    for _ in 0..10 {
        let random_position = Vec3::new(
            (random::<f32>() - 0.5) * UNIVERSE_WIDTH,
            (random::<f32>() - 0.5) * UNIVERSE_HEIGHT,
            0.,
        );

        let random_speed = Vec3::new(
            (random::<f32>() - 0.5) * 1.,
            (random::<f32>() - 0.5) * 1.,
            0.,
        );

        let mass = (random::<f32>()) * 1.5 + 0.5;

        commands.spawn(BallBundle::new(
            random_position,
            random_speed,
            Vec3::ZERO,
            mass,
            20.,
            &mut materials,
            &mut meshes,
        ));
    }
}

fn time_progress(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut time_progress: ResMut<TimeProgress>,
    time: Res<Time>,
) {
    if keys.just_pressed(KeyCode::Space) {
        match *game_state {
            GameState::Running => {
                println!("Pause");
                *game_state = GameState::Paused;
            }
            GameState::Paused => {
                println!("Unpause");
                *game_state = GameState::Running;
            }
        }
    }

    // Update time before pause
    if let GameState::Running = *game_state {
        if time_progress.frame_forward > 0 {
            time_progress.frame_forward -= 1;
            if time_progress.frame_forward == 0 {
                *game_state = GameState::Paused;
            }
        }
    }

    if keys.just_pressed(KeyCode::KeyF) {
        time_progress.frame_forward += 1;
        *game_state = GameState::Running;
    }

    if keys.just_pressed(KeyCode::ArrowRight) {
        if let GameState::Paused = *game_state {
            println!("Step forward");
            *game_state = GameState::Running;
            time_progress.frame_forward = 60;
        }
    }

    // Update time
    if let GameState::Running = *game_state {
        time_progress.time += time.delta_seconds();
        time_progress.frame += 1;
        println!(
            "Time: {:.2} s, Frame: {}",
            time_progress.time, time_progress.frame
        );
    }
}

fn update_balls(
    mut commands: Commands,
    time: Res<Time>,
    mut balls: Query<(Entity, &mut Ball, &mut Transform)>,
) {
    let other_balls = balls
        .iter()
        .map(|(_, ball, _)| ball.clone())
        .collect::<Vec<_>>();

    for (entity_id, mut ball, mut transform) in &mut balls {
        // dbg!(&ball.position);
        // dbg!(&ball.speed);
        // dbg!(&ball.acceleration);
        // dbg!(&transform.translation);

        if ball.fixed {
            continue;
        }

        // If ball is too slow and too low, stop it
        if ball.speed.norm() < 1. && ball.position.y - ball.size / 2. < -WINDOW_HEIGHT / 2. + 1.0 {
            ball.fixed = true;
            ball.speed = Vec3::ZERO;
        }

        let mut acceleration = ball.acceleration;
        // Forces
        let weight = Vec3::new(0., G, 0.) * ball.mass;

        let friction = ball.speed * -1. * FRICTION;

        acceleration += weight;
        acceleration += friction;

        // Attraction
        for other_ball in other_balls.iter() {
            if ball.position == other_ball.position {
                continue;
            }
            let distance = ball.position.distance(other_ball.position) / SCALE;
            let normal = (other_ball.position - ball.position).normalize();
            let force = normal * (ball.mass * other_ball.mass / distance.powi(2));
            acceleration += force / ball.mass;
        }

        ball.speed += acceleration * time.delta_seconds();

        let speed = ball.speed;
        ball.position += speed * time.delta_seconds();

        /*
        // Balls collision check
        for other_ball in other_balls.iter() {
            if ball.position == other_ball.position {
                continue;
            }
            let distance = ball.position.distance(other_ball.position);
            if distance < ball.size + other_ball.size {
                let normal = (other_ball.position - ball.position).normalize();
                let relative_speed = ball.speed - other_ball.speed;
                let impulse =
                    2. * relative_speed.dot(normal) / (ball.mass + other_ball.mass) * normal;
                ball.speed -= impulse * other_ball.mass;

                let size = ball.size;
                ball.position -= normal * (size + other_ball.size - distance) / 2.;
            }
        }
        */

        // Update transform
        transform.translation = ball.position * SCALE;

        // If outside off the universe, destroy the ball
        if transform.translation.x - ball.size / 2. > UNIVERSE_WIDTH / 2.
            || transform.translation.x + ball.size / 2. < -UNIVERSE_WIDTH / 2.
            || transform.translation.y - ball.size / 2. > UNIVERSE_HEIGHT / 2.
            || transform.translation.y + ball.size / 2. < -UNIVERSE_HEIGHT / 2.
        {
            //dbg!("Despawn {:?}", entity_id);
            commands.entity(entity_id).despawn();
            continue;
        }

        // Bounding off the walls check (last)
        if transform.translation.x - ball.size / 2. < -WINDOW_WIDTH / 2. && ball.speed.x < 0. {
            ball.speed.x = -ball.speed.x;
            ball.position.x = -WINDOW_WIDTH / 2. + ball.size / 2.;
        } else if transform.translation.x + ball.size / 2. > WINDOW_WIDTH / 2. && ball.speed.x > 0.
        {
            ball.speed.x = -ball.speed.x;
            ball.position.x = WINDOW_WIDTH / 2. - ball.size / 2.;
        }
        if transform.translation.y - ball.size / 2. < -WINDOW_HEIGHT / 2. && ball.speed.y < 0. {
            ball.speed.y = -ball.speed.y;
            ball.position.y = -WINDOW_HEIGHT / 2. + ball.size / 2.;
        } else if transform.translation.y + ball.size / 2. > WINDOW_HEIGHT / 2. && ball.speed.y > 0.
        {
            ball.speed.y = -ball.speed.y;
            ball.position.y = WINDOW_HEIGHT / 2. - ball.size / 2.;
        }

        // Update transform
        transform.translation = ball.position * SCALE;
    }
}
