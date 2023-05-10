# WSH

A tool to search the web with bang command similar to DuckDuckGo.

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
url = "http://www.duden.de/suchen/dudenonline/{{s}}"
```

The prefix is the character that precedes every command.

If you update the configuration file, you can reload it by sending a GET request to `http://wsh.example.com:$WEB_COMMAND_PORT/r/`.

### The service

The tool is a single executable. It is configured via environment variables.

```bash
WEBCOMMAND_PORT=8012
WEBCOMMAND_HOST_MODE=true
# If the host mode variable is true, the service will look 
# for the configuration file in the path specified by the following variable.
# Otherwise it will try to fetch the configuration file from the url specified by the following variable.
WEBCOMMAND_CONFIG=./sites.toml
```

#### The host mode

To speed up the service it is recommended to run it as a daemon on your local machine (this is the default mode).  
In this case, the service will try to fetch the configuration file from a remote location, so that you can have a single configuration file for all your machines.  
If you have a remote machine you can run the service on it and set the host mode variable. In this case it will use a local configuration file and expose it via `http://wsh.example.com:$WEB_COMMAND_PORT/u/`.