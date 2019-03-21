use crate::common::*;
use futures::future::FutureResult;
use grpc::{Client, ClientStub};
use lnd_rust::{rpc_grpc::LightningClient, tls_certificate::TLSCertificate};
use std::{io, net::SocketAddr, path::Path, sync::Arc};

pub fn init_lightning_client(
    tls_cert: &Path,
    addr: SocketAddr,
) -> Result<LightningClient, CreateError> {
    let certificate = TLSCertificate::from_path(tls_cert)?;
    let config = Default::default();
    let tls = certificate.into_tls("localhost")?;
    let grpc_client = grpc::Client::new_expl(&addr, "localhost", tls, config)?;
    Ok(LightningClient::with_client(Arc::new(grpc_client)))
}

impl LightningNode for LightningClient {
    fn create_invoice(&self, satoshis: Satoshis) -> FutureResult<Invoice, CreateInvoiceError> {
        unimplemented!()
    }

    fn pay_invoice(&self, invoice: Invoice) -> FutureResult<PaidInvoice, PayError> {
        unimplemented!()
    }
}

// Error initializing an LndClient
#[derive(Debug)]
pub enum CreateError {
    Io(io::Error),
    Tls(tls_api::Error),
    Grpc(grpc::Error),
}

impl From<io::Error> for CreateError {
    fn from(other: io::Error) -> Self {
        CreateError::Io(other)
    }
}

impl From<tls_api::Error> for CreateError {
    fn from(other: tls_api::Error) -> Self {
        CreateError::Tls(other)
    }
}

impl From<grpc::Error> for CreateError {
    fn from(other: grpc::Error) -> Self {
        CreateError::Grpc(other)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use grpc::{Metadata, RequestOptions};
    use lnd_rust::rpc::GetInfoRequest;
    use lnd_rust::rpc_grpc::Lightning;
    use std::net::{IpAddr, SocketAddr};

    #[test]
    fn info() {
        let ip_addr: String = include_str!("../lndauth/ip")
            .chars()
            .filter(|c| c != &'\n')
            .collect();
        let ip: IpAddr = ip_addr.parse().unwrap();
        let socket_addr = SocketAddr::new(ip, 10009);
        let client = init_lightning_client(Path::new("lndauth/tls.cert"), socket_addr).unwrap();
        // let metadata = Metadata::new();
        let requestoptions = RequestOptions::new();
        let getinforequest = GetInfoRequest::new();
        let fut_fut_response = client.get_info(requestoptions, getinforequest);

        let (metadata_pre, fut_response, metadata_post) = fut_fut_response.wait().unwrap();
    }
}
