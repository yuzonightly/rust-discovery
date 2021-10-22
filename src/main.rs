use serde::Deserialize;
use std::fmt::{self, Display, Formatter};
use std::fs::{self, remove_file};
use std::path::PathBuf;
use std::process::{self, Command};

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

fn main() {
    let name = "variables1"; // arg --all(a) --specify(s)
                             // let path = Path::new("./exercises");
    let toml_str = &fs::read_to_string("info.toml").unwrap();
    let exercises = toml::from_str::<ExerciseList>(toml_str).unwrap().exercises;
    // println!("{:?}", exercises[0]);

    let exercise = find_exercise(name, &exercises);

    let compilation_result = compile_exercise(exercise);

    let compilation = exercises.iter().for_each(|e| {
        let compilation_result = compile_exercise(e);

        let compilation = match compilation_result {
            Ok(compiled_exercise) => {
                println!("Exercise compiled successfully.");
                let run_exercise = run(compiled_exercise.exercise);
                match run_exercise {
                    Ok(ExerciseOutput) => {
                        println!("Exercise run successfully.");
                    },
                    Err(ExerciseOutput) => {
                        println!("Exercise did not run successfully.");
                    }
                };
            },
            Err(output) => {
                // warn!(
                //     "Compiling of {} failed! Please try again. Here's the output:",
                //     exercise
                // );
                // println!("{}", output.stderr);
                println!("Exercise compilation failed.");
                // Err(())
            }
        };
    });
    clean();
}
