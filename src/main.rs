use bevy::{prelude::*, window::PrimaryWindow,};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::{thread_rng, Rng};
// use rand::{thread_rng, Rng};

#[derive(Component)]
#[derive(Default)]
#[derive(Reflect)]
// #[reflect(Component)]
pub struct Tile { 
    x: u8,
    y: u8, 
}

#[derive(Component)]
pub enum Type{
    Empty,
    Bomb,
    Number,
}

fn main() {
    App::new()
        .register_type::<Tile>()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_tiles)

        .add_startup_system(set_bomb_positions)
        .run();
}

fn spawn_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window: &Window = window.get_single().unwrap();
    let mut x = 0;
    

    for i in 0..5 {
        for j in 0..5 {
            x += 1;
            commands.spawn((
                Tile{
                    x: j,
                    y: i,
                },
                SpriteBundle{
                    texture: asset_server.load("sprites/tile_five.png"),
                    transform: Transform::from_xyz(
                        window.width() * 40.0 / 100.0 + j as f32 * 64.0,
                        window.height() * 75.0 / 100.0 - i as f32 * 64.0, 
                        0.0,
                    ),
                    ..default()
                },
                Name::new("Tile".to_string() + &x.to_string())
            ));

            // commands.entity(tile)
        }
    }
}


fn set_bomb_positions() {
    let mut rng = thread_rng();
    let mut cords: (u8, u8) = (0, 0);
    let mut selected: Vec<(u8, u8)> = Vec::new();
    let mut i = 4;

    'outer: while(i > 0){
        cords = (rng.gen_range(1..6), rng.gen_range(1..6));
        println!("{:?}", selected);
        for x in selected.iter() {
            if cords == *x {
                println!("{:?}", cords);
                continue 'outer;
            }
        }
        i -= 1;
        selected.push(cords);
    }
    println!("Final: {:?}", selected);
    
}


fn set_bomb_sprites(
    mut tiles: Query<&Handle<Image>, With<Tile>>
) {
    for tile in tiles.iter(){
        //if bomb change sprite
    }
}

pub fn spawn_camera(
    mut commands: Commands, 
    window_query: Query<&Window, With<PrimaryWindow>>
) {
    let window: &Window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}
