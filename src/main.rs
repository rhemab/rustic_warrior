use macroquad::prelude::*;

use std::collections::{HashMap, VecDeque};

mod assets;

pub const SQUARE_SIZE: f32 = 50.0;
const NUM_SQUARES: usize = 30;

pub const FPS: f32 = 10.0;

const HOVER_COLOR: Color = Color::from_rgba(255, 255, 255, 64);
const RED_HOVER_COLOR: Color = Color::from_rgba(255, 0, 0, 100);
const AVAILABLE_MOVES_COLOR: Color = Color::from_rgba(255, 255, 255, 80);
const SELECT_MOVE_COLOR: Color = Color::from_rgba(255, 255, 255, 200);

pub const TILEMAP: &[u8] =
    include_bytes!("../assets/tiny_swords/Terrain/Tileset/Tilemap_color1.png");

type Cell = [usize; 2];

#[derive(Debug, Clone, Copy)]
enum UnitKind {
    Warrior,
    Archer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Idle,
    Run,
    Attack((f32, usize)), // attack hold total animation time and enemy idx
}

#[derive(Debug, Clone)]
struct Unit {
    name: String,
    unit_kind: UnitKind,
    action: Action,
    game_pos: [usize; 2],
    view_pos: Vec2,
    move_path: Vec<Cell>,
    flip_x: bool,
    movement: usize,
    speed: f32,
    range: usize,
    max_health: f32,
    health: f32,
    attack: f32,
    last_dmg_taken: f32,
    show_dmg_taken: f32,
}

#[derive(Default)]
struct Game {
    grass: Vec<Vec<bool>>,
    units: Vec<Unit>,
    enemy_units: Vec<Unit>,
    selected_unit: Option<usize>,
    mouse_pos: Vec2,
    total_frame_time: f32,
    hovering_unit: Option<usize>,
    hovering_enemy_unit: Option<usize>,
    available_moves: Vec<[usize; 2]>,
    came_from: HashMap<Cell, Cell>,
}

#[macroquad::main("Rustic Warrior")]
async fn main() {
    set_fullscreen(true);

    let blue_units = assets::Units::new_blue();
    let red_units = assets::Units::new_red();
    let terrain = assets::Terrain::new();

    let tiles = Texture2D::from_file_with_format(TILEMAP, None);
    tiles.set_filter(FilterMode::Nearest);

    let assets = assets::Assets {
        blue_units,
        red_units,
        terrain,
        tiles,
    };

    let mut game = Game::default();
    game.grass = vec![vec![true; NUM_SQUARES]; NUM_SQUARES];

    // generate units
    {
        let x = 12;
        let y = 12;

        let warrior = new_unit("Erik".to_string(), UnitKind::Warrior, x, y);
        game.units.push(warrior);

        let enemy_archer = new_unit("Enemy".to_string(), UnitKind::Archer, x + 1, y);
        game.enemy_units.push(enemy_archer);
    }

    let mut cam = Camera2D::default();
    cam.target = vec2(NUM_SQUARES as f32 * SQUARE_SIZE / 2.0, 600.0);

    loop {
        cam.zoom = vec2(2.0 / screen_width(), 2.0 / screen_height());
        set_camera(&cam);
        clear_background(Color::from_rgba(71, 171, 169, 255));

        game.total_frame_time += get_frame_time();

        {
            let (mx, my) = mouse_position();
            let mouse_world = cam.screen_to_world(vec2(mx, my));
            game.mouse_pos = mouse_world;
        }

        // handle mouse drag
        if is_mouse_button_down(MouseButton::Left) {
            let delta = mouse_delta_position();
            cam.target.x += delta.x / cam.zoom.x;
            cam.target.y += delta.y / cam.zoom.y;
        }

        game.update(&assets);
        game.draw(&assets);
        game.draw_ui();

        next_frame().await
    }
}

impl Game {
    fn update(&mut self, assets: &assets::Assets) {
        // check for hovered unit
        self.hovering_unit = None;
        self.hovering_enemy_unit = None;

        for (i, unit) in self.units.iter().enumerate() {
            let pos = unit.view_pos;
            if self.mouse_in_pos(pos) {
                self.hovering_unit = Some(i);
            }
        }
        for (i, unit) in self.enemy_units.iter().enumerate() {
            let pos = unit.view_pos;
            if self.mouse_in_pos(pos) {
                self.hovering_enemy_unit = Some(i);
            }
        }

        if let Some(i) = self.selected_unit {
            if self.available_moves.is_empty() {
                let unit = &self.units[i];
                (self.available_moves, self.came_from) = unit.get_available_moves(&self);
            }
        }

        // check for selected unit
        if is_mouse_button_pressed(MouseButton::Left) {
            if let Some(i) = self.selected_unit {
                if self.hovering_unit.is_none() && self.hovering_enemy_unit.is_none() {
                    // move selected piece
                    if let Some(cell) = &self.mouse_to_cell() {
                        if self.available_moves.contains(cell) {
                            self.units[i].move_path.push(*cell);
                            let mut curr_cell = cell;
                            while let Some(c) = self.came_from.get(curr_cell) {
                                curr_cell = c;
                                self.units[i].move_path.push(*c);
                            }
                        }
                    }
                }
                if let Some(enemy_i) = self.hovering_enemy_unit {
                    // attack enemy
                    let unit = &mut self.units[i];
                    let enemy = &mut self.enemy_units[enemy_i];
                    let [unit_x, unit_y] = unit.game_pos;
                    let [enemy_x, enemy_y] = enemy.game_pos;
                    let distance =
                        ((unit_x + unit_y) as i32 - (enemy_x + enemy_y) as i32).abs() as usize;
                    if distance <= unit.range {
                        unit.action = Action::Attack((0.0, enemy_i));
                        if unit_x > enemy_x {
                            unit.flip_x = true;
                        } else {
                            unit.flip_x = false;
                        }
                    }
                }
            }
            // set selected unit
            self.selected_unit = self.hovering_unit;
            self.available_moves.clear();
            self.came_from.clear();
        }

        let dt = get_frame_time();

        for unit in &mut self.units {
            unit.update_pos(dt);
            match unit.action {
                Action::Attack((anim_time, enemy_i)) => {
                    let sprite = unit.sprite(&assets.blue_units);
                    let frame = anim_time * FPS;
                    if frame > sprite.frames {
                        unit.action = Action::Idle;
                        // update health & despawn after the animation finishes
                        let enemy = &mut self.enemy_units[enemy_i];
                        enemy.health -= unit.attack;
                        enemy.show_dmg_taken = 2.0;
                        enemy.last_dmg_taken = unit.attack;
                        if enemy.health <= 0.0 {
                            self.enemy_units.remove(enemy_i);
                            self.hovering_enemy_unit = None;
                        }
                    }
                }
                _ => {}
            }
            match &mut unit.action {
                Action::Attack((anim_time, _)) => {
                    *anim_time += dt;
                }
                _ => {}
            }
        }

        for unit in &mut self.enemy_units {
            unit.update_pos(dt);
            if unit.show_dmg_taken > 0.0 {
                unit.show_dmg_taken -= dt;
            }
        }
    }
    fn draw(&self, assets: &assets::Assets) {
        // draw grass
        for x in 0..NUM_SQUARES {
            for y in 0..NUM_SQUARES {
                let px = x as f32 * SQUARE_SIZE;
                let py = y as f32 * SQUARE_SIZE;
                if !self.grass[y][x] {
                    assets
                        .terrain
                        .water
                        .draw(self.total_frame_time, px, py, false, false);
                    continue;
                }
                let (c, r) = pick_tile(&self.grass, x as i32, y as i32);
                draw_texture_ex(
                    &assets.tiles,
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
                    assets
                        .terrain
                        .cloud1
                        .draw(self.total_frame_time, px, py, false, false);
                }
                if x == 13 && y == 4 {
                    assets
                        .terrain
                        .cloud2
                        .draw(self.total_frame_time, px, py, false, false);
                }
                if x == 20 && y == 4 {
                    assets
                        .terrain
                        .cloud3
                        .draw(self.total_frame_time, px, py, false, false);
                }
                if x == 20 && y == 4 {
                    assets
                        .terrain
                        .cloud4
                        .draw(self.total_frame_time, px, py, false, false);
                }
                if x == 10 && y == 10 {
                    assets
                        .terrain
                        .tree1
                        .draw(self.total_frame_time, px, py, false, false);
                }
                if x == 20 && y == 12 {
                    assets
                        .terrain
                        .tree1
                        .draw(self.total_frame_time, px, py, false, false);
                }
            }
        }

        // show available moves for selected unit
        if let Some(_) = self.selected_unit {
            for game_pos in &self.available_moves {
                let pos = vec2(
                    game_pos[0] as f32 * SQUARE_SIZE,
                    game_pos[1] as f32 * SQUARE_SIZE,
                );
                let mut color = AVAILABLE_MOVES_COLOR;
                if self.mouse_in_pos(pos) {
                    color = SELECT_MOVE_COLOR;
                }
                draw_rectangle(pos.x, pos.y, SQUARE_SIZE - 1.0, SQUARE_SIZE - 1.0, color);
            }

            if let Some(mouse_cell) = self.mouse_to_cell() {
                let mut c = &mouse_cell;
                while let Some(cell) = &self.came_from.get(c) {
                    c = *cell;
                    let pos = vec2(cell[0] as f32 * SQUARE_SIZE, cell[1] as f32 * SQUARE_SIZE);
                    draw_rectangle(
                        pos.x,
                        pos.y,
                        SQUARE_SIZE - 1.0,
                        SQUARE_SIZE - 1.0,
                        SELECT_MOVE_COLOR,
                    );
                }
            }
            if let Some(enemy_i) = self.hovering_enemy_unit {
                let enemy = &self.enemy_units[enemy_i];
                draw_rectangle(
                    enemy.view_pos.x,
                    enemy.view_pos.y,
                    SQUARE_SIZE,
                    SQUARE_SIZE,
                    RED_HOVER_COLOR,
                );
            }
        }

        // highlight hovered unit
        if let Some(i) = self.hovering_unit {
            let unit = &self.units[i];
            draw_rectangle(
                unit.view_pos.x,
                unit.view_pos.y,
                SQUARE_SIZE,
                SQUARE_SIZE,
                HOVER_COLOR,
            );
        }

        // draw units last
        for (i, unit) in self.units.iter().enumerate() {
            let mut hovered = false;
            if let Some(idx) = self.hovering_unit {
                if idx == i {
                    hovered = true;
                }
            }
            unit.draw(self.total_frame_time, hovered, &assets.blue_units);
        }

        for (i, unit) in self.enemy_units.iter().enumerate() {
            let mut hovered = false;
            if let Some(idx) = self.hovering_enemy_unit {
                if idx == i {
                    hovered = true;
                }
            }
            unit.draw(self.total_frame_time, hovered, &assets.red_units);
        }
    }

    fn draw_ui(&self) {
        set_default_camera();
        let mut unit = None;

        // draw hovered unit stats
        if let Some(i) = self.hovering_unit {
            unit = Some(&self.units[i]);
        }
        if let Some(i) = self.selected_unit {
            unit = Some(&self.units[i]);
        }
        if let Some(i) = self.hovering_enemy_unit {
            unit = Some(&self.enemy_units[i]);
        }

        if let Some(u) = unit {
            let margin = 0.0;
            let w = 200.0;
            let h = 150.0;

            let x = margin;
            let y = screen_height() - h - margin;
            let txt = format!(
                "{}\nHealth: {}/{}\nAttack: {}\nRange: {}\nMove: {}",
                u.name, u.health, u.max_health, u.attack, u.range, u.movement
            );
            draw_rectangle(x, y, w, h, SELECT_MOVE_COLOR);
            draw_multiline_text(&txt, x + 10.0, y + 35.0, 20.0, None, BLACK);
        }
    }

    fn mouse_in_pos(&self, pos: Vec2) -> bool {
        let x_max = pos.x + SQUARE_SIZE;
        let y_max = pos.y + SQUARE_SIZE;

        if self.mouse_pos.x > pos.x && self.mouse_pos.x < x_max {
            if self.mouse_pos.y > pos.y && self.mouse_pos.y < y_max {
                return true;
            }
        }

        false
    }

    fn mouse_to_cell(&self) -> Option<Cell> {
        let x = self.mouse_pos.x / SQUARE_SIZE;
        let y = self.mouse_pos.y / SQUARE_SIZE;

        if x >= 0.0 && x <= SQUARE_SIZE * NUM_SQUARES as f32 {
            if y >= 0.0 && y <= SQUARE_SIZE * NUM_SQUARES as f32 {
                return Some([x as usize, y as usize]);
            }
        }

        None
    }
}

impl Unit {
    fn update_pos(&mut self, dt: f32) {
        if self.action == Action::Idle
            && let Some(cell) = self.move_path.pop()
        {
            self.game_pos = cell;
        }
        match self.action {
            Action::Idle | Action::Run => {
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
            _ => {}
        }
    }

    fn draw(&self, t: f32, _hovered: bool, units: &assets::Units) {
        match self.action {
            Action::Attack((anim_time, _)) => {
                self.sprite(units).draw(
                    anim_time,
                    self.view_pos.x,
                    self.view_pos.y,
                    self.flip_x,
                    true,
                );
            }
            _ => {
                self.sprite(units)
                    .draw(t, self.view_pos.x, self.view_pos.y, self.flip_x, false);
            }
        }

        // draw health bar
        let offset = 5.0;
        let x = self.view_pos.x + offset;
        let y = self.view_pos.y + SQUARE_SIZE;
        let max_width = SQUARE_SIZE - (offset * 2.0);
        let width = (self.health * max_width) / self.max_health;
        if self.health < self.max_health {
            draw_line(x, y, x + width, y, 2.0, RED);
        }

        // show dammage taken
        if self.show_dmg_taken > 0.0 {
            let txt = format!("-{}", self.last_dmg_taken);
            let offset = SQUARE_SIZE / 2.0;
            let x = self.view_pos.x + offset;
            let y = self.view_pos.y;
            draw_text(txt, x, y, 20.0, RED);
        }
    }

    fn sprite<'b>(&self, units: &'b assets::Units) -> &'b assets::Sprite {
        let unit_kind = match self.unit_kind {
            UnitKind::Warrior => &units.warrior,
            UnitKind::Archer => &units.archer,
        };
        match self.action {
            Action::Idle => &unit_kind.idle,
            Action::Run => &unit_kind.run,
            Action::Attack(_) => &unit_kind.attack,
        }
    }

    fn get_available_moves(&self, game: &Game) -> (Vec<Cell>, HashMap<Cell, Cell>) {
        let mut available_moves = vec![];
        let x = self.game_pos[0];
        let y = self.game_pos[1];

        let mut came_from: HashMap<Cell, Cell> = HashMap::new();
        let mut dist: HashMap<Cell, usize> = HashMap::new();
        let mut q = VecDeque::new();
        let root = [x, y];
        dist.insert(root, 0);
        q.push_back(root);

        while let Some(pos) = q.pop_front() {
            let d = dist[&pos];
            if d > self.movement {
                continue;
            }
            available_moves.push(pos);

            let next_positions = [
                [pos[0], pos[1] + 1],               // up
                [pos[0], pos[1].saturating_sub(1)], // down
                [pos[0].saturating_sub(1), pos[1]], // left
                [pos[0] + 1, pos[1]],               // right
            ];

            for mut next_pos in next_positions {
                next_pos[0] = next_pos[0].clamp(0, NUM_SQUARES - 1);
                next_pos[1] = next_pos[1].clamp(0, NUM_SQUARES - 1);

                if dist.contains_key(&next_pos) {
                    continue;
                }
                let occupied = game.units.iter().find(|u| u.game_pos == next_pos).is_some()
                    || game
                        .enemy_units
                        .iter()
                        .find(|u| u.game_pos == next_pos)
                        .is_some();
                if occupied {
                    continue;
                }

                dist.insert(next_pos, d + 1);
                came_from.insert(next_pos, pos);
                q.push_back(next_pos);
            }
        }

        (available_moves, came_from)
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

fn new_unit(name: String, unit_kind: UnitKind, x: usize, y: usize) -> Unit {
    let mut movement = 5;
    let mut speed = 50.0;
    let mut range = 1;
    let mut max_health = 100.0;
    let mut attack = 50.0;

    match unit_kind {
        UnitKind::Warrior => {
            movement = 4;
            max_health = 200.0;
            attack = 100.0;
        }
        UnitKind::Archer => {
            range = 3;
            speed = 75.0;
        }
    }

    Unit {
        name,
        unit_kind,
        action: Action::Idle,
        game_pos: [x, y],
        view_pos: vec2(x as f32 * SQUARE_SIZE, y as f32 * SQUARE_SIZE),
        move_path: vec![],
        flip_x: false,
        movement,
        speed,
        range,
        max_health,
        health: max_health,
        attack,
        last_dmg_taken: 0.0,
        show_dmg_taken: 0.0,
    }
}
