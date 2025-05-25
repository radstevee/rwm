use crate::platform::Platform;

use super::{X11State, X11_STATE};

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