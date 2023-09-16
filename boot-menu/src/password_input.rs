use std::{
    process::Stdio,
    sync::{Arc, Mutex},
};

use cursive::{
    view::Nameable,
    views::{self},
    Cursive,
};

use crate::{LoginState, State};

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
                    data.password = Some(text.to_string());
                    let mut stateref = data.login_state.lock().unwrap();
                    *stateref = LoginState::LogInOkay;
                    drop(stateref);

                    siv.pop_layer();
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
                    data.password = Some(text.to_string());
                    let mut stateref = data.login_state.lock().unwrap();
                    *stateref = LoginState::LogInOkay;
                    drop(stateref);

                    siv.pop_layer();
                });
                edit
            }))
            .with_name("ykpin_input"),
    )
}
