//! Exposes helpers for use in testing.
//! Tests use real satoshis.

use crate::common::*;
use grpc::ClientStub;
use lnd_rust::{
    macaroon_data::MacaroonData,
    rpc::{
        AddInvoiceResponse, FeeLimit, FeeLimit_oneof_limit, Invoice_InvoiceState, SendRequest,
        SendResponse,
    },
    rpc_grpc::{Lightning, LightningClient},
    tls_certificate::TLSCertificate,
};
use std::{
    io,
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    sync::Arc,
};

pub const ACCOUNT_A: Master = Master(U256([
    0xda, 0xbd, 0xf8, 0xc5, 0x74, 0xfb, 0x9a, 0x9e, 0x27, 0x72, 0x05, 0xe2, 0xda, 0x3d, 0x38, 0xf1,
    0x49, 0x60, 0x8e, 0x34, 0x96, 0x8c, 0x1f, 0xf1, 0x5f, 0xb9, 0xf1, 0x83, 0xde, 0x5c, 0x40, 0x00,
]));
pub const DEFAULT_FEE: Fee<Satoshis> = Fee(Satoshis(10));

pub fn init_lightning_client(
    tls_cert: &Path,
    macaroon: &Path,
    addr: SocketAddr,
) -> Result<(LightningClient, MacaroonData), CreateError> {
    let certificate = TLSCertificate::from_path(tls_cert)?;
    let macaroon = MacaroonData::from_file_path(macaroon)?;
    let config = Default::default();
    let tls = certificate.into_tls("localhost")?;
    let grpc_client = grpc::Client::new_expl(&addr, "localhost", tls, config)?;
    Ok((
        LightningClient::with_client(Arc::new(grpc_client)),
        macaroon,
    ))
}

pub fn init_default_lightning_client() -> Result<(LightningClient, MacaroonData), CreateError> {
    // TODO, don't hardcode.
    init_lightning_client(
        Path::new("/Volumes/btcchain/persist/lnd/tls.cert"),
        Path::new("/Volumes/btcchain/persist/lnd/data/chain/bitcoin/mainnet/admin.macaroon"),
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 10009),
    )
}

#[cfg(test)]
pub use crate::fake_db::db_with_account_a_balance;
