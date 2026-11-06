use macroquad::prelude::*;

use crate::{FPS, SQUARE_SIZE};

// Blue units
pub const BLUE_WARRIOR_IDLE: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Blue Units/Warrior/Warrior_Idle.png");

pub const BLUE_WARRIOR_ATTACK1: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Blue Units/Warrior/Warrior_Attack1.png");

pub const BLUE_WARRIOR_RUN: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Blue Units/Warrior/Warrior_Run.png");

pub const BLUE_ARCHER_IDLE: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Blue Units/Archer/Archer_Idle.png");

pub const BLUE_ARCHER_ATTACK: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Blue Units/Archer/Archer_Shoot.png");

pub const BLUE_ARCHER_RUN: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Blue Units/Archer/Archer_Run.png");

// Red units
pub const RED_WARRIOR_IDLE: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Red Units/Warrior/Warrior_Idle.png");

pub const RED_WARRIOR_ATTACK1: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Red Units/Warrior/Warrior_Attack1.png");

pub const RED_WARRIOR_RUN: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Red Units/Warrior/Warrior_Run.png");

pub const RED_ARCHER_IDLE: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Red Units/Archer/Archer_Idle.png");

pub const RED_ARCHER_ATTACK: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Red Units/Archer/Archer_Shoot.png");

pub const RED_ARCHER_RUN: &[u8] =
    include_bytes!("../assets/tiny_swords/Units/Red Units/Archer/Archer_Run.png");

// Terrain
pub const TREE1: &[u8] =
    include_bytes!("../assets/tiny_swords/Terrain/Resources/Wood/Trees/Tree1.png");

pub const WATER_TILE: &[u8] =
    include_bytes!("../assets/tiny_swords/Terrain/Tileset/Water Background color.png");

// Decorations
pub const CLOUD1: &[u8] =
    include_bytes!("../assets/tiny_swords/Terrain/Decorations/Clouds/Clouds_01.png");

pub const CLOUD2: &[u8] =
    include_bytes!("../assets/tiny_swords/Terrain/Decorations/Clouds/Clouds_02.png");

pub const CLOUD3: &[u8] =
    include_bytes!("../assets/tiny_swords/Terrain/Decorations/Clouds/Clouds_03.png");

pub const CLOUD4: &[u8] =
    include_bytes!("../assets/tiny_swords/Terrain/Decorations/Clouds/Clouds_04.png");

pub struct Terrain {
    pub tree1: Sprite,
    pub cloud1: Sprite,
    pub cloud2: Sprite,
    pub cloud3: Sprite,
    pub cloud4: Sprite,
    pub water: Sprite,
}

pub struct Units {
    pub warrior: Unit,
    pub archer: Unit,
}

pub struct Unit {
    pub idle: Sprite,
    pub attack: Sprite,
    pub run: Sprite,
}

#[derive(Debug)]
pub struct Sprite {
    tex: Texture2D,
    frames: f32,
    frame_width: f32,
    frame_height: f32,
    scale: f32,
}

pub struct Assets {
    pub blue_units: Units,
    pub red_units: Units,
    pub terrain: Terrain,
    pub tiles: Texture2D,
}

impl Terrain {
    pub fn new() -> Self {
        // tree1
        let tex = Texture2D::from_file_with_format(TREE1, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 8.0;
        let tree1 = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 1536.0 / frames,
            frame_height: 256.0,
        };

        // cloud1
        let tex = Texture2D::from_file_with_format(CLOUD1, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 1.0;
        let cloud1 = Sprite {
            tex,
            frames,
            scale: 3.0,
            frame_width: 576.0 / frames,
            frame_height: 256.0,
        };

        // cloud2
        let tex = Texture2D::from_file_with_format(CLOUD2, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 1.0;
        let cloud2 = Sprite {
            tex,
            frames,
            scale: 3.0,
            frame_width: 576.0 / frames,
            frame_height: 256.0,
        };

        // cloud3
        let tex = Texture2D::from_file_with_format(CLOUD3, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 1.0;
        let cloud3 = Sprite {
            tex,
            frames,
            scale: 3.0,
            frame_width: 576.0 / frames,
            frame_height: 256.0,
        };

        // cloud4
        let tex = Texture2D::from_file_with_format(CLOUD4, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 1.0;
        let cloud4 = Sprite {
            tex,
            frames,
            scale: 3.0,
            frame_width: 576.0 / frames,
            frame_height: 256.0,
        };

        // water
        let tex = Texture2D::from_file_with_format(WATER_TILE, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 1.0;
        let water = Sprite {
            tex,
            frames,
            scale: 1.0,
            frame_width: 64.0 / frames,
            frame_height: 64.0,
        };

        Terrain {
            tree1,
            cloud1,
            cloud2,
            cloud3,
            cloud4,
            water,
        }
    }
}

impl Units {
    pub fn new_blue() -> Self {
        // idle
        let tex = Texture2D::from_file_with_format(BLUE_WARRIOR_IDLE, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 8.0;
        let idle = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 1536.0 / frames,
            frame_height: 192.0,
        };

        // attack
        let tex = Texture2D::from_file_with_format(BLUE_WARRIOR_ATTACK1, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 4.0;
        let attack = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 768.0 / frames,
            frame_height: 192.0,
        };

        // run
        let tex = Texture2D::from_file_with_format(BLUE_WARRIOR_RUN, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 6.0;
        let run = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 1152.0 / frames,
            frame_height: 192.0,
        };

        let warrior = Unit { idle, attack, run };

        // archer idle
        let tex = Texture2D::from_file_with_format(BLUE_ARCHER_IDLE, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 6.0;
        let idle = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 1152.0 / frames,
            frame_height: 192.0,
        };

        // archer attack
        let tex = Texture2D::from_file_with_format(BLUE_ARCHER_ATTACK, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 8.0;
        let attack = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 1536.0 / frames,
            frame_height: 192.0,
        };

        // archer run
        let tex = Texture2D::from_file_with_format(BLUE_ARCHER_RUN, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 4.0;
        let run = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 768.0 / frames,
            frame_height: 192.0,
        };

        let archer = Unit { idle, attack, run };

        Units { warrior, archer }
    }

    pub fn new_red() -> Self {
        // warrior idle
        let tex = Texture2D::from_file_with_format(RED_WARRIOR_IDLE, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 8.0;
        let idle = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 1536.0 / frames,
            frame_height: 192.0,
        };

        // warrior attack
        let tex = Texture2D::from_file_with_format(RED_WARRIOR_ATTACK1, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 4.0;
        let attack = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 768.0 / frames,
            frame_height: 192.0,
        };

        // warrior run
        let tex = Texture2D::from_file_with_format(RED_WARRIOR_RUN, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 6.0;
        let run = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 1152.0 / frames,
            frame_height: 192.0,
        };

        let warrior = Unit { idle, attack, run };

        // archer idle
        let tex = Texture2D::from_file_with_format(RED_ARCHER_IDLE, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 6.0;
        let idle = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 1152.0 / frames,
            frame_height: 192.0,
        };

        // archer attack
        let tex = Texture2D::from_file_with_format(RED_ARCHER_ATTACK, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 8.0;
        let attack = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 1536.0 / frames,
            frame_height: 192.0,
        };

        // archer run
        let tex = Texture2D::from_file_with_format(RED_ARCHER_RUN, None);
        tex.set_filter(FilterMode::Nearest);
        let frames = 4.0;
        let run = Sprite {
            tex,
            frames,
            scale: 2.0,
            frame_width: 768.0 / frames,
            frame_height: 192.0,
        };

        let archer = Unit { idle, attack, run };

        Units { warrior, archer }
    }
}

impl Sprite {
    pub fn draw(&self, t: f32, x: f32, y: f32, flip_x: bool) {
        let frame = ((t * FPS) as usize) % self.frames as usize;
        let scale = self.scale;
        let size = SQUARE_SIZE * scale;
        let offset = (size - SQUARE_SIZE) / 2.0;

        draw_texture_ex(
            &self.tex,
            x - offset,
            y - offset,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    frame as f32 * self.frame_width,
                    0.0,
                    self.frame_width,
                    self.frame_height,
                )),
                dest_size: Some(vec2(size, size)),
                flip_x,
                ..Default::default()
            },
        );
    }
}
