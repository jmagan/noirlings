use console::style;

use crate::exercise::{Exercise, Mode, TomlFile};
// use crate::ui::progress;

// Build the given Exercise and return an object with information
// about the state of the compilation
pub fn build_exercise(exercise: &Exercise) -> Result<String, ()> {
    progress!("Building {} exercise...", exercise);

    let compilation_result = exercise.build();

    if let Err(error) = compilation_result {
        eprintln!("{error}");

        warn!("Compiling of {} failed! Please try again.", exercise);
        Err(())
    } else {
        Ok(compilation_result.unwrap())
    }
}

// Build the given Exercise and return an object with information
// about the state of the compilation
pub fn execute_exercise(exercise: &Exercise, prover_toml: TomlFile) -> Result<String, ()> {
    progress!("Running {} exercise...", exercise);

    let compilation_result = exercise.execute(prover_toml);

    if let Err(error) = compilation_result {
        eprintln!("{error}");

        warn!("Failed to run {}! Please try again.", exercise);
        Err(())
    } else {
        Ok(compilation_result.unwrap())
    }
}


pub fn bb_prove_exercise(exercise: &Exercise, prover_toml: TomlFile) -> Result<String, ()> {
    progress!("Running {} exercise...", exercise);

    let compilation_result = exercise.execute(prover_toml);
    let proof_creation_result = exercise.create_proof();

    if let Err(error) = compilation_result {
        eprintln!("{error}");

        warn!("Failed to execute {}! Please try again.", exercise);
        Err(())
    } else if let Err(error) = proof_creation_result {
        eprintln!("{error}");

        warn!("Compilation worked but failed to create proof with barretenberg for {}! Please try again.", exercise);
        eprintln!("Are you sure you installed barretenberg properly ?");
        Err(())
        
    } else {
        Ok(compilation_result.unwrap())
    }
}

pub fn bb_prove_verify_exercise(exercise: &Exercise, prover_toml: TomlFile) -> Result<String, ()> {
    progress!("Running {} exercise...", exercise);

    let compilation_result = exercise.execute(prover_toml);
    let verification_result = exercise.prove_verify_proof();

    if let Err(error) = compilation_result {
        eprintln!("{error}");

        warn!("Failed to execute {}! Please try again.", exercise);
        Err(())
    } else if let Err(error) = verification_result {
        eprintln!("{error}");

        warn!("Compilation worked but failed to prove and verify with barretenberg backend for {}! Please try again.", exercise);
        eprintln!("Are you sure you installed barretenberg properly ?");
        Err(())
        
    } else {
        Ok(compilation_result.unwrap())
    }
}

// Tests the given Exercise and return an object with information
// about the state of the tests
pub fn test_exercise(exercise: &Exercise) -> Result<String, ()> {
    progress!("Testing {} exercise...", exercise);

    let compilation_result = exercise.test();

    if let Some(error) = compilation_result.as_ref().err() {
        warn!(
            "Testing of {} failed! Please try again. See the output above ^",
            exercise
        );
        println!("{error}");
        Err(())
    } else {
        Ok(compilation_result.unwrap())
    }
}

pub fn print_exercise_output(exercise_output: String) {
    if exercise_output.len() > 0 {
        println!("    {} {exercise_output}", style("Output").green().bold());
    }
}

pub fn print_exercise_success(exercise: &Exercise) {
    match exercise.mode {
        Mode::Build => success!("Successfully built {}!", exercise),
        Mode::Execute(ref toml) => success!("Successfully ran {}!\n With inputs: {}", exercise, toml.to_string()),
        Mode::Test => success!("Successfully tested {}!", exercise),
        Mode::BbProve(ref toml) => success!("Successfully ran {} and created proof!\n With inputs: {}", exercise, toml.to_string()),
        Mode::BbVerify(ref toml) => success!("Successfully ran {} and verified proof!\n With inputs: {}", exercise, toml.to_string()),
    }
}
