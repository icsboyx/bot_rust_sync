// Macro to create a TwitchConnection object
macro_rules! create_twitch_connection {
    ($stream:expr, $callbacks:expr) => {
        &mut TwitchConnection {
            stream: $stream.clone(),
            callbacks: $callbacks.clone(),
        }
    };
}

use colored::Colorize;
use std::{
    fmt::Debug,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use openssl::ssl::{SslConnector, SslMethod, SslStream};

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
// Trait to define a custom stream with Read, Write, Sync, Send, and Debug capabilities
trait CustomStreamTrait: Read + Write + Sync + Send + Debug {}

// Implement CustomStreamTrait for TcpStream
impl CustomStreamTrait for TcpStream {}

// Implement CustomStreamTrait for SslStream<TcpStream>
impl CustomStreamTrait for SslStream<TcpStream> {}

#[derive(Debug)]
// Define a struct `Streams` that wraps a boxed dynamic trait object `CustomStreamTrait`
struct Streams(Box<dyn CustomStreamTrait>);

// Struct to represent a connection to the Twitch server
#[derive(Debug, Clone)]
pub struct TwitchConnection {
    stream: Arc<Mutex<Streams>>,
    pub callbacks: Arc<Mutex<TwitchCallbacks>>,
}
impl TwitchConnection {
    pub fn new(server_address: String, tls: bool, sslverify: bool) -> Self {
        // Convert server address to string and append port
        let server_address_port = server_address.to_string();
        // Connect to the Twitch server using the provided server address and port
        let tcp_stream = std::net::TcpStream::connect(server_address_port).unwrap();

        // Check if TLS mode is enabled
        let streams = if tls {
            println!(
                "{}",
                "########################\r\n TLS Mode\r\n########################".green()
            );
            // Create an SSL connector with TLS method
            let mut ssl_connection = SslConnector::builder(SslMethod::tls()).unwrap();
            // Disable SSL verification if sslverify is false
            if !sslverify {
                ssl_connection.set_verify(openssl::ssl::SslVerifyMode::NONE);
            }
            // Build the SSL connector
            let ssl_connection = ssl_connection.build();
            // Establish an SSL connection with the Twitch server
            let ssl_stream = ssl_connection
                .connect("irc.chat.twitch.tv", tcp_stream.try_clone().unwrap())
                .unwrap();
            // Set the SSL stream to non-blocking mode
            ssl_stream.get_ref().set_nonblocking(true).unwrap();
            // Wrap the SSL stream in a Streams struct and box it
            Streams(Box::new(ssl_stream))
        } else {
            println!(
                "{}",
                "########################\r\n Clear Mode\r\n########################".red()
            );
            // Set the TCP stream to non-blocking mode
            tcp_stream.set_nonblocking(true).unwrap();
            // Wrap the TCP stream in a Streams struct and box it
            Streams(Box::new(tcp_stream))
        };

        // Clone the stream for use in the internal thread
        let stream_int = Arc::new(Mutex::new(streams));
        // Clone the stream for use as return value
        let stream_ret = stream_int.clone();

        // Create a new TwitchCallbacks object with default values
        let callbacks = Arc::new(Mutex::new(TwitchCallbacks {
            ..Default::default()
        }));
        let callbacks_int = callbacks.clone();

        thread::spawn(move || {
            let mut buffer = vec![0; 1024];
            loop {
                match stream_int.lock().unwrap().0.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        // Process each line received from the server
                        for line in String::from_utf8_lossy(&buffer[0..n - 2]).split("\r\n") {
                            println!("{} {}", "[Twitch] RAW: ".blue(), line);
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
                                    create_twitch_connection!(stream_int, callbacks_int),
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
                                            create_twitch_connection!(stream_int, callbacks_int),
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
                                            create_twitch_connection!(stream_int, callbacks_int),
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
                                            create_twitch_connection!(stream_int, callbacks_int),
                                            &message,
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        match e.kind() {
                            std::io::ErrorKind::WouldBlock => {}

                            _ => {
                                // Print any errors that occur
                                println!("{} {}", "[Twitch] Error: {}".red(), e.kind());
                                break;
                            }
                        }
                    }
                    _ => {}
                }
                thread::sleep(Duration::from_millis(5));
            }
        });
        // Return a new TwitchConnection object
        Self {
            stream: stream_ret,
            callbacks,
        }
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
        println!("{} {}", "[BOT] Sending: {}".green(), message);
        let stream_int = self.clone();
        let message_int = message.to_string();
        thread::spawn(move || {
            stream_int
                .stream
                .lock()
                .unwrap()
                .0
                .write_all(format!("{}\r\n", message_int).as_bytes())
                .unwrap()
        });
    }

    pub fn keep_alive(&mut self, interval: f32) {
        // Clone the stream and spawn a new thread to send PING messages to the server at regular intervals
        let stream = self.stream.clone();
        thread::spawn(move || loop {
            println!("{}", "[BOT] Sending: PING".green());
            stream
                .lock()
                .unwrap()
                .0
                .write_all("PING \r\n".as_bytes())
                .unwrap();
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
