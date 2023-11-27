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
struct Context {
    sender: String,
    command: String,
    receiver: String,
}

#[derive(Debug, Default)]
struct IRCMessage {
    tags: String,
    context: Context,
    message: String,
}
#[derive(Debug, Default)]
pub struct TwitchCallbacks {
    ping_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
    privmsg_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
    whisper_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
    custom_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
}
impl TwitchCallbacks {
    fn iter(&self) -> impl Iterator<Item = &Option<fn(&mut TwitchConnection, &IRCMessage)>> {
        vec![
            &self.ping_callback,
            &self.privmsg_callback,
            &self.whisper_callback,
            &self.custom_callback,
        ]
        .into_iter()
    }
}

struct TwitchConnection {
    stream: TcpStream,
    callbacks: Arc<Mutex<TwitchCallbacks>>,
}

impl TwitchConnection {
    fn new(server_address: String) -> Self {
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
    fn server_auth(&mut self, password: &str, username: &str) {
        self.send_message(format!("PASS oauth:{}", password).as_str());
        self.send_message(format!("NICK {}", username).as_str());
    }
    fn join_channel(&mut self, channel: &str) {
        self.send_message(format!("JOIN #{}", channel).as_str());
    }

    fn send_message(&mut self, message: &str) {
        println!("[BOT] Sending: {}", message);
        let _ = self
            .stream
            .write(format!("{}\n\r", message).as_bytes())
            .unwrap();
    }

    fn keep_alive(&mut self, interval: f32) {
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
        let message_split = message
            .splitn(3, ':')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

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
}

pub fn run(config: Config) {
    let server_address = format!("{}:{}", config.sever.address, config.sever.port);
    let mut twitch = TwitchConnection::new(server_address);
    twitch.server_auth(config.user.token.as_str(), config.user.nickname.as_str());
    for channel in config.user.channels.iter() {
        twitch.join_channel(channel.as_str());
    }
    twitch.keep_alive(60.0);

    twitch.callbacks.lock().unwrap().privmsg_callback = Some(my_privmsg_callback);
    twitch.callbacks.lock().unwrap().custom_callback = Some(my_custom_callback);
    twitch.callbacks.lock().unwrap().whisper_callback = Some(my_whisper_callback);
    twitch.callbacks.lock().unwrap().ping_callback = Some(my_ping_callback);
}

fn my_privmsg_callback(twitch: &mut TwitchConnection, payload: &IRCMessage) {
    println!("[BOT] External callback privmsg {:?}", payload)
}

fn my_custom_callback(twitch: &mut TwitchConnection, payload: &IRCMessage) {
    println!("[BOT] External callback custom {:?}", payload);
    if payload.context.command == "PRIVMSG" && payload.message.starts_with("!Ciao") {
        let msg = format!(
            "PRIVMSG {} :Ciao @{}",
            payload.context.receiver, payload.context.sender
        );
        twitch.send_message(&msg);
    }
}

fn my_whisper_callback(twitch: &mut TwitchConnection, payload: &IRCMessage) {
    println!("[BOT] External callback whisper {:?}", payload)
}

fn my_ping_callback(twitch: &mut TwitchConnection, payload: &IRCMessage) {
    println!("[BOT] External callback ping  {:#?}", payload)
}
