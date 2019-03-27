use crate::common::*;
use warp::Filter;

pub fn serve() -> Result<(), ServeError> {
    let api = Api {
        database: FakeDb::new(),
        lighting_node: init_default_lightning_client().map_err(ServeError::Create)?,
    };

    let generate_invoice = warp::post(warp::path("invoice"))
        .and(warp::filters::body::json())
        .map(
            |GenerateInvoiceRequest { lesser, satoshis }| -> GenerateInvoiceResponse {
                unimplemented!()
            },
        );

    // let handler = warp::post2().and(generate_invoice);

    Ok(())
}

#[derive(Debug)]
pub enum ServeError {
    Create(CreateError),
}
