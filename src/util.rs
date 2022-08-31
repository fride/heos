use std::sync::{Arc, RwLock};

pub struct Shared<R> {
    state: Arc<RwLock<R>>,
}
impl<R> Clone for Shared<R> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<R> Shared<R> {
    pub fn new(state: R) -> Self {
        Self {
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn with_state<H, T>(&self, handler: H) -> T
    where
        H: FnOnce(&R) -> T,
    {
        let read = self.state.read().unwrap();
        let r = handler(&read);
        r
    }
    pub fn with_mutable_state<H, T>(&self, handler: H) -> T
    where
        H: FnOnce(&mut R) -> T,
    {
        let mut write = self.state.write().unwrap();
        let r = handler(&mut write);
        r
    }
}

impl<R> ToString for Shared<R>
where
    R: ToString,
{
    fn to_string(&self) -> String {
        self.with_state(|state| state.to_string())
    }
}
