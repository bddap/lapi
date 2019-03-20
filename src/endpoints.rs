use crate::common::*;
use futures::future::FutureResult;
use futures::Future;

struct Api<D: Db, L: LightningNode> {
    database: D,
    lighting_node: L,
}

impl<D: Db, L: LightningNode> Api<D, L> {
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
        let amount = invoice
            .amount_pico_btc()
            .ok_or(PayInvoiceError::Pay(PayError::NoAmount))
            .and_then(|pico| Satoshis::from_pico_btc(pico).map_err(PayInvoiceError::Convert));
        FutureResult::from(amount)
            .and_then(move |amount| {
                self.database
                    .begin_withdrawal(master, amount)
                    .map_err(PayInvoiceError::Begin)
            })
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

#[derive(Debug, Clone)]
pub enum GenerateInvoiceError {
    Create(CreateInvoiceError),
    Store(StoreInvoiceError),
}

#[derive(Debug, Clone)]
pub enum PayInvoiceError {
    Convert(NotDivisible),
    Begin(BeginWithdrawalError),
    Pay(PayError),
    Finish(FinishWithdrawalError),
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::{thread_rng, Rng};

    fn gen_auth() -> (Master, Middle, Lesser) {
        let master: Master = thread_rng().gen();
        let middle: Middle = master.into();
        let lesser: Lesser = middle.into();
        (master, middle, lesser)
    }

    fn generate_invoice<D: Db, L: LightningNode>(api: Api<D, L>) {
        api.generate_invoice(gen_auth().2, Satoshis(1))
            .wait()
            .unwrap();
    }

    fn pay_invoice<D: Db, L: LightningNode>(api: Api<D, L>) {
        let (master, middle, lesser) = gen_auth();
        let invoice = api.generate_invoice(lesser, Satoshis(1)).wait().unwrap();
        api.pay_invoice(master, invoice).wait().unwrap();
    }

    fn check_balance<D: Db, L: LightningNode>(api: Api<D, L>) {
        // assert lesser has no balance
        // pay invoice of n satoshis to lesser
        // assert lesser has n balance
    }

    fn check_invoice_status<D: Db, L: LightningNode>(api: Api<D, L>) {
        // assert lesser balance is Nonexistent
        // create invoice for n satoshis with lesser target
        // assert invoice status NonExistent
        // add invoice to db as unpaid
        // assert invoice status unpaid
        // pay invoice
        // assert invoice status paid
        // assert balance for lesser is n
    }

    fn check_invoice_status_duo<D: Db, L: LightningNode>(api: Api<D, L>) {
        // create two users, A and B
        // assert {A,B}lesser balance is Nonexistent
        // create Ainvoice for n satoshis with Alesser target
        // assert {A,B}invoice status NonExistent
        // add invoice to db as unpaid
        // assert Ainvoice status unpaid
        // assert Binvoice status NonExistent
        // pay Ainvoice
        // assert Ainvoice status paid
        // assert balance for Alesser is n
        // assert balance for Blesser is NonExistent
    }

    #[test]
    fn paid_invoice_increases_balance() {
        // generate invoice for n sat
        // check balance is 0
        // make invoice paid somehow
        // check balance is n sat
        assert!(false, "test not implemented");
    }

    #[test]
    fn pay_invoice_to_local() {
        // A account has n+k satoshis
        // B account has m satoshis
        // B generates invoice for n satoshis
        // A fills invoice
        // assert B account has n+m satoshis
        assert!(false, "test not implemented");
    }

    #[test]
    fn pay_invoice_to_local_to_self() {
        // A account has n satoshis
        // A generates invoice for n satoshis
        // Assert invoice is not Filled
        // A fills invoice
        // Assert invoice is Filled
        // Assert A account has n satoshis
        assert!(false, "test not implemented");
    }

    #[test]
    fn fdb_fnode() {
        // run all tests agianst fake db and fake node.
        panic!("test not implemented");
    }

    #[test]
    fn rdb_rnode() {
        // run all tests agianst real db and real node.
        panic!("test not implemented");
    }

    #[test]
    fn rdb_fnode() {
        // run all tests agianst real db and fake node.
        panic!("test not implemented");
    }

    #[test]
    fn fdb_rnode() {
        // run all tests agianst fake db and real node.
        panic!("test not implemented");
    }
}
