use crate::common::*;
use futures::future::FutureResult;

pub struct LightningNode {}

impl LightningNode {
    pub fn create_invoice(&self, satoshis: Satoshis) -> FutureResult<Invoice, CreateInvoiceError> {
        unimplemented!()
    }

    pub fn pay_invoice(&self, invoice: Invoice) -> FutureResult<PaidInvoice, PayError> {
        unimplemented!()
    }
}

pub enum CreateInvoiceError {}

pub enum PayError {}
