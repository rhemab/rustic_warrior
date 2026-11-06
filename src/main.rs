use macroquad::prelude::*;

mod assets;

pub const SQUARE_SIZE: f32 = 50.0;
const NUM_SQUARES: usize = 30;

pub const FPS: f32 = 10.0;

const HOVER_COLOR: Color = Color::from_rgba(255, 255, 255, 64);
const MOVEMENT_COLOR: Color = Color::from_rgba(255, 255, 255, 80);

pub const TILEMAP: &[u8] =
    include_bytes!("../assets/tiny_swords/Terrain/Tileset/Tilemap_color1.png");

#[derive(Debug, Clone, Copy)]
enum UnitKind {
    Warrior,
    Archer,
}

#[derive(Debug, Clone, Copy)]
enum Action {
    Idle,
    Run,
    Attack,
}

#[derive(Debug, Clone)]
struct Unit {
    unit_kind: UnitKind,
    action: Action,
    game_pos: [usize; 2],
    view_pos: Vec2,
    flip_x: bool,
    movement: usize,
    speed: f32,
}

struct GameState {
    grass: Vec<Vec<bool>>,
    units: Vec<Unit>,
    enemy_units: Vec<Unit>,
    selected_unit: Option<usize>,
}

#[macroquad::main("Rustic Warrior")]
async fn main() {
    // set_fullscreen(true);
    request_new_screen_size(2000.0, 2000.0);

    let blue_units = assets::Units::new_blue();
    let red_units = assets::Units::new_red();
    let terrain = assets::Terrain::new();

    let tiles = Texture2D::from_file_with_format(TILEMAP, None);
    tiles.set_filter(FilterMode::Nearest);

    let mut game_state = GameState {
        grass: vec![vec![true; NUM_SQUARES]; NUM_SQUARES],
        units: vec![],
        enemy_units: vec![],
        selected_unit: None,
    };

    // generate units
    {
        let x = 12;
        let y = 12;

        let warrior = new_unit(UnitKind::Warrior, x, y);
        game_state.units.push(warrior);

        let enemy_archer = new_unit(UnitKind::Archer, x + 1, y);
        game_state.enemy_units.push(enemy_archer);
    }

    let mut available_moves: Vec<[usize; 2]> = vec![];

    let mut t = 0.0;

    let mut cam = Camera2D::default();
    cam.target = vec2(NUM_SQUARES as f32 * SQUARE_SIZE / 2.0, 600.0);

    loop {
        cam.zoom = vec2(2.0 / screen_width(), 2.0 / screen_height());
        set_camera(&cam);
        clear_background(Color::from_rgba(71, 171, 169, 255));

        t += get_frame_time();

        let (mx, my) = mouse_position();
        let mouse_world = cam.screen_to_world(vec2(mx, my));

        // draw grass
        for x in 0..NUM_SQUARES {
            for y in 0..NUM_SQUARES {
                let px = x as f32 * SQUARE_SIZE;
                let py = y as f32 * SQUARE_SIZE;
                if !game_state.grass[y][x] {
                    terrain.water.draw(t, px, py, false);
                    continue;
                }
                let (c, r) = pick_tile(&game_state.grass, x as i32, y as i32);
                draw_texture_ex(
                    &tiles,
                    px,
                    py,
                    WHITE,
                    DrawTextureParams {
                        source: Some(src(c, r)),
                        dest_size: Some(vec2(SQUARE_SIZE, SQUARE_SIZE)),
                        ..Default::default()
                    },
                );
            }
        }

        // draw clouds & trees
        for x in 0..NUM_SQUARES {
            for y in 0..NUM_SQUARES {
                let px = x as f32 * SQUARE_SIZE;
                let py = y as f32 * SQUARE_SIZE;

                if x == 3 && y == 4 {
                    terrain.cloud1.draw(t, px, py, false);
                }
                if x == 13 && y == 4 {
                    terrain.cloud2.draw(t, px, py, false);
                }
                if x == 20 && y == 4 {
                    terrain.cloud3.draw(t, px, py, false);
                }
                if x == 20 && y == 4 {
                    terrain.cloud4.draw(t, px, py, false);
                }
                if x == 10 && y == 10 {
                    terrain.tree1.draw(t, px, py, false);
                }
                if x == 20 && y == 12 {
                    terrain.tree1.draw(t, px, py, false);
                }
            }
        }

        // check for hovered unit
        let mut hovering_unit = None;
        for (i, unit) in game_state.units.iter().enumerate() {
            let pos = unit.view_pos;
            let x_max = pos.x + SQUARE_SIZE;
            let y_max = pos.y + SQUARE_SIZE;

            if mouse_world.x > pos.x && mouse_world.x < x_max {
                if mouse_world.y > pos.y && mouse_world.y < y_max {
                    draw_rectangle(pos.x, pos.y, SQUARE_SIZE, SQUARE_SIZE, HOVER_COLOR);
                    hovering_unit = Some(i);
                }
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(i) = game_state.selected_unit {
                if hovering_unit.is_none() {
                    for game_pos in &available_moves {
                        let pos = vec2(
                            game_pos[0] as f32 * SQUARE_SIZE,
                            game_pos[1] as f32 * SQUARE_SIZE,
                        );
                        let x_max = pos.x + SQUARE_SIZE;
                        let y_max = pos.y + SQUARE_SIZE;
                        if mouse_world.x > pos.x && mouse_world.x < x_max {
                            if mouse_world.y > pos.y && mouse_world.y < y_max {
                                game_state.units[i].game_pos = *game_pos;
                            }
                        }
                    }
                }
            }
            game_state.selected_unit = hovering_unit;
        }

        // show available moves for selected unit
        if let Some(i) = game_state.selected_unit {
            let unit = &game_state.units[i];
            available_moves = unit.get_available_moves(&game_state);
            for game_pos in &available_moves {
                let pos = vec2(
                    game_pos[0] as f32 * SQUARE_SIZE,
                    game_pos[1] as f32 * SQUARE_SIZE,
                );
                let mut color = MOVEMENT_COLOR;
                let x_max = pos.x + SQUARE_SIZE;
                let y_max = pos.y + SQUARE_SIZE;
                if mouse_world.x > pos.x && mouse_world.x < x_max {
                    if mouse_world.y > pos.y && mouse_world.y < y_max {
                        color = Color::from_rgba(255, 255, 255, 200);
                    }
                }
                draw_rectangle(pos.x, pos.y, SQUARE_SIZE - 1.0, SQUARE_SIZE - 1.0, color);
            }
        }

        // draw units last
        let dt = get_frame_time();

        for unit in &mut game_state.units {
            unit.update_pos(dt);
            unit.draw(t, &blue_units);
        }

        for unit in &mut game_state.enemy_units {
            unit.update_pos(dt);
            unit.draw(t, &red_units);
        }

        // handle mouse drag
        if is_mouse_button_down(MouseButton::Left) {
            let delta = mouse_delta_position();
            cam.target.x += delta.x / cam.zoom.x;
            cam.target.y += delta.y / cam.zoom.y;
        }

        next_frame().await
    }
}

impl Unit {
    fn update_pos(&mut self, dt: f32) {
        let target_pos = vec2(
            self.game_pos[0] as f32 * SQUARE_SIZE,
            self.game_pos[1] as f32 * SQUARE_SIZE,
        );

        let to_target = target_pos - self.view_pos;
        let step = self.speed * dt;

        if to_target.length() <= step {
            self.action = Action::Idle;
            self.view_pos = target_pos;
        } else {
            self.action = Action::Run;
            let dir = to_target.normalize();
            self.view_pos += dir * step;
            if dir.x != 0.0 {
                self.flip_x = dir.x < 0.0; // face the direction of travel
            }
        }
    }
    fn draw(&self, t: f32, units: &assets::Units) {
        let unit_kind = match self.unit_kind {
            UnitKind::Warrior => &units.warrior,
            UnitKind::Archer => &units.archer,
        };
        match self.action {
            Action::Idle => unit_kind
                .idle
                .draw(t, self.view_pos.x, self.view_pos.y, self.flip_x),
            Action::Run => unit_kind
                .run
                .draw(t, self.view_pos.x, self.view_pos.y, self.flip_x),
            Action::Attack => {
                unit_kind
                    .attack
                    .draw(t, self.view_pos.x, self.view_pos.y, self.flip_x)
            }
        }
    }

    fn get_available_moves(&self, game_state: &GameState) -> Vec<[usize; 2]> {
        let mut available_moves = vec![];
        let x = self.game_pos[0];
        let y = self.game_pos[1];

        let mut visited = vec![];

        let mut q = std::collections::VecDeque::new();
        let root = [x, y];
        q.push_back(root);

        while let Some(mut pos) = q.pop_front() {
            if pos != root {
                pos[0] = pos[0].clamp(0, NUM_SQUARES - 1);
                pos[1] = pos[1].clamp(0, NUM_SQUARES - 1);
                if visited.contains(&pos) {
                    continue;
                }
                let occupied = game_state
                    .units
                    .iter()
                    .find(|u| u.game_pos == pos)
                    .is_some()
                    || game_state
                        .enemy_units
                        .iter()
                        .find(|u| u.game_pos == pos)
                        .is_some();
                if occupied {
                    continue;
                }
                let x_diff = pos[0].abs_diff(x);
                let y_diff = pos[1].abs_diff(y);
                if x_diff + y_diff > self.movement {
                    continue;
                }
            }

            visited.push(pos);
            available_moves.push([pos[0], pos[1]]);

            q.push_back([pos[0] + 1, pos[1]]);
            q.push_back([pos[0].saturating_sub(1), pos[1]]);
            q.push_back([pos[0], pos[1] + 1]);
            q.push_back([pos[0], pos[1].saturating_sub(1)]);
            q.push_back([pos[0] + 1, pos[1] + 1]);
            q.push_back([pos[0] + 1, pos[1].saturating_sub(1)]);
            q.push_back([pos[0].saturating_sub(1), pos[1] + 1]);
            q.push_back([pos[0].saturating_sub(1), pos[1].saturating_sub(1)]);
        }

        available_moves
    }
}

const TILE: f32 = 64.0;

fn src(col: usize, row: usize) -> Rect {
    Rect::new(col as f32 * TILE, row as f32 * TILE, TILE, TILE)
}

fn grass_at(map: &Vec<Vec<bool>>, x: i32, y: i32) -> bool {
    x >= 0
        && y >= 0
        && (y as usize) < map.len()
        && (x as usize) < map[0].len()
        && map[y as usize][x as usize]
}

fn pick_tile(map: &Vec<Vec<bool>>, x: i32, y: i32) -> (usize, usize) {
    let up = grass_at(map, x, y - 1);
    let down = grass_at(map, x, y + 1);
    let left = grass_at(map, x - 1, y);
    let right = grass_at(map, x + 1, y);

    let col = match (left, right) {
        (false, true) => 0,
        (true, true) => 1,
        (true, false) => 2,
        (false, false) => 3, // 1 tile wide
    };
    let row = match (up, down) {
        (false, true) => 0,
        (true, true) => 1,
        (true, false) => 2,
        (false, false) => 3, // 1 tile tall
    };
    (col, row)
}

fn new_unit(unit_kind: UnitKind, x: usize, y: usize) -> Unit {
    Unit {
        unit_kind,
        action: Action::Idle,
        game_pos: [x, y],
        view_pos: vec2(x as f32 * SQUARE_SIZE, y as f32 * SQUARE_SIZE),
        flip_x: false,
        movement: 5,
        speed: 50.0,
    }
}
