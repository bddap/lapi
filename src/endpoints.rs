use crate::common::*;
use futures::Future;

struct Api<D: Db> {
    database: D,
    lighting_node: LightningNode,
}

impl<D: Db> Api<D> {
    pub fn generate_invoice<'a>(
        &'a self,
        lesser: Lesser,
        satoshis: Satoshis,
    ) -> impl Future<Item = Invoice, Error = GenerateInvoiceError> + 'a {
        self.lighting_node
            .create_invoice(satoshis)
            .map_err(GenerateInvoiceError::Create)
            .and_then(move |untracked_invoice| {
                self.database
                    .store_unpaid_invoice(lesser, untracked_invoice)
                    .map_err(GenerateInvoiceError::Store)
            })
    }

    pub fn pay_invoice<'a>(
        &'a self,
        master: Master,
        invoice: Invoice,
    ) -> impl Future<Item = (), Error = PayInvoiceError> + 'a {
        self.database
            .begin_withdrawal(master, invoice.amount())
            .map_err(PayInvoiceError::Begin)
            .and_then(move |()| {
                self.lighting_node
                    .pay_invoice(invoice)
                    .map_err(PayInvoiceError::Pay)
            })
            .and_then(move |paid_invoice| {
                self.database
                    .finish_withdrawal(paid_invoice)
                    .map_err(PayInvoiceError::Finish)
            })
    }

    pub fn check_balance<'a>(
        &'a self,
        middle: Middle,
    ) -> impl Future<Item = Satoshis, Error = CheckBalanceError> + 'a {
        self.database.check_balance(middle)
    }

    pub fn check_invoice_status<'a>(
        &'a self,
        middle: Middle,
        invoice: Invoice,
    ) -> impl Future<Item = InvoiceStatus, Error = CheckInvoiceStatusError> + 'a {
        self.database.check_invoice_status(middle, invoice)
    }
}

pub enum GenerateInvoiceError {
    Create(CreateInvoiceError),
    Store(StoreInvoiceError),
}

pub enum PayInvoiceError {
    Begin(BeginWithdrawalError),
    Pay(PayError),
    Finish(FinishWithdrawalError),
}
