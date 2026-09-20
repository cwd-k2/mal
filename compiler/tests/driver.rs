use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

mod support;

use support::NativeFixture;

#[path = "driver/artifacts.rs"]
mod artifacts;
#[path = "driver/cli.rs"]
mod cli;
#[path = "driver/examples.rs"]
mod examples;
#[path = "driver/examples_database.rs"]
mod examples_database;
#[path = "driver/examples_tools.rs"]
mod examples_tools;
#[path = "driver/failures.rs"]
mod failures;
