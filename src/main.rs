mod config;
mod twitch;
use colored::Colorize;
use twitch::{IRCMessage, TwitchCapabilities, TwitchConnection};

fn main() {
    // Load the configuration
    let config = config::load_config();

    // Create a new TwitchConnection
    let server_address = format!("{}:{}", config.sever.address, config.sever.port);
    let mut twitch = TwitchConnection::new(
        server_address,
        config.sever.ssl_tls,
        config.sever.ssl_verify_mode,
    );

    // Authenticate with the server
    twitch.server_auth(config.user.token.as_str(), config.user.nickname.as_str());

    // Join each channel specified in the config
    for channel in config.user.channels.iter() {
        twitch.join_channel(channel.as_str());
    }

    // Request certain capabilities from the server
    twitch.request_capabilities(vec![
        TwitchCapabilities::Tags,
        TwitchCapabilities::Commands,
        TwitchCapabilities::Membership,
    ]);

    // Start the keep-alive thread
    twitch.keep_alive(60.0);

    // Set the callbacks
    twitch.callbacks.lock().unwrap().privmsg_callback = Some(my_privmsg_callback);
    twitch.callbacks.lock().unwrap().custom_callback = Some(my_custom_callback);
    twitch.callbacks.lock().unwrap().whisper_callback = Some(my_whisper_callback);
    twitch.callbacks.lock().unwrap().ping_callback = Some(my_ping_callback);

    // Main loop
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// Callback for PRIVMSG messages
fn my_privmsg_callback(twitch: &mut TwitchConnection, payload: &IRCMessage) {
    println!("[BOT] External callback privmsg {:?}{:?}", twitch, payload)
}

// Callback for custom messages
fn my_custom_callback(twitch: &mut TwitchConnection, payload: &IRCMessage) {
    println!(
        "{} {:?}",
        "[BOT] External callback custom ".yellow(),
        payload
    );
    // If the message is a PRIVMSG that starts with "!Ciao", send a response
    if payload.context.command == "PRIVMSG" && payload.message.starts_with("!Ciao") {
        let msg = format!(
            "PRIVMSG {} :Ciao @{}",
            payload.context.receiver, payload.context.sender
        );
        twitch.send_message(&msg);
    }
}

// Callback for WHISPER messages
fn my_whisper_callback(twitch: &mut TwitchConnection, payload: &IRCMessage) {
    println!("[BOT] External callback privmsg {:?}{:?}", twitch, payload)
}

// Callback for PING messages
fn my_ping_callback(twitch: &mut TwitchConnection, payload: &IRCMessage) {
    println!("[BOT] External callback privmsg {:?}{:?}", twitch, payload)
}
