use macroquad::prelude::*;

use std::collections::{HashMap, VecDeque};

mod assets;

pub const SQUARE_SIZE: f32 = 50.0;
const NUM_SQUARES: usize = 50;

pub const FPS: f32 = 10.0;

const HOVER_COLOR: Color = Color::from_rgba(255, 255, 255, 64);
const RED_HOVER_COLOR: Color = Color::from_rgba(255, 0, 0, 100);
const AVAILABLE_MOVES_COLOR: Color = Color::from_rgba(255, 255, 255, 80);
const SELECT_MOVE_COLOR: Color = Color::from_rgba(255, 255, 255, 200);

pub const TILEMAP: &[u8] =
    include_bytes!("../assets/tiny_swords/Terrain/Tileset/Tilemap_color1.png");

type Cell = [usize; 2];

#[derive(Debug, Clone, Copy, PartialEq)]
enum UnitKind {
    Warrior,
    Archer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Idle,
    Run,
    Attack((f32, usize)), // attack holds total animation time and enemy idx
}

#[derive(Debug, Clone)]
struct Unit {
    name: String,
    unit_kind: UnitKind,
    action: Action,
    game_pos: [usize; 2],
    pos: Vec2,
    move_path: Vec<Cell>,
    flip_x: bool,
    last_dmg_taken: f32,
    show_dmg_taken: f32,
    show_miss: f32,
    movement: usize,
    speed: f32,
    range: usize,
    max_health: f32,
    health: f32,
    attack: f32,
    defense: f32,
    accuracy: f32,
    projectile: Option<Projectile>,
}

#[derive(Debug, Clone)]
enum ProjectileKind {
    Arrow,
}

#[derive(Debug, Clone)]
struct Projectile {
    kind: ProjectileKind,
    start: Vec2,
    end: Vec2,
    t: f32,        // progress 0.0 -> 1.0
    duration: f32, // seconds of flight
    arc_height: f32,
}

#[derive(Default)]
struct Game {
    clouds: Vec<Vec2>,
    grass: Vec<Vec<bool>>,
    trees: Vec<Vec<bool>>,
    units: Vec<Unit>,
    enemy_units: Vec<Unit>,
    selected_unit: Option<usize>,
    mouse_pos: Vec2,
    total_frame_time: f32,
    hovering_unit: Option<usize>,
    hovering_enemy_unit: Option<usize>,
    available_moves: Vec<[usize; 2]>,
    came_from: HashMap<Cell, Cell>,
    hit_chance: f32,
}

#[macroquad::main("Rustic Warrior")]
async fn main() {
    set_fullscreen(true);

    let blue_units = assets::Units::new_blue();
    let red_units = assets::Units::new_red();
    let terrain = assets::Terrain::new();
    let projectiles = assets::Projectiles::new();

    let tiles = Texture2D::from_file_with_format(TILEMAP, None);
    tiles.set_filter(FilterMode::Nearest);

    let assets = assets::Assets {
        projectiles,
        blue_units,
        red_units,
        terrain,
        tiles,
    };

    let mut game = Game::default();
    game.hit_chance = 50.0;
    game.grass = generate_map();
    game.trees = generate_trees(&game.grass);
    game.clouds = generate_clouds(&game.grass);

    // generate units
    game.units.append(&mut generate_units(&game));
    game.enemy_units.append(&mut generate_enemy_units(&game));

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
            let pos = unit.pos;
            if self.mouse_in_pos(pos) {
                self.hovering_unit = Some(i);
            }
        }
        for (i, unit) in self.enemy_units.iter().enumerate() {
            let pos = unit.pos;
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
                    if unit.enemy_in_range(enemy) {
                        unit.action = Action::Attack((0.0, enemy_i));
                        if enemy.game_pos[0] > unit.game_pos[0] {
                            enemy.flip_x = true;
                        } else {
                            enemy.flip_x = false;
                        }
                        unit.flip_x = !enemy.flip_x;
                    }
                }
            }
            // set selected unit
            self.selected_unit = self.hovering_unit;
            self.available_moves.clear();
            self.came_from.clear();
        }

        let dt = get_frame_time();

        for (i, unit) in self.units.iter_mut().enumerate() {
            unit.update_pos(dt);
            if let Some(p) = &mut unit.projectile {
                if p.update(dt) {
                    unit.projectile = None;
                }
            }
            match unit.action {
                Action::Attack((anim_time, enemy_i)) => {
                    let enemy = &mut self.enemy_units[enemy_i];

                    let sprite = unit.sprite(&assets.blue_units);
                    let frame = anim_time * FPS;
                    if frame as usize == 5 {
                        if unit.projectile.is_none() {
                            if unit.unit_kind == UnitKind::Archer {
                                let projectile =
                                    Projectile::new(ProjectileKind::Arrow, unit.pos, enemy.pos);
                                unit.projectile = Some(projectile);
                            }
                        }
                    }
                    if frame > sprite.frames {
                        unit.action = Action::Idle;
                        // check hit_chance
                        let mut hit = false;
                        self.hit_chance += unit.accuracy;
                        if self.hit_chance >= 100.0 {
                            hit = true;
                            self.hit_chance -= 100.0;
                        }

                        if hit {
                            // update health/despawn after the animation finishes
                            let dmg = (unit.attack - enemy.defense).max(0.0);
                            enemy.health -= dmg;
                            enemy.show_dmg_taken = 1.0;
                            enemy.last_dmg_taken = dmg;
                        } else {
                            enemy.show_miss = 1.0;
                        }

                        // despawn or counter attack
                        if enemy.health <= 0.0 {
                            self.enemy_units.remove(enemy_i);
                            self.hovering_enemy_unit = None;
                        } else {
                            // if in range, counter attack
                            if enemy.enemy_in_range(unit) {
                                enemy.action = Action::Attack((0.0, i));
                                if enemy.game_pos[0] > unit.game_pos[0] {
                                    enemy.flip_x = true;
                                } else {
                                    enemy.flip_x = false;
                                }
                                unit.flip_x = !enemy.flip_x;
                            }
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
            if unit.show_dmg_taken > 0.0 {
                unit.show_dmg_taken -= dt;
            }
            if unit.show_miss > 0.0 {
                unit.show_miss -= dt;
            }
        }

        for unit in &mut self.enemy_units {
            unit.update_pos(dt);
            if let Some(p) = &mut unit.projectile {
                if p.update(dt) {
                    unit.projectile = None;
                }
            }
            match unit.action {
                Action::Attack((anim_time, enemy_i)) => {
                    let enemy = &mut self.units[enemy_i];

                    let sprite = unit.sprite(&assets.blue_units);
                    let frame = anim_time * FPS;
                    if frame as usize == 5 {
                        if unit.projectile.is_none() {
                            if unit.unit_kind == UnitKind::Archer {
                                let projectile =
                                    Projectile::new(ProjectileKind::Arrow, unit.pos, enemy.pos);
                                unit.projectile = Some(projectile);
                            }
                        }
                    }
                    if frame > sprite.frames {
                        unit.action = Action::Idle;
                        // check hit_chance
                        let mut hit = false;
                        self.hit_chance += unit.accuracy;
                        if self.hit_chance >= 100.0 {
                            hit = true;
                            self.hit_chance -= 100.0;
                        }

                        // update health/despawn after the animation finishes
                        if hit {
                            let dmg = (unit.attack - enemy.defense).max(0.0);
                            enemy.health -= dmg;
                            enemy.show_dmg_taken = 1.0;
                            enemy.last_dmg_taken = dmg;
                        } else {
                            enemy.show_miss = 1.0;
                        }

                        if enemy.health <= 0.0 {
                            self.units.remove(enemy_i);
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
            if unit.show_dmg_taken > 0.0 {
                unit.show_dmg_taken -= dt;
            }
            if unit.show_miss > 0.0 {
                unit.show_miss -= dt;
            }
        }
    }
    fn draw(&self, assets: &assets::Assets) {
        // draw grass
        for y in 0..NUM_SQUARES {
            for x in 0..NUM_SQUARES {
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

        // draw trees
        for y in 0..NUM_SQUARES {
            for x in 0..NUM_SQUARES {
                let px = x as f32 * SQUARE_SIZE;
                let py = y as f32 * SQUARE_SIZE;
                if !self.grass[y][x] || !self.trees[y][x] {
                    continue;
                }
                assets
                    .terrain
                    .tree1
                    .draw(self.total_frame_time, px, py, false, false);
            }
        }

        // draw clouds
        for (i, cloud) in self.clouds.iter().enumerate() {
            if i % 5 == 0 {
                assets
                    .terrain
                    .cloud4
                    .draw(self.total_frame_time, cloud.x, cloud.y, false, false);
            } else if i % 4 == 0 {
                assets
                    .terrain
                    .cloud3
                    .draw(self.total_frame_time, cloud.x, cloud.y, false, false);
            } else if i % 3 == 0 {
                assets
                    .terrain
                    .cloud2
                    .draw(self.total_frame_time, cloud.x, cloud.y, false, false);
            } else {
                assets
                    .terrain
                    .cloud1
                    .draw(self.total_frame_time, cloud.x, cloud.y, false, false);
            }
        }

        // show available moves for selected unit
        if let Some(i) = self.selected_unit {
            let unit = &self.units[i];
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
                if unit.enemy_in_range(enemy) {
                    draw_rectangle(
                        enemy.pos.x,
                        enemy.pos.y,
                        SQUARE_SIZE,
                        SQUARE_SIZE,
                        RED_HOVER_COLOR,
                    );
                }
            }
        }

        // highlight hovered unit
        if let Some(i) = self.hovering_unit {
            let unit = &self.units[i];
            draw_rectangle(
                unit.pos.x,
                unit.pos.y,
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

            // draw projectile
            if let Some(p) = &unit.projectile {
                let sprite = p.sprite(&assets.projectiles);
                p.draw(&sprite);
            }
        }

        for (i, unit) in self.enemy_units.iter().enumerate() {
            let mut hovered = false;
            if let Some(idx) = self.hovering_enemy_unit {
                if idx == i {
                    hovered = true;
                }
            }
            unit.draw(self.total_frame_time, hovered, &assets.red_units);

            // draw projectile
            if let Some(p) = &unit.projectile {
                let sprite = p.sprite(&assets.projectiles);
                p.draw(&sprite);
            }
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
                "{}\nHealth: {}/{}\nAttack: {}\nDefense: {}\nRange: {}\nAccuracy: {}\nMove: {}",
                u.name,
                u.health,
                u.max_health,
                u.attack,
                u.defense,
                u.range,
                u.accuracy,
                u.movement
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

    fn cell_is_occupied(&self, cell: Cell) -> bool {
        let x = cell[0];
        let y = cell[1];
        if !self.grass[y][x] {
            return true;
        }
        if self.trees[y][x] {
            return true;
        }
        if self.units.iter().find(|u| u.game_pos == cell).is_some() {
            return true;
        }
        if self
            .enemy_units
            .iter()
            .find(|u| u.game_pos == cell)
            .is_some()
        {
            return true;
        }

        false
    }
}

impl Unit {
    fn enemy_in_range(&self, enemy: &Unit) -> bool {
        let [unit_x, unit_y] = self.game_pos;
        let [enemy_x, enemy_y] = enemy.game_pos;
        let x_diff = unit_x.abs_diff(enemy_x);
        let y_diff = unit_y.abs_diff(enemy_y);
        let distance = x_diff + y_diff;
        if distance <= self.range {
            return true;
        }

        false
    }
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

                let to_target = target_pos - self.pos;
                let step = self.speed * dt;

                if to_target.length() <= step {
                    self.action = Action::Idle;
                    self.pos = target_pos;
                } else {
                    self.action = Action::Run;
                    let dir = to_target.normalize();
                    self.pos += dir * step;
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
                self.sprite(units)
                    .draw(anim_time, self.pos.x, self.pos.y, self.flip_x, true);
            }
            _ => {
                self.sprite(units)
                    .draw(t, self.pos.x, self.pos.y, self.flip_x, false);
            }
        }

        // draw health bar
        let offset = 5.0;
        let x = self.pos.x + offset;
        let y = self.pos.y + SQUARE_SIZE;
        let max_width = SQUARE_SIZE - (offset * 2.0);
        let width = (self.health * max_width) / self.max_health;
        if self.health < self.max_health {
            draw_line(x, y, x + width, y, 2.0, RED);
        }

        // show damage taken
        if self.show_dmg_taken > 0.0 {
            let txt = format!("-{}", self.last_dmg_taken);
            let offset = SQUARE_SIZE / 2.0;
            let x = self.pos.x + offset;
            let y = self.pos.y;
            draw_text(txt, x, y, 20.0, RED);
        }

        // show miss
        if self.show_miss > 0.0 {
            let offset = SQUARE_SIZE / 2.0;
            let x = self.pos.x + offset;
            let y = self.pos.y;
            draw_text("MISS", x, y, 20.0, RED);
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
                if game.cell_is_occupied(next_pos) {
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

impl Projectile {
    fn sprite<'b>(&self, projectiles: &'b assets::Projectiles) -> &'b assets::Sprite {
        match self.kind {
            ProjectileKind::Arrow => &projectiles.arrow,
        }
    }

    fn new(kind: ProjectileKind, start: Vec2, end: Vec2) -> Self {
        let dist = start.distance(end);
        Projectile {
            kind,
            start,
            end,
            t: 0.0,
            duration: dist / 400.0,  // 400 px/s, so far shots take longer
            arc_height: dist * 0.25, // higher arc for longer shots
        }
    }

    fn pos_at(&self, t: f32) -> Vec2 {
        let flat = self.start.lerp(self.end, t);
        // parabola: 0 at t=0 and t=1, peaks at t=0.5
        let lift = 4.0 * t * (1.0 - t) * self.arc_height;
        vec2(flat.x, flat.y - lift) // minus = up on screen (y points down)
    }

    fn update(&mut self, dt: f32) -> bool {
        self.t = (self.t + dt / self.duration).min(1.0);
        self.t >= 1.0 // true when it has landed
    }

    fn draw(&self, sprite: &assets::Sprite) {
        let pos = self.pos_at(self.t);
        // direction = where it will be a moment later, minus where it is now
        let ahead = self.pos_at((self.t + 0.01).min(1.0));
        let dir = ahead - pos;
        let angle = dir.y.atan2(dir.x);

        let scale = sprite.scale;
        let size = SQUARE_SIZE * scale;
        let offset = (size - SQUARE_SIZE) / 2.0;

        draw_texture_ex(
            &sprite.tex,
            pos.x - offset,
            pos.y - offset,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                rotation: angle, // assumes the sprite points right
                ..Default::default()
            },
        );
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
    let mut defense = 10.0;
    let mut accuracy = 70.0;

    match unit_kind {
        UnitKind::Warrior => {
            movement = 4;
            max_health = 200.0;
            attack = 100.0;
            defense = 20.0;
        }
        UnitKind::Archer => {
            range = 3;
            speed = 75.0;
            accuracy = 90.0;
        }
    }

    Unit {
        name,
        unit_kind,
        action: Action::Idle,
        game_pos: [x, y],
        pos: vec2(x as f32 * SQUARE_SIZE, y as f32 * SQUARE_SIZE),
        move_path: vec![],
        flip_x: false,
        last_dmg_taken: 0.0,
        show_dmg_taken: 0.0,
        show_miss: 0.0,
        movement,
        speed,
        range,
        max_health,
        health: max_health,
        attack,
        defense,
        accuracy,
        projectile: None,
    }
}

use std::time::SystemTime;

fn generate_map() -> Vec<Vec<bool>> {
    let mut map = vec![vec![false; NUM_SQUARES]; NUM_SQUARES];
    let center = NUM_SQUARES as f32 / 2.0;
    let base_radius = center * 0.75;

    // Generate random offsets based on current time so the shape changes every time
    let time_seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as f32;
    let offset_x = (time_seed * 0.0001).sin() * 10.0;
    let offset_y = (time_seed * 0.0001).cos() * 10.0;

    for y in 0..NUM_SQUARES {
        for x in 0..NUM_SQUARES {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            let norm_dist = distance / center;

            if norm_dist > 1.0 {
                continue;
            }

            let nx = (x as f32 / NUM_SQUARES as f32 * 6.0) + offset_x;
            let ny = (y as f32 / NUM_SQUARES as f32 * 6.0) + offset_y;
            let noise = (nx * 3.5 + ny * 2.2).sin() * 6.0 + (nx * 7.1 - ny * 6.4).cos() * 3.0;
            let falloff = 1.0 - norm_dist * norm_dist;

            if distance < base_radius + noise * falloff {
                map[y][x] = true; // Land
            }
        }
    }
    map
}

fn generate_trees(map: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let mut trees = vec![vec![false; NUM_SQUARES]; NUM_SQUARES];

    // Unique tree seed per generation call
    let tree_seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u32;

    for y in 0..NUM_SQUARES {
        for x in 0..NUM_SQUARES {
            if !map[y][x] || y + 1 >= NUM_SQUARES || !map[y + 1][x] {
                continue;
            }

            // Include the tree_seed in the hash so tree positions change every time
            let hash = ((x as u32).wrapping_mul(374761393))
                ^ ((y as u32).wrapping_mul(668265263))
                ^ tree_seed;
            let random_val = (hash % 1000) as f32 / 1000.0;

            if random_val < 0.03 {
                trees[y][x] = true;
            }
        }
    }
    trees
}

fn generate_clouds(map: &[Vec<bool>]) -> Vec<Vec2> {
    let mut clouds = vec![];

    for y in 0..NUM_SQUARES {
        for x in 0..NUM_SQUARES {
            if map[y][x] {
                continue;
            }

            // Pseudo-random integer hash for true, non-patterned scattering
            let hash = ((x as u32).wrapping_mul(374761393)) ^ ((y as u32).wrapping_mul(668265263));
            let random_val = hash as f32 / u32::MAX as f32;

            // 3% chance for sparse, natural distribution
            if random_val < 0.02 {
                clouds.push(vec2(x as f32 * SQUARE_SIZE, y as f32 * SQUARE_SIZE));
            }
        }
    }

    clouds
}

fn generate_units(game: &Game) -> Vec<Unit> {
    let mut units = vec![];
    let mut left_tiles = vec![];

    let left_limit = (NUM_SQUARES / 4).max(5);

    for y in 0..NUM_SQUARES {
        for x in 0..left_limit {
            if game.grass[y][x] {
                left_tiles.push((x, y));
            }
        }
    }

    let step = if left_tiles.len() >= 10 {
        left_tiles.len() / 10
    } else {
        1
    };
    let mut index = 0;

    // Interleaved array containing 5 Warriors and 5 Archers
    let kinds = [
        UnitKind::Warrior,
        UnitKind::Archer,
        UnitKind::Warrior,
        UnitKind::Archer,
        UnitKind::Warrior,
        UnitKind::Archer,
        UnitKind::Archer,
        UnitKind::Warrior,
        UnitKind::Archer,
        UnitKind::Warrior,
    ];

    for kind in kinds {
        if let Some(&(x, y)) = left_tiles.get(index) {
            let u = new_unit("Player".to_string(), kind, x, y);
            units.push(u);
            index = (index + step) % left_tiles.len();
        }
    }

    units
}

fn generate_enemy_units(game: &Game) -> Vec<Unit> {
    let mut units = vec![];
    let mut right_tiles = vec![];

    let right_limit = NUM_SQUARES - (NUM_SQUARES / 4).max(5);

    for y in 0..NUM_SQUARES {
        for x in right_limit..NUM_SQUARES {
            if game.grass[y][x] {
                right_tiles.push((x, y));
            }
        }
    }

    let step = if right_tiles.len() >= 10 {
        right_tiles.len() / 10
    } else {
        1
    };
    let mut index = 0;

    let kinds = [
        UnitKind::Warrior,
        UnitKind::Archer,
        UnitKind::Warrior,
        UnitKind::Archer,
        UnitKind::Warrior,
        UnitKind::Archer,
        UnitKind::Archer,
        UnitKind::Warrior,
        UnitKind::Archer,
        UnitKind::Warrior,
    ];

    for kind in kinds {
        if let Some(&(x, y)) = right_tiles.get(index) {
            let mut u = new_unit("Enemy".to_string(), kind, x, y);
            u.flip_x = true;
            units.push(u);
            index = (index + step) % right_tiles.len();
        }
    }

    units
}
