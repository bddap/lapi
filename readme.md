# Experimental api for easy lightning payments.

# todo

- [x] Create lighting node trait
- [x] Define or import lightning invoice type
- [x] Mock lighning node impl
- [x] Impl LightningNode with lnd as backend
- [ ] Expose rest api
- [ ] Run generic tests against each pair in cartesian_product({FakeDb, RealDb}, {FakeNode, RealNode})
- [ ] Verify lightning payment_hash works as an invoice uuid. If not, we may need to invoice description field instead.
- [ ] Impl Db with some sort of persistent storage backend
- [ ] Implement tests generic over LightningNode and Db traits.

# Build setup 🤮

The lnd rpc lib we use requires GOPATH to be set and
$GOPATH/src/github.com/grpc-ecosystem/grpc-gateway to exist at compile time.
It also required `protoc` to be installed and on-path. `brew install prrotobuf`.
