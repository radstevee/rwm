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

        if mods != pressed_mods {
            continue;
        }
        
        if keybind.key() == &keysym {
            return Some(keybind.action().clone());
        }
    }

    None
}

fn find_keysym(code: Keycode, state: &X11State) -> Result<String> {
    let cookie = state.conn.get_keyboard_mapping(code, 1).unwrap();
    let reply = cookie.reply()?;

    if let Some(&keysym) = reply.keysyms.first() {
        if let Some(keysym) = decode_keysym(keysym, code) {
            Ok(keysym)
        } else {
            Err(anyhow!("unknown keysym: {keysym}").into())
        }
    } else {
        Err(anyhow!("unknown keysym: {code}").into())
    }
}

fn decode_keysym(keysym: u32, keycode: Keycode) -> Option<String> {
    if keycode == 36 {
        return Some("return".to_string())
    }
 
    if (0x20..=0x7E).contains(&keysym) {
        Some((keysym as u8 as char).to_string())
    } else {
        None
    }
}
