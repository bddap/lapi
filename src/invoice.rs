use crate::common::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Invoice {
    target: Lesser,
    amount: Satoshis,
}

impl Invoice {
    pub fn amount(&self) -> Satoshis {
        self.amount.clone()
    }
}

#[derive(Clone)]
pub enum InvoiceStatus {
    Paid,
    Unpaid,
}

pub struct PaidInvoice(pub Invoice);
