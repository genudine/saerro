# ESS Demux

This service guarantees one thing to you; it will have a websocket connected with ESS events.

The specific flow is as follows:

1. If https://push.nanite-systems.net/ is up, the client websocket is wired to that.
2. Else, connect to https://push.planetside2.com/ based on `?environment={}`, and the client websocket is wired to either 1 or 3 of those.

   - If environment = `all`, it will connect 3 times to `pc`, `ps4us`, and `ps4eu`.
   - Else, connect to specified environment.
   - Also, try reconnecting to the main socket every minute.

3. If that fails, the client websocket will never respond.

## Why would you want this?

NSS helps be resilient to ESS failures, but NSS isn't failure-proof itself. This acts as a proxy that'll gracefully select one source or another.

### Alternatives

If you can accept the loss of PS4 data, you may use nginx or HAProxy to achieve the same effect...

[**nginx example.conf**](./docs/alternatives/ess.nginx.conf)

The above may not work entirely correctly... ymmv.

Saerro **does** want PS4 data, so we use the ess-demux service.

## How to use this

The service runs on port 8007 by default, you can change it to whatever via `PORT`, if you're using this as a bare service. You may also change the `DEFAULT_SERVICE_ID` from `s:example`; allowing you to omit this from the URL.

`docker run -d -p 8007:8007 ghcr.io/genudine/saerro/ess-demux:latest`

Connect to `ws://localhost:8007/streaming?environment=all&service-id=s:example`

Send subscriptions like any other ESS-compatible websocket.

Upon connection, you can expect an event like this:

```json
{
  "connected": true,
  "service": "ess-demux",
  "type": "essDemuxConnectionStateChanged",
  "upstream": "nss" // or "ess"
}
```
