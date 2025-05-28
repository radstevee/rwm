use std::sync::Mutex;

use super::{X11_STATE, X11State};
use crate::prelude::*;

pub struct X11;

impl Platform for X11 {
    fn init(&self) -> Result<()> {
        X11_STATE.set(Mutex::new(X11State::create()?)).map_err(|_| ()).unwrap();
        let state = X11State::state();
        debug!("Monitors: {:#?}", state.monitors.clone());

        Ok(())
    }

    fn name(&self) -> &'static str {
        "X11"
    }

    fn run(&self) -> Result<()> {
        X11State::state().event_loop()
    }
}
