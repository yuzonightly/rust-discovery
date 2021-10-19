use argh::FromArgs;
use console::Emoji;
use notify::DebouncedEvent;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, remove_file, File};
use std::io::Read;
use std::io::{self, prelude::*};
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use std::process::{self, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// See Exercise struct
#[derive(Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    // Indicates that the exercise should be compiled as a binary
    Compile,
    // Indicates that the exercise should be compiled as a test harness
    Test,
    // Indicates that the exercise should be linted with clippy
    Clippy,
}

// Deserialize for info.toml (list of all exercises)
#[derive(Deserialize, Debug)]
pub struct Exercise {
    // Name of the exercise
    pub name: String,
    // The path to the file containing the exercise's source code
    pub path: PathBuf,
    // The mode of the exercise (Test, Compile, or Clippy)
    pub mode: Mode,
    // The hint text associated with the exercise
    pub hint: String,
}

// List of exercises (Exercise struct)
#[derive(Deserialize)]
pub struct ExerciseList {
    pub exercises: Vec<Exercise>,
}

// fn compile_exercise() {}

// fn test_exercise() {}

// fn clippy_exercise() {}

// Returns the Exercise chosen by the user
#[allow(dead_code)]
fn find_exercise<'a>(name: &str, exercises: &'a [Exercise]) -> &'a Exercise {
    exercises
        .iter()
        .find(|e| e.name == name)
        .unwrap_or_else(|| {
            println!("Exercise {} not found!", name);
            std::process::exit(1)
        })
}

fn main() {
    let name = "variables"; // arg
    // let path = Path::new("./exercises");
    let toml_str = &fs::read_to_string("info.toml").unwrap();
    let exercises = toml::from_str::<ExerciseList>(toml_str).unwrap().exercises;
    // println!("{:?}", exercises[0]);

    

    // find_exercise(&name, exercises: &'a [Exercise])
}
