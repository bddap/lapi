use futures::Future;

pub type DynFut<I, E> = Box<Future<Item = I, Error = E> + Send>;
