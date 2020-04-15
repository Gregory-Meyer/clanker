// Copyright (C) 2020 Gregory Meyer
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

mod color;
mod compress;
mod git;

#[macro_use]
extern crate clap;

use color::Color;
use git::Repository;

use std::{env, ffi::CStr, mem::MaybeUninit, process::Command, thread};

use clap::{AppSettings, Arg, ArgMatches, SubCommand};

fn main() {
    include_str!("../Cargo.toml");

    let min_home_dir_uid_arg = Arg::with_name("min_home_dir_uid")
        .short("m")
        .long("min-home-dir-uid")
        .value_name("UID")
        .help(
            "Minimum UID for home directory prefix compression. A home directory prefix will \
               only be stripped if that user's UID is greater than or equal to this value.",
        )
        .default_value("0")
        .validator(|maybe_min_home_dir_uid| {
            if maybe_min_home_dir_uid.parse::<u64>().is_err() {
                Err("expected an integer".to_string())
            } else {
                Ok(())
            }
        });
    let working_directory_arg = Arg::with_name("working_directory")
        .short("w")
        .long("working-directory")
        .value_name("DIRECTORY")
        .help("Path to use as the current working directory.")
        .env("PWD");

    let matches = app_from_crate!()
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequired)
        .subcommand(
            SubCommand::with_name("prompt")
                .about("Left side command prompt.")
                .arg(
                    Arg::with_name("unpriviliged_cursor")
                        .short("u")
                        .long("unpriviliged-cursor")
                        .value_name("CURSOR")
                        .help("Cursor used when the current user is unpriviliged.")
                        .default_value(">"),
                )
                .arg(
                    Arg::with_name("priviliged_cursor")
                        .short("p")
                        .long("priviliged-cursor")
                        .value_name("CURSOR")
                        .help("Cursor used when the current user is priviliged.")
                        .default_value("#"),
                )
                .arg(
                    Arg::with_name("no_username_hostname")
                        .short("S")
                        .long("no-username-hostname")
                        .help("If set, the current username and hostname will not be output."),
                )
                .arg(min_home_dir_uid_arg.clone())
                .arg(working_directory_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("right-prompt")
                .about("Right side command prompt.")
                .arg(
                    Arg::with_name("return_code")
                        .short("r")
                        .long("return-code")
                        .value_name("CODE")
                        .help("Return code from the last run command.")
                        .default_value("0")
                        .validator(|maybe_return_code| {
                            if maybe_return_code.parse::<i32>().is_err() {
                                Err("expected an integer".to_string())
                            } else {
                                Ok(())
                            }
                        }),
                ),
        )
        .subcommand(
            SubCommand::with_name("title")
                .about("Window title for terminal emulators.")
                .arg(
                    Arg::with_name("current")
                        .short("c")
                        .long("currently-running")
                        .value_name("COMMAND")
                        .help("Name of the currently running command.")
                        .takes_value(true),
                )
                .arg(min_home_dir_uid_arg.clone())
                .arg(working_directory_arg.clone()),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("prompt") {
        let unpriviliged_cursor = matches.value_of("unpriviliged_cursor").unwrap();
        let priviliged_cursor = matches.value_of("priviliged_cursor").unwrap();
        let min_home_dir_uid = matches
            .value_of("min_home_dir_uid")
            .unwrap()
            .parse()
            .unwrap();
        let compressed_working_directory = compressed_working_directory(matches, min_home_dir_uid);

        if matches.is_present("no_username_hostname") {
            if is_root() {
                print!(
                    "{}{} ",
                    compressed_working_directory.red(),
                    priviliged_cursor
                );
            } else {
                print!(
                    "{}{} ",
                    compressed_working_directory.green(),
                    unpriviliged_cursor
                );
            }
        } else {
            let (username, is_root) = username_and_is_root();
            let hostname = hostname();

            if is_root {
                print!(
                    "{}@{} {}{} ",
                    username,
                    hostname,
                    compressed_working_directory.red(),
                    priviliged_cursor
                );
            } else {
                print!(
                    "{}@{} {}{} ",
                    username,
                    hostname,
                    compressed_working_directory.green(),
                    unpriviliged_cursor
                );
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("right-prompt") {
        let return_code: i32 = matches.value_of("return_code").unwrap().parse().unwrap();

        if let Some(head) = repo_head() {
            if return_code != 0 {
                print!("{} ({})", return_code.red(), head);
            } else {
                print!("({})", head);
            }
        } else if return_code != 0 {
            print!("{}", return_code.red());
        }
    } else if let Some(matches) = matches.subcommand_matches("title") {
        let min_home_dir_uid = matches
            .value_of("min_home_dir_uid")
            .unwrap()
            .parse()
            .unwrap();
        let compressed_working_directory = compressed_working_directory(matches, min_home_dir_uid);

        if let Some(current) = matches.value_of("current") {
            print!("{} {}", current, compressed_working_directory)
        } else {
            print!("{}", compressed_working_directory);
        }
    } else {
        unreachable!();
    }
}

fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn username_and_is_root() -> (String, bool) {
    let euid = unsafe { libc::geteuid() };
    let passwd = unsafe { libc::getpwuid(euid) };
    assert!(!passwd.is_null());

    let username = unsafe { CStr::from_ptr((*passwd).pw_name) };

    (username.to_string_lossy().into_owned(), euid == 0)
}

fn hostname() -> String {
    // less work than utsname - no dynamic alloc for hostname buffer
    // probably
    let mut utsname = MaybeUninit::uninit();
    let ret = unsafe { libc::uname(&mut *utsname.as_mut_ptr()) };
    assert_eq!(ret, 0);

    let utsname = unsafe { utsname.assume_init() };
    let hostname = unsafe { CStr::from_ptr(utsname.nodename.as_ptr()) };

    hostname.to_string_lossy().into_owned()
}

fn compressed_working_directory(matches: &ArgMatches, min_home_dir_uid: u64) -> String {
    if let Some(dir) = matches.value_of_os("working_directory") {
        compress::compress(dir.as_ref(), min_home_dir_uid).ok()
    } else if let Ok(dir) = env::current_dir() {
        compress::compress(&dir, min_home_dir_uid).ok()
    } else {
        None
    }
    .unwrap_or_else(|| "?".to_string())
}

fn repo_head() -> Option<String> {
    let is_dirty_thread = thread::spawn(repository_is_dirty);

    let repo = Repository::open_from_env()?;
    let identifier = identify_head(&repo)?;

    if is_dirty_thread.join().ok().unwrap_or(false) {
        Some(format!("{} *", identifier))
    } else {
        Some(identifier)
    }
}

fn identify_head(repo: &Repository) -> Option<String> {
    let head = repo.head()?;

    if let Some(name) = head.branch_name() {
        Some(name.to_string_lossy().into_owned())
    } else {
        let head_commit = head.peel_to_commit()?; // this had better point to a commit...
        let tags = repo.tags_pointing_to(&head_commit).unwrap_or_else(Vec::new);

        if tags.is_empty() {
            head_commit.as_object().short_id()
        } else {
            let tag_names: Vec<_> = tags.iter().map(|n| n.to_string_lossy()).collect();

            Some(tag_names.join("\\"))
        }
    }
}

fn repository_is_dirty() -> bool {
    // this is much faster than checking for the first diff and then aborting
    // difference on my computer was from 3s (hand rolled libgit2, abort after first diff) to
    // 660 ms when using the future-style computation here
    Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .ok()
        .map(|output| {
            if !output.status.success() {
                return false;
            }

            !output.stdout.is_empty()
        })
        .unwrap_or(false)
}
