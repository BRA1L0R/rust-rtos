pub struct AbiArguments<const U: usize> {
    args: [u32; 4],
}

impl<const U: usize> AbiArguments<U> {
    pub fn pushed(&self) -> &[u32] {
        &self.args[..U]
    }
}

impl AbiArguments<0> {
    pub fn new() -> Self {
        Self {
            args: Default::default(),
        }
    }
}

macro_rules! impl_ra {
    ($ra:tt, $current:expr) => {
        #[allow(dead_code)]
        impl $ra<$current> {
            #[inline(always)]
            pub fn push(self, arg: u32) -> $ra<{ $current + 1 }> {
                let $ra { mut args } = self;
                args[$current] = arg;
                $ra { args }
            }
        }
    };
}

// workaround until
// const exprs are stable to use
impl_ra!(AbiArguments, 0);
impl_ra!(AbiArguments, 1);
impl_ra!(AbiArguments, 2);
