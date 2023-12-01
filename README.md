# Twitch Bot

This is a Twitch bot built in Rust. It connects to Twitch channels and handles different types of messages.

## Features

- **Multiple Channel Support**: The bot can connect to multiple Twitch channels. The channels are specified in the configuration.

- **Capability Requests**: The bot requests certain capabilities from the Twitch server, specifically tags, commands, and membership. This allows the bot to receive certain types of messages.

- **Keep Alive**: The bot sends a PING message to the server every 60 seconds to keep the connection open.

- **Message Handling**: The bot handles different types of messages, including private messages, custom messages, whispers, and pings. It uses callbacks to handle these messages, which means you can customize the bot's behavior by modifying these callbacks.

## Usage

1. **Clone the repository**: Use `git clone` to clone the repository to your local machine.

2. **Update the Configuration**: Update the `config` with your Twitch server address, port, token, nickname, and channels.

3. **Run the Bot**: Run the bot with `cargo run`. The bot will connect to the specified channels and start listening for messages.

## Configuration

Before running the bot, you need to set up the configuration. A template configuration file is provided in `config.json.template`.

1. **Copy the Template**: Copy `config.json.template` to a new file named `config.json`.

2. **Update the Configuration**: Open `config.json` in a text editor. You'll see something like this:

    ```json
        {
            "application": {
                "log_level": "Trace"
            },
            "sever": {
                "address": "irc.chat.twitch.tv",
                "port": 6697,
                "ssl_tls": true,
                "ssl_verify_mode": true
            },
            "user": {
                "token": "your-token-here",
                "nickname": "your-nickname-here",
                "main_channel": "channel1",
                "channels": [
                    "channel1",
                    "channel2",
                    "channel3"
                ]
            }
        }
    ```

    Replace `"your-token-here"` with your Twitch token, `"your-nickname-here"` with your Twitch nickname, and `["channel1", "channel2"]` with a list of channels you want the bot to join.

3. **Save the Configuration**: Save `config.json`. The bot will read this file when it starts.

Please make sure not to commit `config.json` to version control, as it contains sensitive information. It's already included in the `.gitignore` file to prevent this.

This code is responsible for setting up the Twitch bot and connecting it to the Twitch server.

1. **Load Configuration**: The `config::load_config()` function is called to load the bot's configuration from a file.

2. **Create Twitch Connection**: A `TwitchConnection` object is created using the server address and port from the configuration. The `server_auth` method is then called to authenticate the bot with the Twitch server using the token and nickname from the configuration.

3. **Join Channels**: The bot iterates over the list of channels from the configuration and joins each one using the `join_channel` method.

4. **Request Capabilities**: The bot requests certain capabilities from the Twitch server, specifically tags, commands, and membership. This allows the bot to receive certain types of messages.

5. **Keep Alive**: The `keep_alive` method is called to send a PING message to the server every 60 seconds. This keeps the connection to the server open.

6. **Set Callbacks**: The bot sets up callbacks for different types of messages. These are functions that will be called when a certain type of message is received. The `privmsg_callback` is set to `my_privmsg_callback`, the `custom_callback` is set to `my_custom_callback`, the `whisper_callback` is set to `my_whisper_callback`, and the `ping_callback` is set to `my_ping_callback`. These functions are defined elsewhere in the code.

## Callbacks

```rust
twitch.callbacks.lock().unwrap().privmsg_callback = Some(my_privmsg_callback);
twitch.callbacks.lock().unwrap().custom_callback = Some(my_custom_callback);
twitch.callbacks.lock().unwrap().whisper_callback = Some(my_whisper_callback);
twitch.callbacks.lock().unwrap().ping_callback = Some(my_ping_callback);
```

The bot uses callbacks to handle different types of messages. These are functions that are called when a certain type of message is received. You can modify these functions in `main.rs` to customize the bot's behavior.

- `my_privmsg_callback`: This function is called when a private message is received. It takes the Twitch connection and the message as arguments.

- `my_custom_callback`: This function is called when a custom message is received. It takes the Twitch connection and the message as arguments.

- `my_whisper_callback`: This function is called when a whisper is received. It takes the Twitch connection and the message as arguments.

- `my_ping_callback`: This function is called when a PING message is received. It takes the Twitch connection and the message as arguments.


The functions `my_privmsg_callback`, `my_custom_callback`, `my_whisper_callback`, and `my_ping_callback` are the functions that will be called when these events occur. You can replace these with your own functions if you prefer. Just make sure that your functions have the same signature as the ones provided.

### Contribution
Contributions are welcome! If you find a bug or want to add new features, feel free to create a pull request.

## Special Thanks

This project was inspired by the Twitch channel of:\
![Prof. Andrea Pollini](https://static-cdn.jtvnw.net/jtv_user_pictures/b4199595-d595-4788-9f04-f4aa370e902a-profile_image-70x70.png)[Prof. Andrea Pollini](https://www.twitch.tv/profandreapollini),\
![Memmo_Twich](https://static-cdn.jtvnw.net/jtv_user_pictures/93321124-9685-4bf5-9abd-85967497553f-profile_image-70x70.png)
[Memmo_Twich](https://www.twitch.tv/memmo_twitch)\
and  the supportive Twitch community. Thanks to their encouragement and feedback!



## License

This project is licensed under the MIT License - see the [LICENSE](https://www.mit.edu/~amini/LICENSE.md) for details.