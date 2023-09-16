use std::{
    process::Stdio,
    sync::{Arc, Mutex},
};

use cursive::{
    view::Nameable,
    views::{self},
    Cursive,
};

use crate::{
    exits::{full_menu, partial_menu},
    LoginState, State,
};

/// This thread is responsible for switching between the two different types of password entry.
///
/// - If we are LoginState::WaitingForLogin, and there is no Yubikey present, ensure that the password entry dialog is on top.
/// - If we are LoginState::WaitingForLogin, and there is a Yubikey present, ensure that the PIN entry dialog is on top.
pub fn input_switcher_thread(cb_sink: cursive::CbSink, state: Arc<Mutex<LoginState>>) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs_f32(0.1f32));
        // Check that the state is currently WaitingForLogin
        if !matches!(*state.lock().unwrap(), LoginState::WaitingForLogin) {
            continue;
        }
        // If the state is LogInOkay, then we will never return here,
        // so break out of the loop.
        if matches!(*state.lock().unwrap(), LoginState::LogInOkay) {
            break;
        }

        // Check for the presence of a Yubikey.

        let child = std::process::Command::new("ykinfo")
            .arg("-s")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let output = child.wait_with_output().unwrap();
        if output.status.success() {
            // Yubikey is present.
            cb_sink
                .send(Box::new(|siv| {
                    // Check that we are really waiting for the password
                    let state: &mut State = siv.user_data().unwrap();
                    if !matches!(
                        *state.login_state.lock().unwrap(),
                        LoginState::WaitingForLogin
                    ) {
                        return;
                    }

                    // Check that there exists a box called "ykpin_input"
                    let is_ykpin = siv
                        .call_on_name("ykpin_input", |_view: &mut views::Dialog| ())
                        .is_some();

                    if !is_ykpin {
                        // If not, then the top layer is wrong.
                        // Pop it and put the Ykpin layer there.
                        siv.pop_layer();

                        yubikey_pinentry(siv);
                    }
                }))
                .unwrap();
        } else {
            // Yubikey is absent.
            cb_sink
                .send(Box::new(|siv| {
                    // Check that we are really waiting for the password
                    let state: &mut State = siv.user_data().unwrap();
                    if !matches!(
                        *state.login_state.lock().unwrap(),
                        LoginState::WaitingForLogin
                    ) {
                        return;
                    }

                    // Check that there exists a box called "password_input"
                    let is_password = siv
                        .call_on_name("password_input", |_view: &mut views::Dialog| ())
                        .is_some();
                    if !is_password {
                        // If not, then the top layer is wrong.
                        // Pop it and put the Yubikey layer there.
                        siv.pop_layer();

                        password_entry(siv);
                    }
                }))
                .unwrap();
        }
    }
}

/// This function pushes a dialog layer that prompts for a password.
pub fn password_entry(siv: &mut Cursive) {
    siv.add_layer(
        views::Dialog::new()
            .title("Please enter password to continue...")
            .content(views::LinearLayout::vertical().child({
                let mut edit = views::EditView::new();
                edit.set_secret(true);
                edit.set_on_submit(|siv, text| {
                    // TODO: right now we do not do anything with the password we read.
                    // Later, this should be validated, e.g. for decrypting the root partition.
                    let data: &mut State = siv.user_data().unwrap();
                    let mut stateref = data.login_state.lock().unwrap();
                    *stateref = LoginState::ValidatingLogin;
                    drop(stateref);

                    let config = data.config.clone();

                    // Remove the password entry box, and show a "waiting" box,
                    // and in a thread start verifying the result.
                    siv.pop_layer();
                    siv.add_layer(views::Dialog::around(views::TextView::new(
                        "Verifying password...",
                    )));

                    let cb_sink = siv.cb_sink().clone();
                    let pw = text.to_string();
                    std::thread::spawn(move || match config.try_keyfile_from_password(pw) {
                        Ok(keyfile) => {
                            cb_sink
                                .send(Box::new(|siv| {
                                    let data: &mut State = siv.user_data().unwrap();

                                    // Set the state to be logged in, and save the keyfile contents.
                                    data.keyfile = Some(keyfile);
                                    *data.login_state.lock().unwrap() = LoginState::LogInOkay;

                                    // Pop the waiting dialog, and draw the full menu.
                                    siv.pop_layer();
                                    siv.add_layer(full_menu());
                                }))
                                .unwrap();
                        }
                        Err(_) => {
                            cb_sink
                                .send(Box::new(|siv| {
                                    let data: &mut State = siv.user_data().unwrap();

                                    // Set the state to be failed.
                                    *data.login_state.lock().unwrap() = LoginState::LogInFail;

                                    // Pop the waiting dialog, then draw the reduced menu,
                                    // and on top of that draw an error message.
                                    siv.pop_layer();
                                    siv.add_layer(partial_menu());
                                    siv.add_layer(
                                        views::Dialog::around(views::TextView::new(
                                            "Failed to unlock with password",
                                        ))
                                        .title("Error")
                                        .dismiss_button("OK"),
                                    )
                                }))
                                .unwrap();
                        }
                    });
                });
                edit
            }))
            .with_name("password_input"),
    )
}

/// This function pushes a dialog layer that prompts for a Yubikey pin
pub fn yubikey_pinentry(siv: &mut Cursive) {
    siv.add_layer(
        views::Dialog::new()
            .title("Please enter PIN to continue...")
            .content(views::LinearLayout::vertical().child({
                let mut edit = views::EditView::new();
                edit.set_secret(true);
                edit.set_on_submit(|siv, text| {
                    // TODO: right now we do not do anything with the password we read.
                    // Later, this should be validated, e.g. for decrypting the root partition.
                    let data: &mut State = siv.user_data().unwrap();
                    let mut stateref = data.login_state.lock().unwrap();
                    *stateref = LoginState::ValidatingLogin;
                    drop(stateref);

                    let config = data.config.clone();

                    // Remove the password entry box, and show a "waiting" box,
                    // and in a thread start verifying the result.
                    siv.pop_layer();
                    siv.add_layer(views::Dialog::around(views::TextView::new(
                        "Verifying PIN code; you may need to touch your Yubikey now...",
                    )));

                    let cb_sink = siv.cb_sink().clone();
                    let pw = text.to_string();
                    std::thread::spawn(move || {
                        // We need to try the Yubikey a couple times,
                        // because only one process may use it at one time,
                        // and the detection thread could be still running its copy.
                        let resp = config
                            .try_keyfile_from_pin(pw.clone())
                            .or_else(|_| config.try_keyfile_from_pin(pw.clone()))
                            .or_else(|_| config.try_keyfile_from_pin(pw.clone()));
                        match resp {
                            Ok(keyfile) => {
                                cb_sink
                                    .send(Box::new(|siv| {
                                        let data: &mut State = siv.user_data().unwrap();

                                        // Set the state to be logged in, and save the keyfile contents.
                                        data.keyfile = Some(keyfile);
                                        *data.login_state.lock().unwrap() = LoginState::LogInOkay;

                                        // Pop the waiting dialog, and draw the full menu.
                                        siv.pop_layer();
                                        siv.add_layer(full_menu());
                                    }))
                                    .unwrap();
                            }
                            Err(_) => {
                                cb_sink
                                    .send(Box::new(|siv| {
                                        let data: &mut State = siv.user_data().unwrap();

                                        // Set the state to be failed.
                                        *data.login_state.lock().unwrap() = LoginState::LogInFail;

                                        // Pop the waiting dialog, then draw the reduced menu,
                                        // and on top of that draw an error message.
                                        siv.pop_layer();
                                        siv.add_layer(partial_menu());
                                        siv.add_layer(
                                            views::Dialog::around(views::TextView::new(
                                                "Failed to unlock with Yubikey",
                                            ))
                                            .title("Error")
                                            .dismiss_button("OK"),
                                        )
                                    }))
                                    .unwrap();
                            }
                        }
                    });
                });
                edit
            }))
            .with_name("ykpin_input"),
    )
}
