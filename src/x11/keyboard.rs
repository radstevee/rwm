use x11rb::protocol::xproto::ModMask;

use crate::prelude::*;

pub fn mod_mask() -> ModMask {
    let key = config().keyboard().mod_key();

    match key.as_str() {
        "alt" => ModMask::M1,                                // Mod1Mask
        "ctrl" => ModMask::CONTROL,                          // ControlMask
        "super" | "meta" | "windows" | "win" => ModMask::M4, // Mod4Mask

        _ => unreachable!(), // config should have been already validated
    }
}
