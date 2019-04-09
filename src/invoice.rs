use crate::common::*;
pub use lightning_invoice::{Invoice, Sha256};
use lightning_invoice::{ParseOrSemanticError, SignedRawInvoice};
use std::borrow::Borrow;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InvoiceStatus {
    Paid(PaidInvoice),
    Unpaid(Invoice),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PaidInvoice {
    invoice: Invoice,
    preimage: Preimage,
    amount_paid: Satoshis,
}

impl PaidInvoice {
    pub fn create(
        invoice: Invoice,
        preimage: Preimage,
        amount_paid: Satoshis,
    ) -> Result<PaidInvoice, PaidInvoiceInvalid> {
        let amount_requested_pico = invoice.amount_pico_btc().unwrap_or(0);
        // round amount requested up to the nearest whole satoshi
        let amount_requested = Satoshis::from_pico_btc(amount_requested_pico)
            .unwrap_or_else(|NotDivisible { whole, change: _ }| whole + Satoshis(1));

        if preimage.hash() != get_payment_hash(&invoice) {
            Err(PaidInvoiceInvalid::PreimageMismatch)
        } else if amount_requested > amount_paid {
            Err(PaidInvoiceInvalid::AmountTooSmall)
        } else if amount_requested.saturating_mul(Satoshis(2)) < amount_paid {
            Err(PaidInvoiceInvalid::AmountTooLarge)
        } else {
            Ok(PaidInvoice {
                invoice,
                preimage,
                amount_paid,
            })
        }
    }

    pub fn invoice(&self) -> &Invoice {
        &self.invoice
    }

    pub fn preimage(&self) -> &Preimage {
        &self.preimage
    }

    pub fn amount_paid(&self) -> &Satoshis {
        &self.amount_paid
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PaidInvoiceInvalid {
    PreimageMismatch,
    AmountTooSmall,
    AmountTooLarge,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PaidInvoiceOutgoing {
    pub paid_invoice: PaidInvoice,
    pub fees_offered: Fee<Satoshis>,
    pub fees_paid: Fee<Satoshis>,
}

pub fn get_payment_hash(invoice: &Invoice) -> PaymentHash {
    let sl: &[u8] = invoice.payment_hash().0.borrow();
    debug_assert_eq!(sl.len(), 32);
    U256::try_from_slice(sl).unwrap()
}

pub fn parse_bolt11(encoded: &str) -> Result<Invoice, ParseOrSemanticError> {
    let raw = encoded.parse::<SignedRawInvoice>()?;
    let invoice = Invoice::from_signed(raw)?;
    Ok(invoice)
}

pub fn to_bolt11(invoice: &Invoice) -> String {
    invoice.to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_then_deserialize() {
        let some_raw_invoice =
            "lnbc420n1pwf2rsfpp5cakf9e6fvcreyywflk0p9wekl4whwk6qm2ge05g2vhjl5ae0gj5qdpsd3h8x6pwwpmj\
             qmrfde6hsgrrdah8gctfdejhygrxdaezqvtgxqzfvcqp2rzjq2psxxpvnzza4yankfwfvgwj9ne5ga0x8sfrjs\
             hyq244xrq92mn82zyt6yqqgksqqqqqqqqqqqqqqeqqjq7fxyyw5d63ghg4lau9v5zeuttswjlcsprf44y2rv2p\
             c5ew0wr67kzs27gaycuxhz7eex4l92fywd2k44nw9eck4k6eqh394y3kclqssp7yersm";
        let invoice = parse_bolt11(some_raw_invoice).unwrap();
        assert_eq!(&to_bolt11(&invoice), some_raw_invoice);
    }

    #[test]
    fn invalid_bolt11() {
        let some_raw_invoice_invalid =
            "lnbc420n1pwf2rsfpp5cakf9e6fvcreyywflk0p9wekl4whwk6qm2ge05g2vhjl5ae0gj5qdpsd3h8x6pwwpmj\
             qmrfde6hsgrrdah8gctfdejhygrxdaezqvtgxqzfvcqp2rzjq2psxxpvnzza4yankfwfvgwj9ne5ga0x8sfrjs\
             hyq244xrq92mn82zyt6yqqgksqqqqqqqqqqqqqqeqqjq7fxyyw5d63ghg4lau9v5zeuttswjlcsprf44y2rv2p\
             c5ew0wr67kzs27gaycuxhz7eex4l92fywd2k44nw9eck4k6eqh394y3kclqssp7yersa";
        parse_bolt11(some_raw_invoice_invalid).unwrap_err();
    }
}
