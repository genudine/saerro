# Saerro Listening Post

PlanetSide 2 live population API. This API is free and open for anyone to use.

https://saerro.harasse.rs

Our methodology is to add any player ID seen on the Census websockets to a time-sorted set, and returning the number of player IDs seen within 15 minutes.

---

The one and only goal of this app is to provide a current "point-in-time" population status for PlanetSide 2, per world, per faction, (and later, per continent.) Historical info is _not_ a goal; you may implement this on your end.

Please open an issue here or get in touch with Pomf (okano#0001) on the PS2 Discord if you have complex use cases for this data; it may be trivial/easy to implement APIs tailored to your needs.

The main use case is for [Medkit](https://github.com/kayteh/medkit2) bot to have an in-house source of population data, without relying too heavily on any third-party stats service, like Fisu, Honu, or Voidwell; which all have different population tracking needs and goals (and thus, different data.)

## Architecture

- Websocket processors
  - A pair per PC, PS4US, PS4EU
  - Connects to [wss://push.nanite-systems.net](https://nanite-systems.net) and Census Websocket
  - Primary will connect to NS.
  - Backup will connect to Census. It will wait for 60 seconds before deciding the primary is dead, and then start processing events.
- API
  - Serves https://saerro.harasse.rs
  - Built on axum and async-graphql
- Redis
  - Using ZADD with score as timestamp, ZCOUNTBYSCORE by timestamp in 15 minute windows, and cleaned up with SCAN+ZREMBYSCORE, population data is tracked.
  - There is deliberately no persistence.
- Redis "Tender"
  - Cleans up Redis every 5 mins.

# Developing

This app is built with Rust. You can set up a build environment via https://rustup.rs/

To run,

```sh
# Start Redis/backing services
docker compose up -d

# Start Websocket for PC
env \
  WS_ADDR="wss://push.planetside2.com/streaming?environment=ps2&service-id=s:$SERVICE_ID" \
  PAIR=pc \
  ROLE=primary \
  WORLDS=1,10,13,17,19,40 \
  cargo run --bin websocket

# (Optional:) Start redundant websocket for PC
env \
  WS_ADDR="wss://push.planetside2.com/streaming?environment=ps2&service-id=s:$SERVICE_ID" \
  PAIR=pc \
  ROLE=backup \
  WORLDS=1,10,13,17,19,40 \
  cargo run --bin websocket

# (Optional:) Start PS4US websocket
env \
  WS_ADDR="wss://push.planetside2.com/streaming?environment=ps2ps4us&service-id=s:$SERVICE_ID" \
  PAIR=ps4us \
  WORLDS=1000 \
  cargo run --bin websocket

# (Optional:) Start PS4EU websocket
env \
  WS_ADDR="wss://push.planetside2.com/streaming?environment=ps2ps4eu&service-id=s:$SERVICE_ID" \
  PAIR=ps4eu \
  WORLDS=2000 \
  cargo run --bin websocket

# Start API
cargo run --bin api

# Run prune tool
cargo run --bin tools prune

# Build containers
docker build . --build-arg SERVICE=api -t saerro:api
docker build . --build-arg SERVICE=tools -t saerro:tools
docker build . --build-arg SERVICE=websocket -t saerro:websocket
```

## Code Generation

Some aspects of this code are based on "moving parts" within PlanetSide 2. If these change, you can run `cargo run --bin codegen` to regenerate these from API. PRs are accepted for this :)

# Deploying

Currently, the entire stack runs on Docker. You may deploy it to any server via:

```sh
docker compose up -d -f docker-compose.live.yaml
```

It listens on port 80, it's up to you from here.
