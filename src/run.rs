use std::process::Command;

use crate::{
    exercise::{Exercise, Mode},
    utils,
};

// Invoke the rust compiler on the path of the given exercise,
// and run the ensuing binary.
// The verbose argument helps determine whether or not to show
// the output from the test harnesses (if the mode of the exercise is test)
pub fn run(exercise: &Exercise) -> Result<(), ()> {
    let run_result = match &exercise.mode {
        Mode::Build => utils::build_exercise(exercise)?,
        Mode::Execute(str) => utils::execute_exercise(exercise, str.clone())?,
        Mode::BbProve(str) => utils::bb_prove_exercise(exercise, str.clone())?,
        Mode::BbVerify(str) => utils::bb_prove_verify_exercise(exercise, str.clone())?,
        Mode::Test => utils::test_exercise(exercise)?,
        _ => {
            eprintln!("Invalid mode for exercise: {}", exercise.name);
            return Err(());
        }
    };
    utils::print_exercise_output(run_result);
    utils::print_exercise_success(exercise);
    Ok(())
}

// Resets the exercise by stashing the changes.
pub fn reset(exercise: &Exercise) -> Result<(), ()> {
    let command = Command::new("git")
        .args(["stash", "--"])
        .arg(&exercise.path)
        .spawn();

    match command {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}
