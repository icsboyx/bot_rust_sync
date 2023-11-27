macro_rules! create_twitch_connection {
    ($stream:expr, $callbacks:expr) => {
        &mut TwitchConnection {
            stream: $stream.try_clone().unwrap(),
            callbacks: $callbacks.clone(),
        }
    };
}

use crate::config::Config;
use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Debug, Default)]
pub struct Context {
    pub sender: String,
    pub command: String,
    pub receiver: String,
}

#[derive(Debug, Default)]
pub struct IRCMessage {
    pub tags: String,
    pub context: Context,
    pub message: String,
}
#[derive(Debug, Default)]
pub struct TwitchCallbacks {
    pub ping_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
    pub privmsg_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
    pub whisper_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
    pub custom_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
}

pub enum TwitchCapabilities {
    Tags,
    Commands,
    Membership,
}
impl TwitchCapabilities {
    fn request(&self) -> String {
        match self {
            TwitchCapabilities::Tags => "CAP REQ :twitch.tv/tags".to_string(),
            TwitchCapabilities::Commands => "CAP REQ :twitch.tv/commands".to_string(),
            TwitchCapabilities::Membership => "CAP REQ :twitch.tv/membership".to_string(),
        }
    }
}

pub struct TwitchConnection {
    stream: TcpStream,
    pub callbacks: Arc<Mutex<TwitchCallbacks>>,
}

impl TwitchConnection {
    pub fn new(server_address: String) -> Self {
        let server_address_port = server_address.to_string();
        let stream = std::net::TcpStream::connect(server_address_port).unwrap();
        let stream_int = stream.try_clone().unwrap();
        let callbacks = Arc::new(Mutex::new(TwitchCallbacks {
            ..Default::default()
        }));

        let stream_int_callback = stream_int.try_clone().unwrap();
        let callbacks_int = callbacks.clone();

        thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stream_int);
            let mut buffer = vec![0; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        for line in String::from_utf8_lossy(&buffer[0..n - 2]).split("\r\n") {
                            println!("[Twitch] RAW: {}", line);
                            let message = Self::parse_twitch_message(line);

                            if callbacks_int.lock().unwrap().custom_callback.is_some() {
                                callbacks_int
                                    .lock()
                                    .unwrap()
                                    .custom_callback
                                    .as_ref()
                                    .unwrap()(
                                    create_twitch_connection!(stream_int_callback, callbacks_int),
                                    &message,
                                );
                            }
                            match message.context.command.as_str() {
                                "PRIVMSG" => {
                                    if callbacks_int.lock().unwrap().privmsg_callback.is_some() {
                                        callbacks_int
                                            .lock()
                                            .unwrap()
                                            .privmsg_callback
                                            .as_ref()
                                            .unwrap()(
                                            create_twitch_connection!(
                                                stream_int_callback,
                                                callbacks_int
                                            ),
                                            &message,
                                        );
                                    }
                                }
                                "PING" => {
                                    if callbacks_int.lock().unwrap().ping_callback.is_some() {
                                        callbacks_int
                                            .lock()
                                            .unwrap()
                                            .ping_callback
                                            .as_ref()
                                            .unwrap()(
                                            create_twitch_connection!(
                                                stream_int_callback,
                                                callbacks_int
                                            ),
                                            &message,
                                        );
                                    }
                                }
                                "WHISPER" => {
                                    if callbacks_int.lock().unwrap().whisper_callback.is_some() {
                                        callbacks_int
                                            .lock()
                                            .unwrap()
                                            .whisper_callback
                                            .as_ref()
                                            .unwrap()(
                                            create_twitch_connection!(
                                                stream_int_callback,
                                                callbacks_int
                                            ),
                                            &message,
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        println!("[Twitch] Error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        Self { stream, callbacks }
    }
    pub fn server_auth(&mut self, password: &str, username: &str) {
        self.send_message(format!("PASS oauth:{}", password).as_str());
        self.send_message(format!("NICK {}", username).as_str());
    }
    pub fn join_channel(&mut self, channel: &str) {
        self.send_message(format!("JOIN #{}", channel).as_str());
    }

    pub fn send_message(&mut self, message: &str) {
        println!("[BOT] Sending: {}", message);
        let _ = self
            .stream
            .write(format!("{}\n\r", message).as_bytes())
            .unwrap();
    }

    pub fn keep_alive(&mut self, interval: f32) {
        let mut stream = self.stream.try_clone().unwrap();
        thread::spawn(move || loop {
            println!("[BOT] Sending: PING");
            stream.write_all("PING \r\n".as_bytes()).unwrap();
            thread::sleep(Duration::from_secs(interval as u64));
        });
    }
    fn parse_twitch_message(message: &str) -> IRCMessage {
        let mut msg = IRCMessage {
            ..Default::default()
        };
        let mut message_split = message
            .splitn(3, ':')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        if message_split.len() < 3 && !message.starts_with(':') {
            msg.context.command = message_split.get(0).unwrap_or(&"".to_string()).to_owned();
            msg.context.sender = message_split.get(1).unwrap_or(&"".to_string()).to_owned();
            msg.context.receiver = "*".to_string();
            return msg;
        }
        if message_split.len() < 3 && message.starts_with(':') {
            message_split.push("".to_string());
        }

        if message_split.len() < 3 && message_split[0].is_empty() {
            return msg;
        }

        msg.tags = message_split.get(0).unwrap_or(&"".to_string()).to_owned();

        msg.context = message_split
            .get(1) // Use the second element or None if no elements
            .map(|s| {
                let mut context = Context {
                    ..Default::default()
                };
                let mut split = s.split(' ');
                context.sender = split
                    .next()
                    .unwrap_or_default()
                    .to_owned()
                    .split('!')
                    .next()
                    .unwrap_or_default()
                    .to_owned();
                context.command = split.next().unwrap_or_default().to_owned();
                context.receiver = split.next().unwrap_or_default().to_owned();
                context
            })
            .unwrap_or_default();

        msg.message = message_split.get(2).unwrap_or(&"".to_string()).to_owned();

        msg
    }
    pub fn request_capabilities(&mut self, capabilities: Vec<TwitchCapabilities>) {
        for capability in capabilities.iter() {
            self.send_message(capability.request().as_str());
        }
    }
}
