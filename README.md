# Saerro Listening Post

PlanetSide 2 live population API. This API is free and open for anyone to use.

The one and only goal of this app is to provide a current "point-in-time" population status for PlanetSide 2, per world, per faction, (and later, per continent.) Historical info is _not_ a goal.

https://saerro.harasse.rs

## API Reference

- [`/`](https://saerro.harasse.rs) - Shows a help/usage message.
- [`/w/{worldID}`](https://saerro.harasse.rs/w/17) - Shows populations for one world, example:

```json
{
  "worldID": 17,
  "total": 1000,
  "factions": {
    "tr": 334,
    "nc": 333,
    "vs": 333
  }
}
```

- [`/m/?id={worldID1},{worldID2}...`](https://saerro.harasse.rs/m/?id=1,17) - Shows populations for all listed worlds, example:

```json
{
  "worlds": [
    {
      "worldID": 17,
      "total": 1000,
      "factions": {
        "tr": 334,
        "nc": 333,
        "vs": 333
      }
    },
    {
      "worldID": 19,
      "total": 1000,
      "factions": {
        "tr": 334,
        "nc": 333,
        "vs": 333
      }
    }
  ]
}
```

## Architecture

- Websocket processors
  - One pair per PC, PS4US, PS4EU
  - Each pair connects to either Census Websocket or NS Websocket, depending on availability.
