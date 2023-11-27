// Macro to create a TwitchConnection object
macro_rules! create_twitch_connection {
    ($stream:expr, $callbacks:expr) => {
        &mut TwitchConnection {
            stream: $stream.try_clone().unwrap(),
            callbacks: $callbacks.clone(),
        }
    };
}

use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

// Struct to represent the context of a message
#[derive(Debug, Default)]
pub struct Context {
    pub sender: String,
    pub command: String,
    pub receiver: String,
}

// Struct to represent a message from the server
#[derive(Debug, Default)]
pub struct IRCMessage {
    pub tags: String,
    pub context: Context,
    pub message: String,
}

// Struct to represent the callbacks that will be triggered when certain types of messages are received
#[derive(Debug, Default)]
pub struct TwitchCallbacks {
    pub ping_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
    pub privmsg_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
    pub whisper_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
    pub custom_callback: Option<fn(&mut TwitchConnection, &IRCMessage)>,
}

// Enum to represent the different capabilities that can be requested from the Twitch server
pub enum TwitchCapabilities {
    Tags,
    Commands,
    Membership,
}

// Implementation of TwitchCapabilities enum
impl TwitchCapabilities {
    fn request(&self) -> String {
        match self {
            TwitchCapabilities::Tags => "CAP REQ :twitch.tv/tags".to_string(),
            TwitchCapabilities::Commands => "CAP REQ :twitch.tv/commands".to_string(),
            TwitchCapabilities::Membership => "CAP REQ :twitch.tv/membership".to_string(),
        }
    }
}

// Struct to represent a connection to the Twitch server
pub struct TwitchConnection {
    stream: TcpStream,
    pub callbacks: Arc<Mutex<TwitchCallbacks>>,
}
impl TwitchConnection {
    pub fn new(server_address: String) -> Self {
        // Convert server address to string and append port
        let server_address_port = server_address.to_string();
        // Establish a TCP connection to the server
        let stream = std::net::TcpStream::connect(server_address_port).unwrap();
        // Clone the stream for use in the callback thread
        let stream_int = stream.try_clone().unwrap();
        // Create a new TwitchCallbacks object with default values
        let callbacks = Arc::new(Mutex::new(TwitchCallbacks {
            ..Default::default()
        }));

        // Clone the stream and callbacks for use in the callback thread
        let stream_int_callback = stream_int.try_clone().unwrap();
        let callbacks_int = callbacks.clone();

        // Spawn a new thread to handle incoming messages from the server
        thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stream_int);
            let mut buffer = vec![0; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        // Process each line received from the server
                        for line in String::from_utf8_lossy(&buffer[0..n - 2]).split("\r\n") {
                            println!("[Twitch] RAW: {}", line);
                            // Parse the line into an IRCMessage
                            let message = Self::parse_twitch_message(line);

                            // If a custom callback is set, call it
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
                            // Handle different types of messages
                            match message.context.command.as_str() {
                                "PRIVMSG" => {
                                    // If a PRIVMSG callback is set, call it
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
                                    // If a PING callback is set, call it
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
                                    // If a WHISPER callback is set, call it
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
                        // Print any errors that occur
                        println!("[Twitch] Error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        // Return a new TwitchConnection object
        Self { stream, callbacks }
    }
    pub fn server_auth(&mut self, password: &str, username: &str) {
        // Send the password and username to the server to authenticate
        self.send_message(format!("PASS oauth:{}", password).as_str());
        self.send_message(format!("NICK {}", username).as_str());
    }

    pub fn join_channel(&mut self, channel: &str) {
        // Send a JOIN message to the server to join a channel
        self.send_message(format!("JOIN #{}", channel).as_str());
    }

    pub fn send_message(&mut self, message: &str) {
        // Print the message to the console and send it to the server
        println!("[BOT] Sending: {}", message);
        let _ = self
            .stream
            .write(format!("{}\n\r", message).as_bytes())
            .unwrap();
    }

    pub fn keep_alive(&mut self, interval: f32) {
        // Clone the stream and spawn a new thread to send PING messages to the server at regular intervals
        let mut stream = self.stream.try_clone().unwrap();
        thread::spawn(move || loop {
            println!("[BOT] Sending: PING");
            stream.write_all("PING \r\n".as_bytes()).unwrap();
            thread::sleep(Duration::from_secs(interval as u64));
        });
    }

    fn parse_twitch_message(message: &str) -> IRCMessage {
        // Initialize a new IRCMessage with default values
        let mut msg = IRCMessage {
            ..Default::default()
        };

        // Split the message into parts by ':', and collect the parts into a vector
        let mut message_split = message
            .splitn(3, ':')
            .map(|s| s.to_owned())
            .collect::<Vec<String>>();

        // If the message has less than 3 parts and does not start with ':', set the command and sender fields of the context
        if message_split.len() < 3 && !message.starts_with(':') {
            msg.context.command = message_split.get(0).unwrap_or(&"".to_string()).to_owned();
            msg.context.sender = message_split.get(1).unwrap_or(&"".to_string()).to_owned();
            msg.context.receiver = "*".to_string();
            return msg;
        }

        // If the message has less than 3 parts and starts with ':', add an empty string to the end of the vector
        if message_split.len() < 3 && message.starts_with(':') {
            message_split.push("".to_string());
        }

        // Set the tags field of the message
        msg.tags = message_split.get(0).unwrap_or(&"".to_string()).to_owned();

        // Set the context field of the message
        msg.context = message_split
            .get(1) // Use the second element or None if no elements
            .map(|s| {
                let mut context = Context {
                    ..Default::default()
                };
                // Split the second part of the message by ' '
                let mut split = s.split(' ');
                // Set the sender, command, and receiver fields of the context
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

        // Set the message field of the message
        msg.message = message_split.get(2).unwrap_or(&"".to_string()).to_owned();

        msg
    }

    pub fn request_capabilities(&mut self, capabilities: Vec<TwitchCapabilities>) {
        // Send a request to the server for each capability in the list
        for capability in capabilities.iter() {
            self.send_message(capability.request().as_str());
        }
    }
}
