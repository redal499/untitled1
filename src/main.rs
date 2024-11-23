// main.rs

mod ws;

use std::ops::{AddAssign, SubAssign};
use macroquad::color::{BLACK, WHITE};
use macroquad::color_u8;
use macroquad::input::{is_key_down, KeyCode};
use macroquad::math::{Rect, Vec2};
use macroquad::window::{clear_background, next_frame, screen_width, screen_height};
use macroquad::prelude::{draw_texture_ex, load_texture, Color, DrawTextureParams, Texture2D};
use macroquad::shapes::draw_rectangle;
use std::default::Default as stdDefault;
use shared::{ClientMessage, RemoteState, ServerMessage, State};
use glam::Vec2 as glam_Vec2;

use get_if_addrs::get_if_addrs; // Для получения локального IP

pub fn client_send(
    msg: &ClientMessage,
    connection: &mut ws::Connection,
) {
    let bytes = serde_json::to_vec(msg).expect("serialization failed");
    connection.send(bytes);
}

const PLANE_WIDTH: f32 = 180.;
const PLANE_HEIGHT: f32 = 180.;

// Определение структуры игры
pub struct Game {
    pub quit: bool,
    pub texture: Texture2D,
    remote_states: Vec<RemoteState>,
    player_state: RemoteState,
}

impl Game {
    pub fn draw_plane(&self, state: &RemoteState) {
        let cols = (self.texture.width() / PLANE_WIDTH).floor() as usize;
        let index = state.id % 10;
        let tx_x = index % cols;
        let tx_y = index / cols;

        draw_texture_ex(
            &self.texture,
            state.position.x,
            state.position.y,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    tx_x as f32 * PLANE_WIDTH,
                    tx_y as f32 * PLANE_HEIGHT,
                    PLANE_WIDTH,
                    PLANE_HEIGHT,
                )),
                rotation: state.rotation,
                ..stdDefault::default()
            },
        )
    }

    pub fn handle_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Welcome(id) => {
                self.player_state.id = id;
            }
            ServerMessage::GoodBye(id) => {
                self.remote_states.retain(|s| s.id != id);
            }
            ServerMessage::Update(remote_states) => {
                self.remote_states = remote_states;
            }
        }
    }

    pub async fn new() -> Self {
        let texture = load_texture("assets/planes.png").await.unwrap();
        Self {
            quit: false,
            player_state: RemoteState {
                id: 0,
                position: glam_Vec2::new(100f32, 100f32),
                rotation: 0f32,
            },
            texture,
            remote_states: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        if is_key_down(KeyCode::Escape) {
            self.quit = true;
        }

        const ROT_SPEED: f32 = 0.015;
        if is_key_down(KeyCode::Right) {
            self.player_state.rotation += ROT_SPEED;
        }
        if is_key_down(KeyCode::Left) {
            self.player_state.rotation -= ROT_SPEED;
        }

        const SPEED: f32 = 0.6;

        self.player_state.position +=
            vec2_from_angle(self.player_state.rotation) * SPEED;
        for state in &mut self.remote_states {
            state.position += vec2_from_angle(state.rotation) * SPEED;
        }

        // Циклическое перемещение игрока по экрану
        if self.player_state.position.x > screen_width() {
            self.player_state.position.x = -PLANE_WIDTH;
        } else if self.player_state.position.x < -PLANE_WIDTH {
            self.player_state.position.x = screen_width();
        }

        if self.player_state.position.y > screen_height() {
            self.player_state.position.y = -PLANE_HEIGHT;
        } else if self.player_state.position.y < -PLANE_HEIGHT {
            self.player_state.position.y = screen_height();
        }
    }

    pub fn draw(&self) {
        clear_background(color_u8!(0, 211, 205, 205));

        draw_box(Vec2::new(200f32, 200f32), Vec2::new(10f32, 10f32));

        self.draw_plane(&self.player_state);

        for state in &self.remote_states {
            self.draw_plane(state);
        }
    }
}

fn draw_box(pos: Vec2, size: Vec2) {
    let dimension = size * 2.;
    let upper_left = pos - size;

    draw_rectangle(
        upper_left.x,
        upper_left.y,
        dimension.x,
        dimension.y,
        BLACK,
    );
}

fn vec2_from_angle(angle: f32) -> glam::Vec2 {
    let angle = angle - std::f32::consts::FRAC_PI_2;
    glam::Vec2::new(angle.cos(), angle.sin())
}

// Функция для получения локального IPv4 адреса
fn get_local_ipv4() -> String {
    let addrs = get_if_addrs().unwrap();
    for iface in addrs {
        if !iface.is_loopback() {
            match iface.addr.ip() {
                std::net::IpAddr::V4(ipv4) => return ipv4.to_string(),
                _ => continue,
            }
        }
    }
    "0.0.0.0".to_string()
}

#[macroquad::main("Test Game")]
async fn main() {
    let mut connection = ws::Connection::new();
    connection.connect("ws://127.0.0.1:3030/game");

    // Получение локального IP и отправка сообщения Register
    let local_ip = get_local_ipv4();
    println!("Локальный IP-адрес: {}", local_ip);
    let register_msg = ClientMessage::Register { ip: local_ip.clone() };
    client_send(&register_msg, &mut connection);

    let mut game = Game::new().await;

    loop {
        let state = ClientMessage::State(State {
            pos: game.player_state.position,
            r: game.player_state.rotation,
        });
        client_send(&state, &mut connection);

        if let Some(msg) = connection.poll() {
            let msg: ServerMessage = serde_json::from_slice(msg.as_slice())
                .expect("deserialization failed");
            game.handle_message(msg);
        }

        game.update();
        game.draw();
        if game.quit {
            return;
        }
        next_frame().await;
    }
}