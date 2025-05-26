use super::{X11_STATE, X11State};
use crate::prelude::*;

pub struct X11;

impl Platform for X11 {
    fn init(&self) -> anyhow::Result<()> {
        X11_STATE.set(X11State::create()?).map_err(|_| ()).unwrap();

        Ok(())
    }

    fn name(&self) -> &'static str {
        "X11"
    }
}
