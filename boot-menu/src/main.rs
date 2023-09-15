use std::{
    io::{Cursor, Read, Write},
    process::Stdio,
};

use cursive::{
    align::HAlign,
    theme::{BorderStyle, Palette, Theme},
    view::Margins,
    views, Cursive, With,
};
use efivar::efi::{VariableFlags, VariableName};

enum BootMenuOption {
    Arch,
    Windows,
    Uefi,
    Poweroff,
    Reboot,
}

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

#[derive(Default)]
struct State {
    password: Option<String>,
}

fn main() {
    println!("Boot menu launching!");

    // For ease of use, for the duration of the menu, we enable the CAD combination,
    // which will reboot instantly.
    // This does not compromise security if the BIOS menu is behind a password.
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
    siv.set_user_data(State::default());

    // Do not allow closing the app with ^C.
    siv.clear_global_callbacks(cursive::event::Event::CtrlChar('c'));

    // The top level view is a select between the different boot options.
    siv.add_layer(
        views::Dialog::around({
            let mut select = views::SelectView::new()
                // Center the text horizontally
                .h_align(HAlign::Center)
                // Use keyboard to jump to the pressed letters
                .autojump();
            select.add_item("Boot into Arch Linux", BootMenuOption::Arch);
            select.add_item("Boot into Windows", BootMenuOption::Windows);
            select.add_item("Boot into UEFI Settings", BootMenuOption::Uefi);
            select.add_item("Reboot", BootMenuOption::Reboot);
            select.add_item("Poweroff", BootMenuOption::Poweroff);

            select.set_on_submit(choose_menu);

            select
        })
        .title("Boot menu"),
    );

    // Immediately after this, spawn another layer. This will prompt the user for a password.

    siv.add_layer(views::ThemedView::new(password_input_theme(), {
        let mut dialog = views::Dialog::around({
            let mut edit = views::EditView::new();
            edit.set_secret(true);
            edit.set_on_submit(|siv, text| {
                // TODO: right now we do not do anything with the password we read.
                // Later, this should be validated, e.g. for decrypting the root partition.
                siv.pop_layer();
            });
            edit
        });
        dialog.set_title("Please enter boot password to continue");
        dialog.set_padding(Margins::trbl(2, 5, 2, 5));
        dialog
    }));
    siv.run();
}

// These values are used for Linux syscalls and are taken from https://man7.org/linux/man-pages/man2/reboot.2.html
const LINUX_REBOOT_MAGIC1: usize = 0xfee1dead;
const LINUX_REBOOT_MAGIC2: usize = 0x28121969;
const LINUX_REBOOT_CMD_CAD_ON: usize = 0x89abcdef;
const LINUX_REBOOT_CMD_CAD_OFF: usize = 0;
const LINUX_REBOOT_CMD_POWER_OFF: usize = 0x4321fedc;
const LINUX_REBOOT_CMD_RESTART: usize = 0x1234567;

fn choose_menu(siv: &mut Cursive, choice: &BootMenuOption) {
    match choice {
        BootMenuOption::Arch => {
            // Booting into Arch just means exiting the program and continuing the boot process.
            // This stanza is therefore allowed to use `unwrap`s, since those will exit the program just as well.
            siv.quit();

            // Because we're keeping the current kernel, we should disable the CAD key combination.
            // This will allow using it in user space safely.

            unsafe {
                syscalls::syscall!(
                    syscalls::Sysno::reboot,
                    LINUX_REBOOT_MAGIC1,
                    LINUX_REBOOT_MAGIC2,
                    LINUX_REBOOT_CMD_CAD_OFF,
                    0
                )
            }
            .unwrap();
            // Since we now know that we're booting Arch,
            // this is also when we perform the expensive video card initialization.
            // This also happens in the bash script after the program, so it's not a problem if it fails.

            std::process::Command::new("modprobe")
                .arg("nouveau")
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
            // Before continuing, we should also clear the screen.
            // To do this, we print the output of the `clear` command to the screen.
            let clear_screen_magic = [
                0x1b, 0x5b, 0x48, 0x1b, 0x5b, 0x32, 0x4a, 0x1b, 0x5b, 0x33, 0x4a,
            ];
            std::io::stdout()
                .lock()
                .write_all(&clear_screen_magic)
                .unwrap();
            std::io::stdout().lock().flush().unwrap();

            // At this point, we should be exiting fully.
        }
        BootMenuOption::Windows => {
            // To boot into Windows, we need to first find the boot menu entry corresponding to it.
            let manager = efivar::system();

            let boot_menu_entries = manager.get_boot_entries();
            let boot_menu_entries = match boot_menu_entries {
                Ok(e) => e,
                Err(why) => {
                    siv.add_layer(
                        views::Dialog::around(views::TextView::new(format!(
                            "Listing EFI boot entries failed: {why:?}"
                        )))
                        .dismiss_button("Return to menu"),
                    );
                    return;
                }
            };

            // The boot entries don't have their ID inside them,
            // however they are yielded in the BootOrder's order.
            // https://github.com/iTrooz/efiboot-rs/issues/75
            // So, we can also fetch the BootOrder variable
            // and retrieve the values from there.
            let mut boot_order_value = vec![0u8; 512]; // 256 boot entries should be enough
            let boot_order_read_result =
                manager.read(&VariableName::new("BootOrder"), &mut boot_order_value);

            let boot_order_read_result = match boot_order_read_result {
                Ok(e) => e,
                Err(why) => {
                    siv.add_layer(
                        views::Dialog::around(views::TextView::new(format!(
                            "Listing EFI boot entries failed: {why:?}"
                        )))
                        .dismiss_button("Return to menu"),
                    );
                    return;
                }
            };

            if boot_order_read_result.0 % 2 != 0 {
                siv.add_layer(
                    views::Dialog::around(views::TextView::new(format!(
                        "EFI boot entries list has length {}, which is not allowed.",
                        boot_order_read_result.0
                    )))
                    .dismiss_button("Return to menu"),
                );
                return;
            };

            boot_order_value.truncate(boot_order_read_result.0);
            let mut boot_order_cursor = Cursor::new(boot_order_value);

            let mut target_boot_entry_id = None;

            for (iter_index, entry) in boot_menu_entries.enumerate() {
                let mut boot_entry_id = [0, 0];
                if boot_order_cursor.read_exact(&mut boot_entry_id).is_err() {
                    siv.add_layer(
                        views::Dialog::around(views::TextView::new(format!(
                        "Could not retrieve the ID of boot entry which is {iter_index} in order."
                    )))
                        .dismiss_button("Return to menu"),
                    );
                    return;
                }
                let entry = match entry {
                    Ok(e) => e,
                    Err(why) => {
                        siv.add_layer(
                            views::Dialog::around(views::TextView::new(format!(
                                "Failed to get entry number {iter_index} while iterating: {why:?}"
                            )))
                            .dismiss_button("Return to menu"),
                        );
                        return;
                    }
                };

                // TODO: better way to find which of the entries is the Windows one.
                // For now, use the entry's name
                if entry.description == "Windows Boot Manager" {
                    // Found entry, now need to set it to BootNext.
                    // Because we're iterating these in the BootOrder sequence,
                    // the iter_index is not the value we're looking for.
                    // Instead, it's the value from boot_entry_id.
                    // It's encoded as little-endian, and we need to write it the same way,
                    // so we won't bother decoding it.
                    target_boot_entry_id = Some(boot_entry_id);
                    break;
                }
            }

            let target_boot_entry_id = match target_boot_entry_id {
                Some(s) => s,
                None => {
                    siv.add_layer(
                        views::Dialog::around(views::TextView::new(format!(
                            "Could not find Windows boot option. Boot into another system to fix this."
                        )))
                        .dismiss_button("Return to menu"),
                    );
                    return;
                }
            };
            let attrs = VariableFlags::NON_VOLATILE
                & VariableFlags::BOOTSERVICE_ACCESS
                & VariableFlags::RUNTIME_ACCESS;

            // FIXME: for some reason, doing this with the library produces an Input-Output error.
            // This is strange: normally, when writing wrong values there, only an Invalid Argument error is produced.
            // This also happens when trying to execute the relevant operations manually!
            // Shell out to tools to do this instead.

            // let result =
            //     manager.write(&VariableName::new("BootNext"), attrs, &target_boot_entry_id);
            // match result {
            //     Err(why) => {
            //         siv.add_layer(
            //             views::Dialog::around(views::TextView::new(format!(
            //                 "Failed to write BootNext value into EFI variables: {why:?}"
            //             )))
            //             .dismiss_button("Return to menu"),
            //         );
            //         return;
            //     }
            //     Ok(_) => {
            //         // The BootNext has been set, and now we need to reboot into Windows.
            //         siv.add_layer(views::Dialog::around(views::TextView::new(format!(
            //             "Rebooting into Windows..."
            //         ))));
            //     }
            // };

            let mut child = match std::process::Command::new("efibootmgr")
                .arg("-n")
                .arg(format!(
                    "{:02X}{:02X}",
                    target_boot_entry_id[1], target_boot_entry_id[0]
                ))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(why) => {
                    siv.add_layer(
                        views::Dialog::around(views::TextView::new(format!(
                            "Failed to spawn child process efibootmgr: {why:?}"
                        )))
                        .dismiss_button("Return to menu"),
                    );
                    return;
                }
            };

            match child.wait_with_output() {
                Ok(exit) => {
                    if exit.status.success() {
                        // The BootNext has been set, and now we need to reboot into Windows.
                        siv.add_layer(views::Dialog::around(views::TextView::new(format!(
                            "Rebooting into Windows..."
                        ))));
                        choose_menu(siv, &BootMenuOption::Reboot);
                        return;
                    } else {
                        siv.add_layer(
                            views::Dialog::around(views::TextView::new(format!(
                                "efibootmgr exited with code: {:?}\n{}\n{}",
                                exit.status.code(),
                                String::from_utf8_lossy(&exit.stdout),
                                String::from_utf8_lossy(&exit.stderr),
                            )))
                            .dismiss_button("Return to menu"),
                        );
                        return;
                    }
                }
                Err(why) => {
                    siv.add_layer(
                        views::Dialog::around(views::TextView::new(format!(
                            "Failed to get response from efibootmgr: {why:?}"
                        )))
                        .dismiss_button("Return to menu"),
                    );
                    return;
                }
            }
        }
        BootMenuOption::Uefi => {
            // To reboot into UEFI, we need to set the OsIndications variable to indicate
            // that we want to boot to the firmware UI.
            // This is done by setting the least significant bit.
            // See: https://uefi.org/specs/UEFI/2.10/08_Services_Runtime_Services.html#exchanging-information-between-the-os-and-firmware

            let manager = efivar::system();

            // But first, let's also check that OsIndicationsSupported has that least significant bit set.
            let mut output = 0u64.to_le_bytes();
            let read_result =
                manager.read(&VariableName::new("OsIndicationsSupported"), &mut output);
            if let Err(why) = read_result {
                siv.add_layer(views::Dialog::around(views::TextView::new(format!(
                    "Failed to read EFI OsIndicationsSupported: {why:?}\nEnter UEFI settings manually."
                ))).dismiss_button("Return to menu"));
                return;
            }
            let output = u64::from_le_bytes(output);
            if !(output & 0x01 == 0x01) {
                siv.add_layer(views::Dialog::around(views::TextView::new(format!(
                    "EFI OsIndicationsSupported says UEFI settings are not supported (value is {output})."
                ))).dismiss_button("Return to menu"));
                return;
            }

            // Now that we know we can do this, we need to write the value 1 to the OsIndications.
            // We don't need to worry about any other bits because we are the only OS right now.
            // The attrs are the expected attrs for this variable.

            // FIXME: for some reason, doing this with the library produces an Input-Output error.
            // This is strange: normally, when writing wrong values there, only an Invalid Argument error is produced.
            // This also happens when trying to execute the relevant operations manually!
            // Shell out to tools to do this instead.

            // let attrs = VariableFlags::NON_VOLATILE
            //     & VariableFlags::BOOTSERVICE_ACCESS
            //     & VariableFlags::RUNTIME_ACCESS;
            // let write_result = manager.write(
            //     &VariableName::new("OsIndications"),
            //     attrs,
            //     &1u64.to_le_bytes(),
            // );
            // if let Err(why) = write_result {
            //     siv.add_layer(
            //         views::Dialog::around(views::TextView::new(format!(
            //             "Failed to write OsIndications: {why:?}\nEnter UEFI settings manually."
            //         )))
            //         .dismiss_button("Return to menu"),
            //     );
            //     return;
            // }

            // Use bootctl program to enable reboot to firmware

            match std::process::Command::new("bootctl")
                .arg("reboot-to-firmware")
                .arg("1")
                .spawn()
            {
                Ok(mut child) => {
                    match child.wait() {
                        Err(why) => {
                            siv.add_layer(
                                views::Dialog::around(views::TextView::new(format!(
                                    "Failed to wait for bootctl: {why:?}\nEnter UEFI settings manually."
                                )))
                                .dismiss_button("Return to menu"),
                            );
                            return;
                        }
                        Ok(status) => {
                            match status.success() {
                                true => {
                                    // Successfully set, continuing to reboot
                                }
                                false => {
                                    siv.add_layer(views::Dialog::around(views::TextView::new(format!(
                                        "Failed to use bootctl to set OsIndications (status is {:?})\nEnter UEFI settings manually.",
                                        status.code()
                                    )))
                                    .dismiss_button("Return to menu"));

                                    return;
                                }
                            }
                        }
                    }
                }
                Err(why) => {
                    siv.add_layer(
                        views::Dialog::around(views::TextView::new(format!(
                            "Failed to call bootctl: {why:?}\nEnter UEFI settings manually."
                        )))
                        .dismiss_button("Return to menu"),
                    );
                    return;
                }
            };
            siv.add_layer(views::Dialog::around(views::TextView::new(format!(
                "Rebooting into UEFI..."
            ))));

            // Now that the OsIndications is written, we need to reboot.
            choose_menu(siv, &BootMenuOption::Reboot);
        }
        BootMenuOption::Poweroff => {
            // To poweroff, we need to call the Linux syscall reboot(2),
            // with an argument of LINUX_REBOOT_CMD_POWER_OFF (from the man page).
            // To be safe, we precede this with a call to sync(2).

            let sync_result = unsafe { syscalls::syscall!(syscalls::Sysno::sync) };
            if let Err(why) = sync_result {
                let why = why.name_and_description();
                siv.add_layer(views::Dialog::around(views::TextView::new(format!(
                    "Failed to call sync() syscall: {why:?}\nPower off the system manually."
                ))));
                return;
            }

            // Note: the `arg` parameter is explicitly set as zero. I think that's acceptable, but I don't know for sure.
            let reboot_result = unsafe {
                syscalls::syscall!(
                    syscalls::Sysno::reboot,
                    LINUX_REBOOT_MAGIC1,
                    LINUX_REBOOT_MAGIC2,
                    LINUX_REBOOT_CMD_POWER_OFF,
                    0
                )
            };
            if let Err(why) = reboot_result {
                let why = why.name_and_description();
                siv.add_layer(views::Dialog::around(views::TextView::new(format!(
                    "Failed to call reboot() syscall: {why:?}\nPower off the system manually."
                ))))
            }
        }
        BootMenuOption::Reboot => {
            // To reboot, we do the same procedure as above, except with a different argument to reboot().
            let sync_result = unsafe { syscalls::syscall!(syscalls::Sysno::sync) };
            if let Err(why) = sync_result {
                let why = why.name_and_description();
                siv.add_layer(views::Dialog::around(views::TextView::new(format!(
                    "Failed to call sync() syscall: {why:?}\nReboot the system manually."
                ))));
                return;
            }

            // Note: the `arg` parameter is explicitly set as zero. I think that's acceptable, but I don't know for sure.
            let reboot_result = unsafe {
                syscalls::syscall!(
                    syscalls::Sysno::reboot,
                    LINUX_REBOOT_MAGIC1,
                    LINUX_REBOOT_MAGIC2,
                    LINUX_REBOOT_CMD_RESTART,
                    0
                )
            };
            if let Err(why) = reboot_result {
                let why = why.name_and_description();
                siv.add_layer(views::Dialog::around(views::TextView::new(format!(
                    "Failed to call reboot() syscall: {why:?}\nReboot the system manually."
                ))))
            }
        }
    }
}
