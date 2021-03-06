# Experimental api for easy lightning payments.

This project is parked, waiting for Rust futures and async/await syntax to become stable.
Without async/await, this code is complex and difficult to reason about.

# todo

- [x] Create lighting node trait
- [x] Define or import lightning invoice type
- [x] Mock lighning node impl
- [x] Impl LightningNode with lnd as backend
- [x] Expose rest api
- [x] Run generic tests against each pair in cartesian_product({FakeDb, RealDb}, {FakeNode, RealNode})
- [x] Verify lightning payment_hash works as an invoice uuid. If not, we may need to invoice description field instead.
- [ ] Impl Db with some sort of persistent storage backend
- [x] Implement tests generic over LightningNode and Db traits.
- [x] Allow api client to provide hash preimage when generating an invoice. 
      Went a different route, preimage is randomly generated and sent to the client on invoice request.
- [x] endpoints::Api should contain a structured error logger. When a server error is
      encountered, http 500 should be returned and the error logged to logger.
- [ ] Consider tracking multiple payments to a single invoice
- [ ] Consider using Server Sent Events instead of websockets.
      https://docs.rs/warp/0.1.14/warp/filters/sse/struct.Sse.html
	  https://en.wikipedia.org/wiki/Server-sent_events
	  https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events
- [ ] Automatically test api documentation

Dropped tasks:

- [ ] Rename Master Middle Lesser to somthing more consistent like Greater Middle Lesser or 
      Greater Nominal Lesser. Use "auth\_{greater,middle,lesser}" in api arguments for better
	  readability.

# Build setup 🤮

The lnd rpc lib we use requires GOPATH to be set and
$GOPATH/src/github.com/grpc-ecosystem/grpc-gateway to exist at compile time.
It also required `protoc` to be installed and on-path. `brew install protobuf`.
