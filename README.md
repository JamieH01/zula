# zula - Yet Another Shell

zula is a minimal and extendable terminal shell. It aims to streamline convenience tools for common shell interactions, via simple configuration tools and a (WIP) plugin system.
 
## Features
### Aliases
zula features recursive aliasing. Aliases can expand into other aliases, which expand into more aliases...

Commands can be escaped with `!cmd` if you wish to use a program that shares the name of an alias.
Note that aliases only apply to the first command parameter, they will not expand when used as arguments.
### Binds
zula allows you to bind any command to run when pressing `Alt + <key>`.

## Configuration
zulas main configuration file is found in `$ZULA_CONFIG/.zularc`. If this enviroment variable is not set, it defaults to `$XDG_CONFIG_HOME/zula`. It currently supports:
- `#alias` - set an alias word to be expanded into a command.
```
#alias vi nvim .
```
- `#bind` - bind a command to be triggered when pressing `Alt + <key>`.
```
#bind v vi
```

.zularc is parsed line-by-line, if a line is not a valid command it will simply be ignored.

## Todos
Here is a list of features I'm actively/plan to work on.
- config sourcing
- plugin system
- simple scripts
- applets once the plugin system is functional

## Missing
Here are some things that you may expect from a mature shell that zula is missing and may or may not be planned for the future.
- chaining commands: using operators such as `&&` and `>>` to pipe programs together
- auto suggestions
- bash-esque scripting
