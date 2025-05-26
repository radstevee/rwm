use std::fmt::{self, Debug, Formatter};

use crate::prelude::*;

#[derive(Clone, Copy, PartialEq, Getters, PartialOrd)]
#[constructor(named(new), fields(symbol, name, manage_fn))]
pub struct Layout {
    symbol: &'static str,
    name: &'static str,
    manage_fn: fn(Tag, Monitor),
}

impl Debug for Layout {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ('{}')", self.name, self.symbol)
    }
}

pub fn test_layout(_tag: Tag, _mon: Monitor) {}
