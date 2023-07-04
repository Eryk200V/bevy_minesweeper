#![windows_subsystem = "windows"]

//cargo build --target=x86_64-pc-windows-gnu --release


//cargo build --target=x86_64-pc-windows-gnu --release && cargo build --release --target wasm32-unknown-unknown && wasm-bindgen --out-dir ./out/ --web target/wasm32-unknown-unknown/release/saper.wasm


use bevy::{prelude::*, window::{PrimaryWindow, WindowResolution}};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;
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
const TILE_SIZE: f32 = 19.0 * 2.0;


const EAZY_BOARD_SIZE: (u8, u8) = (10, 10);
const EAZY_BOMB_COUNT: u8 = 10;

const MEDIUM_BOARD_SIZE: (u8, u8) = (12, 12);
const MEDIUM_BOMB_COUNT: u8 = 26;

const HARD_BOARD_SIZE: (u8, u8) = (15, 15);
const HARD_BOMB_COUNT: u8 = 40;

const EXPERT_BOARD_SIZE: (u8, u8) = (16, 30);
const EXPERT_BOMB_COUNT: u8 = 99;

#[derive(Resource)]
struct Empty { cords: Vec<(u8, u8)> }

#[derive(Resource)]
struct ButtonPositions { 
    eazy: (f32, f32),
    medium: (f32, f32),
    hard: (f32, f32),
    expert: (f32, f32)
}

#[derive(Resource)]
struct Safe { cords: (u8, u8) }

#[derive(Resource)]
struct MapInfo { 
    board_size: (u8, u8),
    bomb_count: u8, 
}


#[derive(AssetCollection, Resource)]
struct TileSprites {

    #[asset(path = "sprites/eazy2.png")]
    eazy: Handle<Image>,

    #[asset(path = "sprites/medium2.png")]
    medium: Handle<Image>,

    #[asset(path = "sprites/hard2.png")]
    hard: Handle<Image>,

    #[asset(path = "sprites/expert2.png")]
    expert: Handle<Image>,

    #[asset(path = "sprites/tile_unknown2.png")]
    unknown: Handle<Image>,

    #[asset(path = "sprites/tile_exploded2.png")]
    exploded: Handle<Image>,

    #[asset(path = "sprites/tile_flag2.png")]
    flag: Handle<Image>,

    #[asset(path = "sprites/tile_flag_cross2.png")]
    flag_cross: Handle<Image>,

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
#[derive(Reflect)]
// #[reflect(Component)]
#[derive(Component)]
pub struct Tile { 
    x: u8,
    y: u8,
    num: u8,
    bomb: bool,
    covered: bool,
    flag: bool
}

#[derive(Component)]
pub struct Button;

fn main() {
    App::new()
        .register_type::<Tile>()
        .insert_resource(Empty{cords: vec!()})
        .insert_resource(Safe{cords: (0,0)})
        .insert_resource(ButtonPositions{
            eazy: (0.0, 0.0),
            medium: (0.0, 0.0),
            hard: (0.0, 0.0),
            expert: (0.0, 0.0)
        })
        .insert_resource(MapInfo{
            board_size: EAZY_BOARD_SIZE,
            bomb_count: EAZY_BOMB_COUNT
        })
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
                            resolution: WindowResolution::new(EAZY_BOARD_SIZE.1 as f32 * TILE_SIZE, EAZY_BOARD_SIZE.0 as f32 * TILE_SIZE + TILE_SIZE),
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
            ).chain().in_schedule(OnEnter(GameState::InGame))
        )
        .add_system(game_over.in_schedule(OnEnter(GameState::GameOver)))
        .add_system(button_click_check)
        .run();
}

pub fn spawn_camera(
    mut commands: Commands, 
    window_query: Query<&Window>
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
    commands.despawn_all::<With<Button>>();
}

fn spawn_tiles(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut buttons: ResMut<ButtonPositions>,
    tile_sprites: Res<TileSprites>,
    map_info: Res<MapInfo>
) {
    let window: &Window = window.get_single().unwrap();
    let mut x = 0;

    let size_x = map_info.board_size.0;
    let size_y = map_info.board_size.1;

    
    println!("{:?}", std::env::current_exe());
    for i in 0..size_x {
        for j in 0..size_y {
            x += 1;
            commands.spawn((
                Tile{
                    x: j + 1,
                    y: i + 1,
                    num: 0,
                    bomb: false,
                    covered: true,
                    flag: false
                },
                SpriteBundle{
                    texture: tile_sprites.unknown.clone(),
                    transform: Transform::from_xyz(
                        TILE_SIZE * 0.5 + j as f32 * TILE_SIZE,
                        window.height() - TILE_SIZE - i as f32 * TILE_SIZE - TILE_SIZE * 0.5, 
                        0.0,
                    ).with_scale(Vec3::new(2.0, 2.0, 0.0)),
                    ..default()
                },
                
                Name::new("Tile".to_string() + &x.to_string() + " (" + &(j+1).to_string() + ", " + &(i+1).to_string() + ")")
            ));
        }
    }
    //eazy button
    commands.spawn(
        (
            SpriteBundle {
            texture: tile_sprites.eazy.clone(),
            transform: Transform::from_xyz(
                TILE_SIZE * 0.5,
                window.height() - TILE_SIZE * 0.5, 
                0.0,
            ).with_scale(Vec3::new(2.0, 2.0, 0.0)),
            ..default()
            },
            Button,
            Name::new("Reset".to_string()),
        )
    );
    buttons.eazy = (TILE_SIZE * 0.5, window.height() - TILE_SIZE * 0.5);

    //medium button
    commands.spawn(
        (
            SpriteBundle {
            texture: tile_sprites.medium.clone(),
            transform: Transform::from_xyz(
                TILE_SIZE * 0.5 + TILE_SIZE,
                window.height() - TILE_SIZE * 0.5, 
                0.0,
            ).with_scale(Vec3::new(2.0, 2.0, 0.0)),
            ..default()
            },
            Button,
            Name::new("Reset".to_string()),
        )
    );
    buttons.medium = (TILE_SIZE * 0.5 + TILE_SIZE, window.height() - TILE_SIZE * 0.5);

    //hard button
    commands.spawn(
        (
            SpriteBundle {
            texture: tile_sprites.hard.clone(),
            transform: Transform::from_xyz(
                TILE_SIZE * 0.5 + 2.0 * TILE_SIZE,
                window.height() - TILE_SIZE * 0.5, 
                0.0,
            ).with_scale(Vec3::new(2.0, 2.0, 0.0)),
            ..default()
            },
            Button,
            Name::new("Reset".to_string()),
        )
    );
    buttons.hard = (TILE_SIZE * 0.5 + 2.0 * TILE_SIZE, window.height() - TILE_SIZE * 0.5);
    
    //expert button 
    commands.spawn(
        (
            SpriteBundle {
            texture: tile_sprites.expert.clone(),
            transform: Transform::from_xyz(
                TILE_SIZE * 0.5 + 3.0 * TILE_SIZE,
                window.height() - TILE_SIZE * 0.5, 
                0.0,
            ).with_scale(Vec3::new(2.0, 2.0, 0.0)),
            ..default()
            },
            Button,
            Name::new("Reset".to_string()),
        )
    );
    buttons.expert = (TILE_SIZE * 0.5 + 3.0 * TILE_SIZE, window.height() - TILE_SIZE * 0.5);

    println!("Spawned!");
}


fn generate_bomb_positions(safe: (u8, u8), map_size: (u8, u8), bomb_count: u8) -> Vec<(u8, u8)> {
    let mut rng = thread_rng();
    let mut selected: Vec<(u8, u8)> = Vec::new();
    let mut safe_zone: Vec<(u8, u8)> = Vec::new();
    let mut i = bomb_count;
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
        let cords: (u8, u8) = (rng.gen_range(1..map_size.1 + 1), rng.gen_range(1..map_size.0 + 1));
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
    map_info: Res<MapInfo>
) {
    println!("There are {} Entities spawned!", tiles.iter().count());

    let positions = generate_bomb_positions(safe.cords, map_info.board_size, map_info.bomb_count);
    
    
    for mut tile_bomb in tiles.iter_mut() {
        for cords in positions.iter(){
            if (tile_bomb.x, tile_bomb.y) == *cords {
                tile_bomb.bomb = true;
            }
        }
    }

    for mut tile in tiles.iter_mut() {
        for cords in positions.iter() {
            if !tile.bomb {
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
                        if tile.bomb {
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
        if !tile1.covered && !tile1.bomb {
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
        
        if tile.covered && tile.bomb && !tile.flag{
            *image = tile_sprites.bomb.clone();
        } else if tile.flag && !tile.bomb {
            *image = tile_sprites.flag_cross.clone();
        }
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

fn button_click_check(
    mouse_buttons: Res<Input<MouseButton>>,
    mut window: Query<&mut Window>,
    mut camera: Query<(&Camera, &mut Transform)>,
    mut next_state: ResMut<NextState<GameState>>,
    buttons: Res<ButtonPositions>,
    mut map_info: ResMut<MapInfo>
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = window.get_single().unwrap().cursor_position() {
            println!("{}", position);
            if (position.x > buttons.eazy.0 - CLICK_AREA_SIZE && position.x < buttons.eazy.0 + CLICK_AREA_SIZE) &&
                (position.y < buttons.eazy.1 + CLICK_AREA_SIZE && position.y > buttons.eazy.1 - CLICK_AREA_SIZE) {
                map_info.board_size = EAZY_BOARD_SIZE;
                map_info.bomb_count = EAZY_BOMB_COUNT;
                window.single_mut().resolution = WindowResolution::new(EAZY_BOARD_SIZE.1 as f32 * TILE_SIZE, EAZY_BOARD_SIZE.0 as f32 * TILE_SIZE + TILE_SIZE);

                let (_camera, mut transform) = camera.get_single_mut().unwrap();
                let window = window.get_single().unwrap();
                *transform = Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0);
                next_state.set(GameState::SafeClick);
            } 
            else if (position.x > buttons.medium.0 - CLICK_AREA_SIZE && position.x < buttons.medium.0 + CLICK_AREA_SIZE) &&
                (position.y < buttons.medium.1 + CLICK_AREA_SIZE && position.y > buttons.medium.1 - CLICK_AREA_SIZE) {
                map_info.board_size = MEDIUM_BOARD_SIZE;
                map_info.bomb_count = MEDIUM_BOMB_COUNT;
                window.single_mut().resolution = WindowResolution::new(MEDIUM_BOARD_SIZE.1 as f32 * TILE_SIZE, MEDIUM_BOARD_SIZE.0 as f32 * TILE_SIZE + TILE_SIZE);

                let (_camera, mut transform) = camera.get_single_mut().unwrap();
                let window = window.get_single().unwrap();
                *transform = Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0);
                next_state.set(GameState::SafeClick);
            }
            else if (position.x > buttons.hard.0 - CLICK_AREA_SIZE && position.x < buttons.hard.0 + CLICK_AREA_SIZE) &&
                (position.y < buttons.hard.1 + CLICK_AREA_SIZE && position.y > buttons.hard.1 - CLICK_AREA_SIZE) {
                map_info.board_size = HARD_BOARD_SIZE;
                map_info.bomb_count = HARD_BOMB_COUNT;
                window.single_mut().resolution = WindowResolution::new(HARD_BOARD_SIZE.1 as f32 * TILE_SIZE, HARD_BOARD_SIZE.0 as f32 * TILE_SIZE + TILE_SIZE);

                let (_camera, mut transform) = camera.get_single_mut().unwrap();
                let window = window.get_single().unwrap();
                *transform = Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0);
                next_state.set(GameState::SafeClick);
            }
            else if (position.x > buttons.expert.0 - CLICK_AREA_SIZE && position.x < buttons.expert.0 + CLICK_AREA_SIZE) &&
                (position.y < buttons.expert.1 + CLICK_AREA_SIZE && position.y > buttons.expert.1 - CLICK_AREA_SIZE) {
                map_info.board_size = EXPERT_BOARD_SIZE;
                map_info.bomb_count = EXPERT_BOMB_COUNT;
                window.single_mut().resolution = WindowResolution::new(EXPERT_BOARD_SIZE.1 as f32 * TILE_SIZE, EXPERT_BOARD_SIZE.0 as f32 * TILE_SIZE + TILE_SIZE);

                let (_camera, mut transform) = camera.get_single_mut().unwrap();
                let window = window.get_single().unwrap();
                *transform = Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0);
                next_state.set(GameState::SafeClick);
            }
        }
    }
}