use egui::Key::*;

#[derive(Default, Clone, Copy)]
pub(crate) struct SuperModifiers {
    pub caps_lock: bool,
    pub num_lock: bool,

    pub left_shift: bool,
    pub right_shift: bool,
    pub left_ctrl: bool,
    pub right_ctrl: bool,
    pub left_alt: bool,
    pub right_alt: bool,
}

impl Into<egui::Modifiers> for SuperModifiers {
    fn into(self) -> egui::Modifiers {
        egui::Modifiers {
            alt: self.left_alt || self.right_alt,
            ctrl: self.left_ctrl || self.right_ctrl,
            shift: self.left_shift || self.right_shift,
            mac_cmd: false,
            command: self.left_ctrl || self.right_ctrl,
        }
    }
}

pub(crate) enum KeyOrModifier {
    Key(egui::Key),
    Mod(SuperModifiers),
}

pub(crate) fn keycode_to_egui_key(keycode: u32, mods: SuperModifiers) -> Option<KeyOrModifier> {
    use KeyOrModifier::Mod;

    let key = match keycode {
        // This covers all keys on my keyboard, left to right
        // ---
        1 => Escape,

        59 => F1,
        60 => F2,
        61 => F3,
        62 => F4,
        63 => F5,
        64 => F6,
        65 => F7,
        66 => F8,
        67 => F9,
        68 => F10,
        87 => F11, // weird jump in numbers
        88 => F12,

        99 => return None,  // KEY_SYSRQ
        70 => return None,  // KEY_SCROLLLOCK
        119 => return None, // KEY_PAUSE

        // -----
        41 => return None, // KEY_GRAVE (`)
        2 => Num1,
        3 => Num2,
        4 => Num3,
        5 => Num4,
        6 => Num5,
        7 => Num6,
        8 => Num7,
        9 => Num8,
        10 => Num9,
        11 => Num0,
        12 => Minus,
        13 => PlusEquals,
        14 => Backspace,

        110 => Insert,
        102 => Home,
        104 => PageUp,

        69 => {
            return Some(Mod(SuperModifiers {
                num_lock: true,
                ..Default::default()
            }))
        } // KEY_NUMLOCK
        98 => return None, // KEY_KPSLASH
        55 => return None, // KEY_KPASTERISK
        74 => Minus,       // KEY_KPMINUS

        // -----
        15 => Tab,
        16 => Q,
        17 => W,
        18 => E,
        19 => R,
        20 => T,
        21 => Y,
        22 => U,
        23 => I,
        24 => O,
        25 => P,
        26 => return None, // KEY_LEFTBRACE
        27 => return None, // KEY_RIGHTBRACE
        43 => return None, // KEY_BACKSLASH

        111 => Delete,
        107 => End,
        109 => PageDown,

        71 => if_numlock_or(mods, Num7, Home),    // KEY_KP7
        72 => if_numlock_or(mods, Num8, ArrowUp), // KEY_KP8
        73 => if_numlock_or(mods, Num9, PageUp),  // KEY_KP9
        78 => PlusEquals,                         // KEY_KPPLUS

        // -----
        58 => {
            return Some(Mod(SuperModifiers {
                caps_lock: true,
                ..Default::default()
            }))
        } // KEY_CAPSLOCK
        30 => A,
        31 => S,
        32 => D,
        33 => F,
        34 => G,
        35 => H,
        36 => J,
        37 => K,
        38 => L,
        39 => return None, // KEY_SEMICOLON
        40 => return None, // KEY_APOSTROPHE
        28 => Enter,

        75 => if_numlock_or(mods, Num4, ArrowLeft), // KEY_KP4
        76 => if_numlock(mods, Num5)?,              // KEY_KP5,
        77 => if_numlock_or(mods, Num6, ArrowRight), // KEY_KP6

        // -----
        42 => {
            return Some(Mod(SuperModifiers {
                left_shift: true,
                ..Default::default()
            }))
        } // KEY_LEFTSHIFT
        44 => Z,
        45 => X,
        46 => C,
        47 => V,
        48 => B,
        49 => N,
        50 => M,
        51 => return None, // KEY_COMMA
        52 => return None, // KEY_DOT
        53 => return None, // KEY_SLASH
        54 => {
            return Some(Mod(SuperModifiers {
                right_shift: true,
                ..Default::default()
            }))
        } // KEY_RIGHTSHIFT

        103 => ArrowUp,

        79 => if_numlock_or(mods, Num1, End),       // KEY_KP1
        80 => if_numlock_or(mods, Num2, ArrowDown), // KEY_KP2
        81 => if_numlock_or(mods, Num3, PageDown),  // KEY_KP3
        96 => Enter,                                // KEY_KPENTER

        // -----
        29 => {
            return Some(Mod(SuperModifiers {
                left_ctrl: true,
                ..Default::default()
            }))
        } // KEY_LEFTCTRL
        125 => return None, // KEY_LEFTMETA
        56 => {
            return Some(Mod(SuperModifiers {
                left_alt: true,
                ..Default::default()
            }))
        } // KEY_LEFTALT
        57 => Space,
        100 => {
            return Some(Mod(SuperModifiers {
                right_alt: true,
                ..Default::default()
            }))
        } // KEY_RIGHTALT
        127 => return None, // KEY_COMPOSE
        97 => {
            return Some(Mod(SuperModifiers {
                right_ctrl: true,
                ..Default::default()
            }))
        } // KEY_RIGHTCTRL

        105 => ArrowLeft,
        108 => ArrowDown,
        106 => ArrowRight,

        82 => if_numlock_or(mods, Num0, Insert), // KEY_KP0
        83 => if_not_numlock(mods, Delete)?,     // KEY_KPDOT
    };
    Some(KeyOrModifier::Key(key))
}

// Key if numlock, or other key
fn if_numlock_or<T>(mods: SuperModifiers, if_numlock: T, or_else: T) -> T {
    if mods.num_lock {
        if_numlock
    } else {
        or_else
    }
}

/// Key if numlock, or nothing
fn if_numlock<T>(mods: SuperModifiers, if_numlock: T) -> Option<T> {
    mods.num_lock.then_some(if_numlock)
}

/// Key if not numlock, or nothing
fn if_not_numlock<T>(mods: SuperModifiers, if_numlock: T) -> Option<T> {
    (!mods.num_lock).then_some(if_numlock)
}

pub(crate) fn keycode_to_text(keycode: u32, mods: SuperModifiers) -> Option<&'static str> {
    let none = "";

    let text = match keycode {
        // This covers all keys on my keyboard, left to right
        // ---
        1 => none, // Escape

        59..=68 | 87 | 88 => none, // F1..F12

        99 => none,  // KEY_SYSRQ
        70 => none,  // KEY_SCROLLLOCK
        119 => none, // KEY_PAUSE

        // -----
        41 => shift(mods, "~", "`"), // KEY_GRAVE (`)
        2 => shift(mods, "!", "1"),
        3 => shift(mods, "@", "2"),
        4 => shift(mods, "#", "3"),
        5 => shift(mods, "$", "4"),
        6 => shift(mods, "%", "5"),
        7 => shift(mods, "^", "6"),
        8 => shift(mods, "&", "7"),
        9 => shift(mods, "*", "8"),
        10 => shift(mods, "(", "9"),
        11 => shift(mods, ")", "0"),
        12 => shift(mods, "_", "-"),
        13 => shift(mods, "-", "+"),
        14 => none, // backspace

        110 => none, // insert
        102 => none, // backspace
        104 => none, // page up

        69 => none, // KEY_NUMLOCK
        98 => "/",  // KEY_KPSLASH
        55 => "*",  // KEY_KPASTERISK
        74 => "-",  // KEY_KPMINUS

        // -----
        15 => "\t",
        16 => caps(mods, "Q", "q"),
        17 => caps(mods, "W", "w"),
        18 => caps(mods, "E", "e"),
        19 => caps(mods, "R", "r"),
        20 => caps(mods, "T", "t"),
        21 => caps(mods, "Y", "y"),
        22 => caps(mods, "U", "u"),
        23 => caps(mods, "I", "i"),
        24 => caps(mods, "O", "o"),
        25 => caps(mods, "P", "p"),
        26 => shift(mods, "{", "["),  // KEY_LEFTBRACE
        27 => shift(mods, "}", "]"),  // KEY_RIGHTBRACE
        43 => shift(mods, "|", "\\"), // KEY_BACKSLASH

        111 => none, // delete
        107 => none, // end
        109 => none, // page down

        71 => if_numlock(mods, "7")?, // KEY_KP7
        72 => if_numlock(mods, "8")?, // KEY_KP8
        73 => if_numlock(mods, "9")?, // KEY_KP9
        78 => "+",                    // KEY_KPPLUS

        // -----
        58 => none, // KEY_CAPSLOCK
        30 => caps(mods, "A", "a"),
        31 => caps(mods, "S", "s"),
        32 => caps(mods, "D", "d"),
        33 => caps(mods, "F", "f"),
        34 => caps(mods, "G", "g"),
        35 => caps(mods, "H", "h"),
        36 => caps(mods, "J", "j"),
        37 => caps(mods, "K", "k"),
        38 => caps(mods, "L", "l"),
        39 => shift(mods, ":", ";"),  // KEY_SEMICOLON
        40 => shift(mods, "\"", "'"), // KEY_APOSTROPHE
        28 => none,

        75 => if_numlock(mods, "4")?, // KEY_KP4
        76 => if_numlock(mods, "5")?, // KEY_KP5,
        77 => if_numlock(mods, "6")?, // KEY_KP6

        // -----
        42 => none, // KEY_LEFTSHIFT
        44 => caps(mods, "Z", "z"),
        45 => caps(mods, "X", "x"),
        46 => caps(mods, "C", "c"),
        47 => caps(mods, "V", "v"),
        48 => caps(mods, "B", "b"),
        49 => caps(mods, "N", "n"),
        50 => caps(mods, "M", "m"),
        51 => shift(mods, "<", ","), // KEY_COMMA
        52 => shift(mods, ">", "."), // KEY_DOT
        53 => shift(mods, "?", "/"), // KEY_SLASH
        54 => none,                  // KEY_RIGHTSHIFT

        103 => none,

        79 => if_numlock(mods, "1")?, // KEY_KP1
        80 => if_numlock(mods, "2")?, // KEY_KP2
        81 => if_numlock(mods, "3")?, // KEY_KP3
        96 => none,                   // KEY_KPENTER

        // -----
        29 => none,  // KEY_LEFTCTRL
        125 => none, // KEY_LEFTMETA
        56 => none,  // KEY_LEFTALT
        57 => " ",
        100 => none, // KEY_RIGHTALT
        127 => none, // KEY_COMPOSE
        97 => none,  // KEY_RIGHTCTRL

        105 => none,
        108 => none,
        106 => none,

        82 => if_numlock(mods, "0")?, // KEY_KP0
        83 => if_numlock(mods, ".")?, // KEY_KPDOT
    };

    if text.len() == 0 {
        return None;
    }

    Some(text)
}

/// Return one of the items if typing capital letters, the other if not.
///
/// Capital letters are if shift is pressed, or if caps lock, but not when both
fn caps<T>(mods: SuperModifiers, if_big: T, if_small: T) -> T {
    let do_big = (mods.caps_lock) ^ (mods.left_shift || mods.right_shift);
    if do_big {
        if_big
    } else {
        if_small
    }
}

/// Return one of the items if typing shifted letters, the other if not.
///
/// Shifted letters are not affected by caps lock, only by shift.
fn shift<T>(mods: SuperModifiers, if_big: T, if_small: T) -> T {
    let do_big = mods.left_shift || mods.right_shift;
    if do_big {
        if_big
    } else {
        if_small
    }
}
