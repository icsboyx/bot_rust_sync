mod config;
use config::Config;
mod twitch;
use twitch::{IRCMessage, TwitchCapabilities, TwitchConnection};

fn main() {
    let config = config::load_config();

    let server_address = format!("{}:{}", config.sever.address, config.sever.port);
    let mut twitch = TwitchConnection::new(server_address);
    twitch.server_auth(config.user.token.as_str(), config.user.nickname.as_str());

    for channel in config.user.channels.iter() {
        twitch.join_channel(channel.as_str());
    }

    twitch.request_capabilities(vec![
        TwitchCapabilities::Tags,
        TwitchCapabilities::Commands,
        TwitchCapabilities::Membership,
    ]);

    twitch.keep_alive(60.0);

    twitch.callbacks.lock().unwrap().privmsg_callback = Some(my_privmsg_callback);
    twitch.callbacks.lock().unwrap().custom_callback = Some(my_custom_callback);
    twitch.callbacks.lock().unwrap().whisper_callback = Some(my_whisper_callback);
    twitch.callbacks.lock().unwrap().ping_callback = Some(my_ping_callback);

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
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
