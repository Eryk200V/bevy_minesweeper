#![windows_subsystem = "windows"]

//cargo build --target=x86_64-pc-windows-gnu --release

use bevy::{prelude::*, window::{PrimaryWindow, WindowResolution}, transform::commands};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::{thread_rng, Rng};
use bevy_despawn_with::DespawnAllCommandsExt;
use bevy_asset_loader::prelude::*;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum GameState {
    #[default]
    AssetLoading,
    SafeClick,
    InGame,
    GameOver,
}

const CLICK_AREA_SIZE: f32 = 20.0;


// const EAZY_BOARD_SIZE: Vec2 = Vec2::new(10.0, 10.0);
// const EAZY_BOMB_COUNT: u8 = 10;

// const MEDIUM_BOARD_SIZE: Vec2 = Vec2::new(12.0, 12.0);
// const MEDIUM_BOMB_COUNT: u8 = 26;

const HARD_BOARD_SIZE: Vec2 = Vec2::new(15.0, 15.0);
const HARD_BOMB_COUNT: u8 = 40;

const EXPERT_BOARD_SIZE: Vec2 = Vec2::new(16.0, 30.0);
const EXPERT_BOMB_COUNT: u8 = 99;

#[derive(Resource)]
struct Empty { cords: Vec<(u8, u8)> }

#[derive(Resource)]
struct Reset { position: (f32, f32) }

#[derive(Resource)]
struct Safe { cords: (u8, u8) }


#[derive(AssetCollection, Resource)]
struct TileSprites {

    #[asset(path = "sprites/tile_unknown2.png")]
    unknown: Handle<Image>,

    #[asset(path = "sprites/reset2.png")]
    reset: Handle<Image>,

    #[asset(path = "sprites/tile_exploded2.png")]
    exploded: Handle<Image>,

    #[asset(path = "sprites/tile_flag2.png")]
    flag: Handle<Image>,

    #[asset(path = "sprites/tile_bomb2.png")]
    bomb: Handle<Image>,

    #[asset(path = "sprites/tile_empty2.png")]
    zero: Handle<Image>,

    #[asset(path = "sprites/tile_one2.png")]
    one: Handle<Image>,
    
    #[asset(path = "sprites/tile_two2.png")]
    two: Handle<Image>,
    
    #[asset(path = "sprites/tile_three2.png")]
    three: Handle<Image>,
    
    #[asset(path = "sprites/tile_four2.png")]
    four: Handle<Image>,
    
    #[asset(path = "sprites/tile_five2.png")]
    five: Handle<Image>,
    
    #[asset(path = "sprites/tile_six2.png")]
    six: Handle<Image>,
    
    #[asset(path = "sprites/tile_seven2.png")]
    seven: Handle<Image>,
    
    #[asset(path = "sprites/tile_eight2.png")]
    eight: Handle<Image>,
    
    #[asset(path = "sprites/tile_nine2.png")]
    nine: Handle<Image>,
    
}

#[derive(Debug)]
#[derive(Resource)]
#[derive(Reflect)]
// #[reflect(Component)]
#[derive(Component)]
pub struct Tile { 
    x: u8,
    y: u8,
    tile_type: TileType,
    num: u8,
    covered: bool,
    flag: bool
}

#[derive(Component)]
#[derive(Reflect)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum TileType{
    Bomb,
    Other,
}

fn main() {
    App::new()
        .register_type::<Tile>()
        .register_type::<TileType>()
        .insert_resource(Empty{cords: vec!()})
        .insert_resource(Safe{cords: (0,0)})
        .insert_resource(Reset{position: (0.0, 0.0)})
        .insert_resource(ClearColor(Color::rgb_u8(164, 177, 197)))
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
            .continue_to_state(GameState::SafeClick)
        )
        .add_collection_to_loading_state::<_, TileSprites>(GameState::AssetLoading)
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(
                        Window{
                            title: "Minesweeper".to_string(),
                            resolution: WindowResolution::new(HARD_BOARD_SIZE.y * 19.0 * 2.0, HARD_BOARD_SIZE.x * 19.0 * 2.0 + 19.0 * 2.0),
                            fit_canvas_to_parent: true,
                            resizable: false,
                            ..default()
                        }),
                    ..default()
                })
        )
        //.add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(spawn_camera)
        .add_systems(
            (
            despawn_tiles,
            apply_system_buffers,
            spawn_tiles,
            apply_system_buffers,
            ).chain().in_schedule(OnEnter(GameState::SafeClick))
        )
        .add_systems(
            (
            click_switch,
            apply_system_buffers, 
            tile_check,
            apply_system_buffers,
            zero_check
            ).chain().in_set(OnUpdate(GameState::InGame))
        )
        .add_system(first_click.in_set(OnUpdate(GameState::SafeClick)))
        .add_systems(
            (
                set_bombs,
                apply_system_buffers
            ).chain().in_schedule(OnExit(GameState::SafeClick))
        )
        .add_system(game_over.in_schedule(OnEnter(GameState::GameOver)))
        .add_system(reset_click_check)
        .run();
}

pub fn spawn_camera(
    mut commands: Commands, 
    window_query: Query<&Window, With<PrimaryWindow>>
) {
    let window = window_query.get_single().unwrap();
    println!("Resolution {:?}", window.resolution);

    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
            projection: OrthographicProjection { 
                //scale: -0.2,
                ..default()
            },
            ..default()
        },
        Name::new("Main Camera")
    ));
}

fn despawn_tiles(mut commands: Commands) {
    commands.despawn_all::<With<Tile>>();
}

fn spawn_tiles(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut reset_button: ResMut<Reset>,
    tile_sprites: Res<TileSprites>
) {
    let window: &Window = window.get_single().unwrap();
    let mut x = 0;
    
    println!("{:?}", std::env::current_exe());
    for i in 0..HARD_BOARD_SIZE.x as u8 {
        for j in 0..HARD_BOARD_SIZE.y as u8 {
            x += 1;
            commands.spawn((
                Tile{
                    x: j + 1,
                    y: i + 1,
                    tile_type: TileType::Other,
                    num: 0,
                    covered: true,
                    flag: false
                },
                SpriteBundle{
                    texture: tile_sprites.unknown.clone(),
                    transform: Transform::from_xyz(
                        19.0 * 0.5 * 2.0 + j as f32 * 19.0 * 2.0,
                        window.height() - 19.0 * 1.5 - i as f32 * 19.0 * 2.0 + 9.5 - 19.0 * 2.0, 
                        0.0,
                    ).with_scale(Vec3::new(2.0, 2.0, 0.0)),
                    
                    ..default()
                },
                
                Name::new("Tile".to_string() + &x.to_string() + " (" + &(j+1).to_string() + ", " + &(i+1).to_string() + ")")
            ));
        }
    }
    commands.spawn(
    (
       SpriteBundle {
        texture: tile_sprites.reset.clone(),
        transform: Transform::from_xyz(
            19.0 * 0.5 * 2.0,
            window.height() - 19.0, 
            0.0,
        ).with_scale(Vec3::new(2.0, 2.0, 0.0)),
        
        ..default()
       }, 
       Name::new("Reset".to_string()),
    ));
    reset_button.position = (19.0 * 0.5 * 2.0, window.height() - 19.0);
    println!("Spawned!");
}


fn generate_bomb_positions(safe: (u8, u8)) -> Vec<(u8, u8)> {
    let mut rng = thread_rng();
    let mut selected: Vec<(u8, u8)> = Vec::new();
    let mut safe_zone: Vec<(u8, u8)> = Vec::new();
    let mut i = HARD_BOMB_COUNT;
    safe_zone.push(safe);
    safe_zone.push((safe.0 - 1, safe.1 - 1));
    safe_zone.push((safe.0, safe.1 - 1));
    safe_zone.push((safe.0 + 1, safe.1 - 1));
    safe_zone.push((safe.0 - 1, safe.1));
    safe_zone.push((safe.0 + 1, safe.1));
    safe_zone.push((safe.0 - 1, safe.1 + 1));
    safe_zone.push((safe.0, safe.1 + 1));
    safe_zone.push((safe.0 + 1, safe.1 + 1));
    

    'outer: while i > 0 {
        let cords: (u8, u8) = (rng.gen_range(1..HARD_BOARD_SIZE.y as u8 + 1), rng.gen_range(1..HARD_BOARD_SIZE.x as u8 + 1));
        //println!("{:?}", selected);
        for x in selected.iter() {
            if cords == *x {
                //println!("{:?}", cords);
                continue 'outer;
            }
        }
        for x in safe_zone.iter() {
            if cords == *x {
                //println!("{:?}", cords);
                continue 'outer;
            }
        }
        i -= 1;
        selected.push(cords);
    }
    println!("Final: {:?}", selected);
    selected
}

fn set_bombs(
    mut tiles: Query<&mut Tile>,
    safe: Res<Safe>,
    
) {
    println!("There are {} Entities spawned!", tiles.iter().count());
    let positions = generate_bomb_positions(safe.cords);
    
    
    for mut tile_bomb in tiles.iter_mut() {
        for cords in positions.iter(){
            if (tile_bomb.x, tile_bomb.y) == *cords {
                tile_bomb.tile_type = TileType::Bomb;
            }
        }
    }

    for mut tile in tiles.iter_mut() {
        for cords in positions.iter() {
            if tile.tile_type != TileType::Bomb {
                if (tile.x + 1, tile.y + 1) == *cords {
                    tile.num += 1;
                }
                if (tile.x, tile.y + 1) == *cords {
                    tile.num += 1;
                }
                if (tile.x - 1, tile.y + 1) == *cords {
                    tile.num += 1;
                }
                if (tile.x + 1, tile.y) == *cords {
                    tile.num += 1;
                }
                if (tile.x - 1, tile.y) == *cords {
                    tile.num += 1;
                }
                if (tile.x + 1, tile.y - 1) == *cords {
                    tile.num += 1;
                }
                if (tile.x, tile.y - 1) == *cords {
                    tile.num += 1;
                }
                if (tile.x - 1, tile.y - 1) == *cords {
                    tile.num += 1;
                }
            }
        }
    }
    // println!("Set!");
    
}

fn click_switch(
    buttons: Res<Input<MouseButton>>,
    window: Query<&Window>,
    mut tiles: Query<(&mut Tile, &mut Transform, &mut Handle<Image>)>, 
    tile_sprites: Res<TileSprites>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = window.get_single().unwrap().cursor_position() {
        println!("{}", position);
            for (mut tile, g_transorm, mut image) in tiles.iter_mut() {
                if tile.covered && !tile.flag {
                    let x = g_transorm.translation.x;
                    let y = g_transorm.translation.y;
                    if (position.x > x - CLICK_AREA_SIZE && position.x < x + CLICK_AREA_SIZE) &&
                       (position.y < y + CLICK_AREA_SIZE && position.y > y - CLICK_AREA_SIZE) {
                        tile.covered = false;
                        if tile.tile_type == TileType::Bomb {
                            *image = tile_sprites.exploded.clone();
                            next_state.set(GameState::GameOver);
                        }
                    }
                }
            }
        }
    }

    if buttons.just_pressed(MouseButton::Right) {
        if let Some(position) = window.get_single().unwrap().cursor_position() {
            for (mut tile, g_transorm, mut image) in tiles.iter_mut() {
                if tile.covered {
                    let x = g_transorm.translation.x;
                    let y = g_transorm.translation.y;

                    if (position.x > x - CLICK_AREA_SIZE && position.x < x + CLICK_AREA_SIZE) &&
                       (position.y < y + CLICK_AREA_SIZE && position.y > y - CLICK_AREA_SIZE) {
                        if !tile.flag {
                            tile.flag = true;
                            println!("Flaga up!");
                            *image = tile_sprites.flag.clone();
                        } else {
                            tile.flag = false;
                            *image = tile_sprites.unknown.clone();
                        }
                    }
                }
            }
        }
    }
}

fn tile_check(
    mut tiles: Query<(&mut Tile, &mut Handle<Image>)>,
    tile_sprites: Res<TileSprites>,
    mut zeros: ResMut<Empty>
) {
    for (tile1, mut image1) in tiles.iter_mut() {
        //println!("{}/{}", tile1.x, tile1.y);
        if !tile1.covered && tile1.tile_type != TileType::Bomb {
            match tile1.num {
                0 => {
                    *image1 = tile_sprites.zero.clone();
                    zeros.cords.push((tile1.x, tile1.y));
                },
                1 => *image1 = tile_sprites.one.clone(),
                2 => *image1 = tile_sprites.two.clone(),
                3 => *image1 = tile_sprites.three.clone(),
                4 => *image1 = tile_sprites.four.clone(),
                5 => *image1 = tile_sprites.five.clone(),
                6 => *image1 = tile_sprites.six.clone(),
                7 => *image1 = tile_sprites.seven.clone(),
                8 => *image1 = tile_sprites.eight.clone(),
                9 => *image1 = tile_sprites.nine.clone(),
                _ => panic!(),
            }
        }
    }
}

fn zero_check(
    mut zeros: ResMut<Empty>,
    mut tiles: Query<&mut Tile>
) {
    if !zeros.cords.is_empty(){
        // println!("Jest jakiś jebaniec!!");
        for (x, y) in &zeros.cords {
            for mut tile in tiles.iter_mut(){
                if tile.covered && tile.x >= x - 1 && tile.x <= x + 1 && tile.y >= y - 1 && tile.y <= y + 1 {
                    tile.covered = false;
                }
            }
        }
        zeros.cords.clear();
    }
}

fn game_over(
    mut tiles: Query<(&Tile, &mut Handle<Image>)>,
    tile_sprites: Res<TileSprites>
) {
    for (tile, mut image) in tiles.iter_mut() {
        
        if tile.covered && tile.tile_type == TileType::Bomb && !tile.flag{
            *image = tile_sprites.bomb.clone();
        } //else {
            //     match tile.num {
            //         0 => *image = asset_server.load("sprites/tile_empty2.png"),
            //         1 => *image = asset_server.load("sprites/tile_one2.png"),
            //         2 => *image = asset_server.load("sprites/tile_two2.png"),
            //         3 => *image = asset_server.load("sprites/tile_three2.png"),
            //         4 => *image = asset_server.load("sprites/tile_four2.png"),
            //         5 => *image = asset_server.load("sprites/tile_five2.png"),
            //         6 => *image = asset_server.load("sprites/tile_six2.png"),
            //         7 => *image = asset_server.load("sprites/tile_seven2.png"),
            //         8 => *image = asset_server.load("sprites/tile_eight2.png"),
            //         9 => *image = asset_server.load("sprites/tile_nine2.png"),
            //         _ => panic!(),
            //     }
            // }
        
    }
}

fn first_click(
    buttons: Res<Input<MouseButton>>,
    window: Query<&Window>,
    mut tiles: Query<(&mut Tile, &Transform)>,
    mut safe: ResMut<Safe>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = window.get_single().unwrap().cursor_position() {
            for (mut tile, g_transorm) in tiles.iter_mut() {
                let x = g_transorm.translation.x;
                let y = g_transorm.translation.y;
                if (position.x > x - CLICK_AREA_SIZE && position.x < x + CLICK_AREA_SIZE) &&
                   (position.y < y + CLICK_AREA_SIZE && position.y > y - CLICK_AREA_SIZE) {
                    safe.cords = (tile.x, tile.y);
                    tile.covered = false;
                    next_state.set(GameState::InGame);

                }
            }
        }
    }
}

fn reset_click_check(
    buttons: Res<Input<MouseButton>>,
    window: Query<&Window>,
    mut next_state: ResMut<NextState<GameState>>,
    reset: Res<Reset>
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = window.get_single().unwrap().cursor_position() {
            println!("{}", position);
            if (position.x > reset.position.0 - CLICK_AREA_SIZE && position.x < reset.position.0 + CLICK_AREA_SIZE) &&
                (position.y < reset.position.1 + CLICK_AREA_SIZE && position.y > reset.position.1 - CLICK_AREA_SIZE) {
            
                next_state.set(GameState::SafeClick);
            }
        }
    }
}
    