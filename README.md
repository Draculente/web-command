# WSH

A tool to search the web with bang commands similar to DuckDuckGo. For example: `.d hello` will search for "hello" on DuckDuckGo.

## Configuration

### The Commands

The commands are defined in a TOML file:

```toml
prefix = "."

[[sites]]
name = "duckduckgo"
key = "d"
url = "https://duckduckgo.com/?q={{s}}"
[[sites]]
name = "duden"
key = "du"
alias = ["duden"]
url = "http://www.duden.de/suchen/dudenonline/{{s}}"
```

The prefix is the character that precedes every command.

The first entry is the default. If no command is specified, the default command will be used.

Optionally a command can have an array of aliases. On load the aliases will be expanded to their own key. This is useful if you want to have multiple keys for the same site.

A full example can be found [here](...).

If you update the configuration file, you can reload it by sending a GET request to `http://wsh.example.com:$WEBCOMMAND_PORT/r/`.

### The service

The tool is a single executable. It is configured via environment variables.

| Variable | Default | Description |
|---|---|---|
| `WEBCOMMAND_PORT` | `8012` | The port the application is listening on. |
| `WEBCOMMAND_HOST_MODE` | `false` | If the application runs in host mode or mirror mode. |
| `WEBCOMMAND_CONFIG`  | `./sites.toml` | If the application runs in mirror mode (the default), this variable needs to contain an url, that points to a wsh instance running in host mode. If the application runs in host mode, this variable is interpreted as a path to a `toml` configuration file. If you don't want to maintain your own configuration file, you can use `https://wsh.draculente.me/` as the url. This is the configuration file of my instance. If you want to add a site, you can create a pull request here. |

#### The host mode

To speed up the service it is recommended to run it as a daemon in mirror-mode on your local machine (this is the default mode).  
In this case, the service will try to fetch the configuration file from a remote location, so that you can have a single configuration file for all your machines.  

If you have a remote machine you can run the service on it and set the host mode variable. In this case it will use a local configuration file and expose it via `http://wsh.example.com:$WEBCOMMAND_PORT/u/`.

## Installation

### Docker

The easiest way to run the service is to use docker-compose. You can find an example docker-compose file [here](...).  
We set the restart policy to `unless-stopped` so that the service will be restarted on boot.  

1. Create a new directory.
1. Download the docker-compose file into that directory.
1. Create a `.env` file in that directory and set the environment variables as described in `configuration`.
1. IF YOU RUN IT AS HOST: Create a `sites.toml` file in that directory and set the sites as described in `configuration`.
1. Run `docker compose up -d`.

Of course you can also run the service directly with docker (`docker run ...`).

After some benchmarking I found that running the service via docker-compose increases latency by about 68% compared to directly running it on the host (from 789µs to 1.2ms on my machine).

### Systemd

The recommended way to run the service is to use systemd. You can find an example systemd service file [here](...).

#### Running directly on the host

1. Create a new directory.
1. Download the appropriate binary from the [releases](...) page into that directory.
1. Create a `.env` file in that directory and set the environment variables as described in `configuration`.
1. IF YOU RUN IT AS HOST: Create a `sites.toml` file in that directory and set the sites as described in `configuration`.
1. Copy the systemd service file into `/etc/systemd/user/wsh.service`.
1. Adjust the service file to your needs.
1. Run `systemctl --user daemon-reload`.
1. Run `systemctl --user enable wsh.service`.
1. Run `systemctl --user start wsh.service`. 


#### Running in a docker container

If you want to, you could also run the service in a docker container put use systemd to manage it (even though I don't see any reason to do so). For further information see [here](https://www.jetbrains.com/help/youtrack/server/run-docker-container-as-service.html).

## Build

### Rust

1. `git clone https://github.com/Draculente/web-command.git`
2. `cd web-command`
3. `cargo build --release`