use bevy::{prelude::*, window::{PrimaryWindow, WindowResolution}};
//use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::{thread_rng, Rng};

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
enum GameState {
    #[default]
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
struct EmptyCords { cords: Vec<(u8, u8)> }

#[derive(Component)]
#[derive(Debug)]
#[derive(Resource)]
#[derive(Reflect)]
// #[reflect(Component)]
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
        .insert_resource(EmptyCords{cords: vec!()})
        .add_state::<GameState>()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(
                        Window{
                            title: "Minesweeper".to_string(),
                            resolution: WindowResolution::new(HARD_BOARD_SIZE.y * 19.0 * 2.0, HARD_BOARD_SIZE.x * 19.0 * 2.0),
                            fit_canvas_to_parent: true,
                            ..default()
                        }),
                    ..default()
                })
        )
        //.add_plugin(WorldInspectorPlugin::new())
        .add_startup_systems(
            (
            spawn_camera,
            spawn_tiles,
            apply_system_buffers,
            set_bombs
            ).chain()
        )
        .add_systems(
            (
            click_switch,
            apply_system_buffers, 
            tile_check,
            apply_system_buffers,
            zero_check
            ).in_set(OnUpdate(GameState::InGame))
        )
        .add_system(game_over.in_schedule(OnExit(GameState::InGame)))
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
    println!("Camera's Up!");
}


fn spawn_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window: &Window = window.get_single().unwrap();
    let mut x = 0;
    
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
                    texture: asset_server.load("sprites/tile_unknown2.png"),
                    transform: Transform::from_xyz(
                        19.0 * 0.5 * 2.0 + j as f32 * 19.0 * 2.0,
                        window.height() - 19.0 * 1.5 - i as f32 * 19.0 * 2.0 + 9.5, 
                        0.0,
                    ).with_scale(Vec3::new(2.0, 2.0, 0.0)),
                    
                    ..default()
                },
                
                Name::new("Tile".to_string() + &x.to_string() + " (" + &(j+1).to_string() + ", " + &(i+1).to_string() + ")")
            ));
        }
    }
    println!("Spawned!");
}


fn generate_bomb_positions() -> Vec<(u8, u8)> {
    let mut rng = thread_rng();
    let mut selected: Vec<(u8, u8)> = Vec::new();
    let mut i = HARD_BOMB_COUNT;

    'outer: while i > 0 {
        let cords: (u8, u8) = (rng.gen_range(1..HARD_BOARD_SIZE.y as u8 + 1), rng.gen_range(1..HARD_BOARD_SIZE.x as u8 + 1));
        //println!("{:?}", selected);
        for x in selected.iter() {
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
) {
    println!("There are {} Entities spawned!", tiles.iter().count());
    let positions = generate_bomb_positions();
    
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
    println!("Set!");
}

fn click_switch(
    buttons: Res<Input<MouseButton>>,
    window: Query<&Window>,
    mut tiles: Query<(&mut Tile, &mut Transform, &mut Handle<Image>)>, 
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(position) = window.get_single().unwrap().cursor_position() {
            for (mut tile, g_transorm, mut image) in tiles.iter_mut() {
                if tile.covered && !tile.flag {
                    let x = g_transorm.translation.x;
                    let y = g_transorm.translation.y;
                    if (position.x > x - CLICK_AREA_SIZE && position.x < x + CLICK_AREA_SIZE) &&
                       (position.y < y + CLICK_AREA_SIZE && position.y > y - CLICK_AREA_SIZE) {
                        tile.covered = false;
                        if tile.tile_type == TileType::Bomb {
                            *image = asset_server.load("sprites/tile_exploded2.png");
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
                            *image = asset_server.load("sprites/tile_flag2.png");
                        } else {
                            tile.flag = false;
                            *image = asset_server.load("sprites/tile_unknown2.png");
                        }
                    }
                }
            }
        }
    }
}

fn tile_check(
    mut tiles: Query<(&mut Tile, &mut Handle<Image>)>,
    asset_server: Res<AssetServer>,
    mut zeros: ResMut<EmptyCords>
) {
    for (tile1, mut image1) in tiles.iter_mut() {
        //println!("{}/{}", tile1.x, tile1.y);
        if !tile1.covered && tile1.tile_type != TileType::Bomb {
            match tile1.num {
                0 => {
                    *image1 = asset_server.load("sprites/tile_empty2.png");
                    zeros.cords.push((tile1.x, tile1.y));
                },
                1 => *image1 = asset_server.load("sprites/tile_one2.png"),
                2 => *image1 = asset_server.load("sprites/tile_two2.png"),
                3 => *image1 = asset_server.load("sprites/tile_three2.png"),
                4 => *image1 = asset_server.load("sprites/tile_four2.png"),
                5 => *image1 = asset_server.load("sprites/tile_five2.png"),
                6 => *image1 = asset_server.load("sprites/tile_six2.png"),
                7 => *image1 = asset_server.load("sprites/tile_seven2.png"),
                8 => *image1 = asset_server.load("sprites/tile_eight2.png"),
                9 => *image1 = asset_server.load("sprites/tile_nine2.png"),
                _ => panic!(),
            }
        }
    }
}

fn zero_check(
    mut zeros: ResMut<EmptyCords>,
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
    asset_server: Res<AssetServer>
) {
    for (tile, mut image) in tiles.iter_mut() {
        if tile.covered {
            if tile.tile_type == TileType::Bomb {
                *image = asset_server.load("sprites/tile_bomb2.png")
            } else {
                match tile.num {
                    0 => *image = asset_server.load("sprites/tile_empty2.png"),
                    1 => *image = asset_server.load("sprites/tile_one2.png"),
                    2 => *image = asset_server.load("sprites/tile_two2.png"),
                    3 => *image = asset_server.load("sprites/tile_three2.png"),
                    4 => *image = asset_server.load("sprites/tile_four2.png"),
                    5 => *image = asset_server.load("sprites/tile_five2.png"),
                    6 => *image = asset_server.load("sprites/tile_six2.png"),
                    7 => *image = asset_server.load("sprites/tile_seven2.png"),
                    8 => *image = asset_server.load("sprites/tile_eight2.png"),
                    9 => *image = asset_server.load("sprites/tile_nine2.png"),
                    _ => panic!(),
                }
            }
        }
    }
}