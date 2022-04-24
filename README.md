# rusty-heos

## Build

Using https://github.com/nix-community/naersk

## Idea

The Heos API is a bit strange. Some informations can only be retrieved via the "REST"-Api and some onl;y via the event stream.

```
   |device | <----> | api | <---- command
            
   
```
