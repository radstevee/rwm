use x11rb::protocol::xproto::{ConnectionExt, KeyButMask, Keycode, ModMask};

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

fn decompose(mask: KeyButMask) -> Vec<KeyButMask> {
    let variants = [
        KeyButMask::SHIFT,
        KeyButMask::LOCK,
        KeyButMask::CONTROL,
        KeyButMask::MOD1,
        KeyButMask::MOD2,
        KeyButMask::MOD3,
        KeyButMask::MOD4,
        KeyButMask::MOD5,
        KeyButMask::BUTTON1,
        KeyButMask::BUTTON2,
        KeyButMask::BUTTON3,
        KeyButMask::BUTTON4,
        KeyButMask::BUTTON5,
    ];

    let mut result = Vec::new();
    for variant in variants.iter() {
        if mask.bits() & variant.bits() != 0 {
            result.push(*variant);
        }
    }

    result
}

pub fn find_keybind_action_for(
    code: Keycode,
    mask: KeyButMask,
    state: &X11State,
) -> Option<KeybindAction> {
    let keysym = find_keysym(code, state);
    let keysym = match keysym {
        Ok(keysym) => keysym,
        Err(e) => {
            error!("failed finding keysym for {code}: {e}");
            return None;
        }
    };

    let pressed_mods = decompose(mask);
    info!("pressed {keysym} with mods {mask:?}: {pressed_mods:?}");

    for keybind in config().bindings() {
        let mods = keybind
            .modifiers()
            .clone()
            .unwrap_or_else(|| vec![config().keyboard().mod_key().clone()])
            .into_iter()
            .map(|modifier| match &*modifier {
                "shift" => KeyButMask::SHIFT,
                "control" | "ctrl" => KeyButMask::CONTROL,
                "alt" => KeyButMask::MOD1,
                "super" | "meta" | "windows" | "win" => KeyButMask::MOD4,

                _ => unreachable!(), // config should already have been validated
            })
            .collect::<Vec<KeyButMask>>();

        if mods == pressed_mods && keybind.key() == keysym {
            return Some(keybind.action().clone());
        }
    }

    None
}

fn find_keysym(code: Keycode, state: &X11State) -> Result<char> {
    let cookie = state.conn.get_keyboard_mapping(code, 1).unwrap();
    let reply = cookie.reply()?;

    if let Some(&keysym) = reply.keysyms.get(0) {
        if let Some(char) = keysym_to_char(keysym) {
            Ok(char)
        } else {
            Err(anyhow!("unknown keysym: {keysym}").into())
        }
    } else {
        Err(anyhow!("unknown keysym: {code}").into())
    }
}

fn keysym_to_char(keysym: u32) -> Option<char> {
    if keysym >= 0x20 && keysym <= 0x7E {
        Some(keysym as u8 as char)
    } else {
        None
    }
}
