# rusty-heos

## Build

Using https://github.com/nix-community/naersk

The Heos API is a bit strange. Some informations can only be retrieved via the "REST"-Api and some only via the event stream.

- The driver loads all players and groups
- it loads the players and groups details
- it registeres for change events
- http://samherbert.net/svg-loaders/
