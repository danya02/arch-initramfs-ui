mod exits;
mod password_input;

use std::sync::{Arc, Mutex};

use cursive::{
    align::HAlign,
    theme::{BorderStyle, Palette, Theme},
    view::Margins,
    views, With,
};
use disk_crypto::params::EncryptionParams;

use crate::{
    exits::{
        choose_exit, BootMenuExitOption, LINUX_REBOOT_CMD_CAD_ON, LINUX_REBOOT_MAGIC1,
        LINUX_REBOOT_MAGIC2,
    },
    password_input::{input_switcher_thread, password_entry},
};

fn main_theme() -> Theme {
    Theme {
        shadow: true,
        borders: BorderStyle::Outset,
        palette: Palette::retro().with(|palette| {
            use cursive::theme::BaseColor::*;

            {
                // First, override some colors from the base palette.
                use cursive::theme::Color::TerminalDefault;
                use cursive::theme::PaletteColor::*;

                palette[Background] = Black.light();
                palette[View] = TerminalDefault;
                palette[Primary] = White.light();
                palette[TitlePrimary] = Blue.light();
                palette[Secondary] = Blue.light();
                palette[Highlight] = Blue.dark();
            }

            {
                // Then override some styles.
                use cursive::theme::Effect::*;
                use cursive::theme::PaletteStyle::*;
                use cursive::theme::Style;
                palette[Highlight] = Style::from(Blue.light()).combine(Bold);
            }
        }),
    }
}

fn password_input_theme() -> Theme {
    cursive::theme::Theme {
        shadow: true,
        borders: BorderStyle::Outset,
        palette: Palette::retro().with(|palette| {
            use cursive::theme::BaseColor::*;

            {
                //use cursive::theme::Color::TerminalDefault;
                use cursive::theme::PaletteColor;

                palette[PaletteColor::Background] = Red.light();
                palette[PaletteColor::View] = White.light();
                palette[PaletteColor::Primary] = White.light();
                palette[PaletteColor::TitlePrimary] = Red.light();
                palette[PaletteColor::Secondary] = Red.light();
                palette[PaletteColor::Highlight] = Red.dark();
            }

            {
                // Then override some styles.
                use cursive::theme::Effect;
                use cursive::theme::PaletteStyle;
                use cursive::theme::Style;
                palette[PaletteStyle::Highlight] = Style::from(Red.light()).combine(Effect::Bold);
            }
        }),
    }
}

pub enum LoginState {
    /// We have not logged in yet, and a password/pinentry is shown.
    WaitingForLogin,

    /// We have received a password or PIN, and are currently validating it.
    ValidatingLogin,

    /// We have logged in successfully, and the full menu is currently displayed.
    LogInOkay,

    /// We have failed to log in, and the reduced menu is currently displayed.
    LogInFail,
}

pub enum InputCredentials {
    /// The user typed a password
    Password(String),

    /// The user typed a PIN code for the Yubikey
    YkPin(String),
}

impl Default for LoginState {
    fn default() -> Self {
        Self::WaitingForLogin
    }
}

struct State {
    keyfile: Option<Vec<u8>>,
    login_state: Arc<Mutex<LoginState>>,
    config: EncryptionParams,
}

fn main() {
    println!("Boot menu launching!");

    // The first thing we need to do is to parse the encryption config.
    let config_txt = include_str!("../../disk-crypto/encrypt-config.json");
    let config: EncryptionParams = serde_json::from_str(config_txt)
        .expect("Compiled-in encryption JSON is invalid -- please rebuild boot-menu");

    // For ease of use, for the duration of the menu, we enable the CAD combination,
    // which will reboot instantly.
    // This does not compromise security if the BIOS menu is behind a password.
    // Also, errors are ignored here.
    let _ = unsafe {
        syscalls::syscall!(
            syscalls::Sysno::reboot,
            LINUX_REBOOT_MAGIC1,
            LINUX_REBOOT_MAGIC2,
            LINUX_REBOOT_CMD_CAD_ON,
            0
        )
    };

    let mut siv = cursive::CursiveRunnable::new(cursive::backends::termion::Backend::init);

    siv.set_theme(main_theme());
    let state = State {
        config,
        keyfile: None,
        login_state: Arc::new(Mutex::new(LoginState::default())),
    };
    let login_state = state.login_state.clone();
    siv.set_user_data(state);

    // Do not allow closing the app with ^C.
    siv.clear_global_callbacks(cursive::event::Event::CtrlChar('c'));

    // Immediately after this, spawn another layer. This will prompt the user for a password.
    password_entry(&mut siv);

    // Also spawn the input box switcher thread.
    let sink = siv.cb_sink().clone();
    std::thread::spawn(|| input_switcher_thread(sink, login_state));

    siv.run();
}
