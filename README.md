# zula - Yet Another Shell

zula is a minimal and extendable terminal shell. It aims to streamline convenience tools for common shell interactions, via simple configuration tools and a (WIP) plugin system.
 
## Configuration
zulas main configuration file is found in `$XDG_CONFIG_HOME/zula/.zularc`. It currently supports:
- `#alias` - set an alias word to be expanded into a command. 
`#alias vi nvim .`
- `#bind` - bind a command to be triggered when pressing `Alt + <key>`.
`#bind v vi`
.zularc is parsed line-by-line, if a line is not a valid command it will simply be ignored.

## Todos
Here is a list of features I'm actively/plan to work on.
- alias bypass
- dedicated `zula` command for health and info
- plugin system
- simple scripts
- applets once the plugin system is functional

## Missing
Here are some things that you may expect from a mature shell that zula is missing and may or may not be planned for the future.
- chaining commands: using operators such as `&&` and `>>` to pipe programs together
- auto suggestions
- bash-esque scripting
