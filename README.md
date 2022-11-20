# Saerro Listening Post

PlanetSide 2 live population API. This API is free and open for anyone to use.

https://saerro.harasse.rs

Our methodology is to add any player ID seen on the Census websockets to a time-sorted set, and returning the number of player IDs seen within 15 minutes.

---

The one and only goal of this app is to provide a current "point-in-time" population status for PlanetSide 2, per world, per faction, (and later, per continent.) Historical info is _not_ a goal; you may implement this on your end.

Please open an issue here or get in touch with Pomf (okano#0001) on the PS2 Discord if you have complex use cases for this data; it may be trivial/easy to implement APIs tailored to your needs.

The main use case is for [Medkit](https://github.com/kayteh/medkit2) bot to have an in-house source of population data, without relying too heavily on any third-party stats service, like Fisu, Honu, or Voidwell; which all have different population tracking needs and goals (and thus, different data.)

## API Reference

This API will never return a failure unless the app itself is failing. If you request a world ID that doesn't exist, it only sees an empty set, and will not 404. Since 0 can both mean a server is down and it doesn't exist, be mindful of the data. It could even mean the Websockets have failed.

This API only supports GET, and supports CORS.

- [`/`](https://saerro.harasse.rs) - Shows a help/usage message.

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
- API
  - Serves https://saerro.harasse.rs
- Redis
  - Using ZADD with score as timestamp, ZCOUNTBYSCORE by timestamp in 15 minute windows, and cleaned up with SCAN+ZREMBYSCORE, population data is tracked.
- Redis "Tender"
  - Cleans up Redis every 5 mins.
