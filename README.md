# Saerro Listening Post

PlanetSide 2 live population API. This API is free and open for anyone to use.

https://saerro.ps2.live

tl;dr: Watch for specific events, transform and add them to a timeseries set, and query that set for the last 15 minutes.

We're built on 3 core types, `players`, `classes`, and `vehicles`. Each can be filtered by Continent/Zone, Faction, and World.

---

Please open an issue here or get in touch with Pomf (okano#0001) on the PS2 Discord if you have complex use cases for this data; it may be trivial/easy to implement APIs tailored to your needs.

The main use case is for [Medkit](https://github.com/kayteh/medkit2) bot to have an in-house source of population data, without relying too heavily on any third-party stats service, like Fisu, Honu, or Voidwell; which all have different population tracking needs and goals (and thus, different data.)

An example of how it can be used on [pstop](https://pstop.harasse.rs) ([GitHub](https://github.com/genudine/pstop)).

## Architecture

- GraphQL API
  - Serves https://saerro.ps2.live
  - Built on a "stacking filter" graph model, where each dimension adds a filter to lower dimensions.
- Event Streaming Service (ESS) Ingest
  - WebSocket listening to https://push.nanite-systems.net (which is a resilient mirror to https://push.planetside2.com)
  - Listens for `Death`, `VehicleDestroy`, and a number of `GainExperience` events.
- Postgres with TimescaleDB
  - Holds `players` and `analytics` tables as hypertables.
  - Timescale makes this way too fast, mind-blowing :)
- Tasks
  - Occasional jobs that prune the database past what we actually want to retain,
    - Core data tables are kept to about 20 mins max of data, analytics to 1 week
  - Can do database resets/migrations.

# Developing

This app is built with Rust. You can set up a build environment via https://rustup.rs/

To run,

```sh
# Start backing services
docker compose up -d

# Run database migrations (required first step on a freshly up'd database)
cargo run --bin tasks migrate

# Start NSS ingest. Use push.planetside2.com if NSS isn't quite working...
env \
  WS_ADDR="wss://push.nanite-systems.net/streaming?environment=all&service-id=s:$SERVICE_ID" \
  WORLDS=all \
  cargo run --bin websocket

# Start API
cargo run --bin api

# Run prune tool
cargo run --bin tasks prune

# Build containers
docker build . --build-arg SERVICE=api -t saerro:api
docker build . --build-arg SERVICE=tasks -t saerro:tasks
docker build . --build-arg SERVICE=websocket -t saerro:websocket
```

## Code Generation

Some aspects of this code are based on "moving parts" within PlanetSide 2. If these change, you can run `cargo run --bin codegen` to regenerate these from API. PRs are accepted for this :)

# Deploying

Currently, the entire stack runs on Docker. You may deploy it to any server via:

```sh
docker compose up -d -f docker-compose.live.yaml
```

It listens on port 80, it's up to you from here. Make sure to change passwords present in the file. It's not _that secret_ of data, but why risk it?
