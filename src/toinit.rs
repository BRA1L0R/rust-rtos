use core::marker::PhantomData;

pub struct NoToken(());
pub struct ToInit<T, A = NoToken> {
    inner: Option<T>,
    _token: PhantomData<A>,
}

impl<T, A> ToInit<T, A> {
    pub const fn uninit() -> Self {
        Self {
            inner: None,
            _token: PhantomData,
        }
    }

    pub fn init(&mut self, val: T) {
        self.inner.replace(val);
    }

    pub fn access(&self, _token: &A) -> &T {
        self.inner.as_ref().unwrap()
    }

    pub fn access_mut(&mut self, _token: &A) -> &mut T {
        self.inner.as_mut().unwrap()
    }

    pub fn unwrap(&self) -> &T {
        self.inner.as_ref().unwrap()
    }

    pub fn unwrap_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }

    pub fn expect(&self, message: &str) -> &T {
        self.inner.as_ref().expect(message)
    }

    pub fn expect_mut(&mut self, message: &str) -> &mut T {
        self.inner.as_mut().expect(message)
    }
}

impl<T, A> Default for ToInit<T, A> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _token: Default::default(),
        }
    }
}
