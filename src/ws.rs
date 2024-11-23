use std::net::TcpStream;
use macroquad::math::Vec2;
use macroquad::ui::Drag::No;
use tungstenite::{stream::MaybeTlsStream, WebSocket, client::connect, Message, Error};

use shared::{
    ClientMessage,
};

pub fn client_send(
    msg: &ClientMessage,
    connection: &mut Connection,
) {
    let bytes = serde_json::to_vec(msg).expect("serialization failed");
    connection.send(bytes);
}
//реализация отправки сообщения, где мы сериализуем данные

pub struct Connection {
    socket: Option<WebSocket<MaybeTlsStream<TcpStream>>>,
} //определили структуру подключения

impl Connection {
    pub fn new() -> Self {
        Self { socket: None }
    }

    pub fn connect(&mut self, url: &str) {
        if let Ok((mut socket, _)) = connect(url) {
            if let MaybeTlsStream::Plain(s) = socket.get_mut() {
                s.set_nonblocking(true).unwrap();
            }

            self.socket = Some(socket);
        }
    }

    pub fn poll(&mut self) -> Option<Vec<u8>> {
        if let Some(socket) = &mut self.socket {
            match socket.read_message() {
                Ok(msg) => {
                    if let Message::Binary(buf) = msg {
                        return Some(buf);
                    }
                    // Обработка других типов сообщений при необходимости
                },
                Err(Error::Io(ref err)) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    // Нет доступных данных для чтения
                },
                Err(Error::ConnectionClosed) => {
                    // Соединение закрыто
                    eprintln!("WebSocket соединение закрыто");
                    self.socket = None;
                },
                Err(e) => {
                    eprintln!("Ошибка при чтении WebSocket: {:?}", e);
                    self.socket = None;
                }
            }
        }
        None
    }

    pub fn send(&mut self, msg: Vec<u8>) {
        if let Some(socket) = &mut self.socket {
            match socket.write_message(Message::Binary(msg)) {
                Ok(_) => {},
                Err(Error::Io(ref err)) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    // Сокет не готов для записи, можно обработать или проигнорировать
                },
                Err(e) => {
                    eprintln!("Ошибка при записи в WebSocket: {:?}", e);
                    self.socket = None;
                }
            }
        }
    }
}


