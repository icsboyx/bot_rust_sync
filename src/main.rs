mod config;
mod twitch;

fn main() {
    let config = config::load_config();

    twitch::run(config);
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
