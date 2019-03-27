use crate::common::{Log, LogErr};

pub struct FakeLog;

impl Log for FakeLog {
    fn _err(&self, err: LogErr) {
        panic!("{:#?}", err);
    }
}
