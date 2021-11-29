use serde::Deserialize;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, remove_file};
use std::path::PathBuf;
use std::process::{self, Command};

use argh::FromArgs;

#[macro_use]
mod ui;

const RUSTC_COLOR_ARGS: &[&str] = &["--color", "always"];

#[inline]
fn clean() {
    let _ignored = remove_file(&temp_file());
}

// See Exercise struct
#[derive(Deserialize, Copy, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    // Indicates that the exercise should be compiled as a binary
    Compile,
    // Indicates that the exercise should be compiled as a test harness
    Test,
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

// List of exercises ([Exercise])
#[derive(Deserialize)]
pub struct ExerciseList {
    pub exercises: Vec<Exercise>,
}

// The output of the 'compilation'
#[derive(Debug)]
pub struct ExerciseOutput {
    pub stdout: String,
    pub stderr: String,
}

// The result of compiling an exercise
#[derive(Debug)]
pub struct CompiledExercise<'a> {
    exercise: &'a Exercise,
}

impl<'a> CompiledExercise<'a> {
    // Run the compiled exercise
    pub fn run(&self) -> Result<ExerciseOutput, ExerciseOutput> {
        run(self.exercise)
    }
}

impl Display for Exercise {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.path.to_str().unwrap())
    }
}

// Get a temporary file name that is hopefully unique
#[inline]
fn temp_file() -> String {
    let thread_id: String = format!("{:?}", std::thread::current().id())
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();

    format!("./temp_{}_{}", process::id(), thread_id)
}

// Compile the exercise
fn compile_exercise(exercise: &Exercise) -> Result<CompiledExercise, ExerciseOutput> {
    let cmd = match exercise.mode {
        Mode::Compile => Command::new("rustc")
            .args(&[exercise.path.to_str().unwrap(), "-o", &temp_file()])
            .args(RUSTC_COLOR_ARGS)
            .output(),
        Mode::Test => Command::new("rustc")
            .args(&[
                "--test",
                exercise.path.to_str().unwrap(),
                "-o",
                &temp_file(),
            ])
            .args(RUSTC_COLOR_ARGS)
            .output(),
    }
    .expect("Failed to execute 'compilation'");

    if cmd.status.success() {
        Ok(CompiledExercise { exercise })
    } else {
        clean();
        Err(ExerciseOutput {
            stdout: String::from_utf8_lossy(&cmd.stdout).to_string(),
            stderr: String::from_utf8_lossy(&cmd.stderr).to_string(),
        })
    }
}

// Run the exercise
fn run(exercise: &Exercise) -> Result<ExerciseOutput, ExerciseOutput> {
    let arg = match exercise.mode {
        Mode::Test => "--show-output",
        Mode::Compile => "",
    };

    let cmd = Command::new(&temp_file())
        .arg(arg)
        .output()
        .expect("Failed to execute 'run'");

    let output = ExerciseOutput {
        stdout: String::from_utf8_lossy(&cmd.stdout).to_string(),
        stderr: String::from_utf8_lossy(&cmd.stderr).to_string(),
    };

    if cmd.status.success() {
        Ok(output)
    } else {
        Err(output)
    }
}

// Returns the Exercise chosen by the user
fn find_exercise<'a>(name: &str, exercises: &'a [Exercise]) -> &'a Exercise {
    exercises
        .iter()
        .find(|e| e.name == name)
        .unwrap_or_else(|| {
            println!("Exercise {} not found!", name);
            std::process::exit(1)
        })
}

// Returns a list of exercises inside the folder 'name'
fn find_folder<'a>(name: &str, exercises: &'a [Exercise]) -> Vec<&'a Exercise> {
    let mut exer: Vec<&'a Exercise> = Vec::new();
    for e in exercises.iter() {
        let path_split: Vec<&str> = e.path.to_str().unwrap().split("/").collect();
        let folder_name = path_split[1];
        if folder_name == name {
            exer.push(e);
        }
    }
    if exer.is_empty() {
        println!("Folder {} not found!", name);
        std::process::exit(1);
    } else {
        exer
    }
}

#[derive(FromArgs, PartialEq, Debug)]
/// This is a collection of small exercises
struct Args {
    #[argh(switch)]
    /// run all exercises
    all: bool,
    #[argh(subcommand)]
    nested: Option<Subcommands>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Subcommands {
    RunFile(RunFileArgs),
    RunDir(RunDirArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "file")]
/// Runs/Tests a single exercise
struct RunFileArgs {
    #[argh(positional)]
    /// the name of the exercise
    file_name: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "dir")]
/// Runs/Tests exercises inside a folder
struct RunDirArgs {
    #[argh(positional)]
    /// the name of the folder
    dir_name: String,
}

fn main() {
    let args: Args = argh::from_env();

    let toml_str = &fs::read_to_string("info.toml").unwrap();
    let exercises = toml::from_str::<ExerciseList>(toml_str).unwrap().exercises;

    if args.all {
        exercises.iter().for_each(|exercise| {
            let compilation_result = compile_exercise(exercise);
            match compilation_result {
                Ok(compiled_exercise) => {
                    let run_exercise = run(compiled_exercise.exercise);
                    match run_exercise {
                        Ok(output) => {
                            success!("{} executed successfully! Here's the output:", exercise);
                            println!("{}", output.stdout)
                        }
                        Err(output) => {
                            run_error!("Execution of {} failed! Here's the output:", exercise);
                            println!("{}", output.stderr);
                        }
                    };
                }
                Err(output) => {
                    compilation_error!(
                        "Compiling of {} failed! Please correct it and try again. Here's the output:\n",
                        exercise
                    );
                    println!("{}", output.stderr);
                }
            };
            clean();
        });
        std::process::exit(0);
    }

    let command = args.nested.unwrap_or_else(|| {
        println!("{}", "Something went wrong. Try --help.");
        std::process::exit(0);
    });

    match command {
        Subcommands::RunFile(subargs) => {
            let exercise = find_exercise(&subargs.file_name, &exercises);

            let compilation_result = compile_exercise(exercise);

            match compilation_result {
                Ok(compiled_exercise) => {
                    let run_exercise = run(compiled_exercise.exercise);
                    match run_exercise {
                        Ok(output) => {
                            success!("{} executed successfully! Here's the output:", exercise);
                            println!("{}", output.stdout);
                        }
                        Err(output) => {
                            run_error!("Execution of {} failed! Here's the output:", exercise);
                            println!("{}", output.stderr);
                        }
                    };
                }
                Err(output) => {
                    compilation_error!(
                                "Compiling of {} failed! Please correct it and try again. Here's the output:\n",
                                exercise
                            );
                    println!("{}", output.stderr);
                }
            };
            clean();
        }
        Subcommands::RunDir(subargs) => {
            let dir_exercises = find_folder(&subargs.dir_name, &exercises);
            dir_exercises.iter().for_each(|exercise| {
                let compilation_result = compile_exercise(exercise);
                match compilation_result {
                    Ok(compiled_exercise) => {
                        let run_exercise = run(compiled_exercise.exercise);
                        match run_exercise {
                            Ok(output) => {
                                success!("{} executed successfully! Here's the output:", exercise);
                                println!("{}", output.stdout)
                            }
                            Err(output) => {
                                run_error!("Execution of {} failed! Here's the output:", exercise);
                                println!("{}", output.stderr);
                            }
                        };
                    }
                    Err(output) => {
                        compilation_error!(
                            "Compiling of {} failed! Please correct it and try again. Here's the output:\n",
                            exercise
                        );
                        println!("{}", output.stderr);
                    }
                };
                clean();
            });
        }
    }
}
