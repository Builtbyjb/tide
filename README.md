# Tide

## Documentation
** Warning: do not work on windows machines **
Support for windows machines would come at a later date.

Run *tide init* to create a tide.toml file.

*tide run [command]* to run a list of commands assigned to variable ("dev", "prod", "test") in 
command table.

*tide run [command] --watch* to run the commands in watch mode. *tide* monitors the project for file
changes and re runs the commands when the file changes.

### Tide Config
You can configure how tide works by editing the tide.toml configuration file.

The file contains a variable and two tables.

The variable **root_dir** sets the starting point of the directories *tide* will watch.

The table **[command]** contains three variables:
**dev** -> a list of all the commands to run in a development environment.
**prod** -> a list of all the commands to run in a production environment.
**test** -> a list of all the commands to run tests.

The table **[exclude]** contains three variables:
**dir** -> a list of directories *tide* should not watch.
**file** -> a list of files *tide* should not watch.
**ext** -> a list of file extensions *tide* should not watch.