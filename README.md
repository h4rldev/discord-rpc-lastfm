# Discord RPC Last.fm

Discord Rich Presence client for Last.fm, made in Rust.

## Features

- Fetches the current track from Last.fm and displays it as a Discord Rich Presence.
- Handles errors gracefully and retries connections.
- Verifies image URLs before using them.
- Supports environment variables, a config aswell as .env
- _should_ support Linux and Windows.

## Installation

> Get it from [releases](https://github.com/h4rldev/discord-rpc-lastfm/releases)

or

> Build it yourself!
1. Clone the repository: `git clone https://github.com/h4rldev/discord-rpc-lastfm`
2. Navigate to the project directory: `cd discord-rpc-lastfm`
3. Build the project: `cargo build --release`

## Usage

1. Run the executable
2. Follow the prompts to enter your Last.fm username and API key, and possibly client_id (if you want a different title).

## Todo

- [ ] Log to file instead of in console
- [ ] Toggle console
- [ ] Run in background

## Dependencies

- `discord-rich-presence`
- `reqwest`
- `tokio`
- `toml`
- `serde`
- `tracing`
- `dotenv`
- `url`
- `tracing-subscriber`
- `futures`
- `inquire`

## Contributing

Pull requests are welcome.

## License

[BSD 3-Clause](https://choosealicense.com/licenses/bsd-3-clause/)
