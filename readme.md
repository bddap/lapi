# Experimental api for easy lightning payments.

# todo

- [x] Create lighting node trait
- [x] Define or import lightning invoice type
- [x] Mock lighning node impl
- [ ] Impl LightningNode with lnd as backend
- [ ] Impl Db with some sort of persistent storage backend
- [ ] Run generic tests against each pair in cartesian_product({FakeDb, RealDb}, {FakeNode, RealNode})
- [ ] Verify lightning payment_hash works as an invoice uuid. If not, we may need to invoice description field instead.
- [ ] Implement tests generic over LightningNode and Db traits.
