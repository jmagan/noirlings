use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::{
    constants::PROVER_INPUT_FILE, insert_all_files_for_workspace_into_file_manager, ops::TestStatus, parse_all
};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_frontend::hir::FunctionNameMatch;
use std::{
    env::current_dir,
    fs::{self},
    path::PathBuf, process::Command,
};
use noirc_driver::{CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING};

use crate::{exercise::TomlFile, nargo::{
        cli_compile_workspace_full, compile, execute_program_and_decode, read_program_from_file, run_tests, save_witness_to_dir
    }};


// Prepares testing crate
// Copies the exercise file into testing crate
pub fn prepare_crate_for_exercise(file_path: &PathBuf, prover_toml: Option<TomlFile>) -> PathBuf {
    let crate_path = current_dir()
        .unwrap()
        .join(PathBuf::from("runner_crate"));
    let src_dir = crate_path.join("src");
    if !src_dir.exists() {
        let _ = fs::create_dir(&src_dir);
    }
    let lib_path = src_dir.join("main.nr");
    let file_path = current_dir().unwrap().join(file_path);

    match fs::copy(&file_path, &lib_path) {
        Ok(_) => {}
        Err(err) => panic!("Error occurred while preparing the exercise,\nExercise: {file_path:?}\nLib path: {lib_path:?}\n{err:?}"),
    };

    if let Some(prover_toml) = prover_toml {
        match prover_toml {
            TomlFile::Inlined(str) => {
                let prover_toml_path = crate_path.join(format!("{}.toml", PROVER_INPUT_FILE));
                fs::write(prover_toml_path, str).expect("Unable to write file");
            },
            TomlFile::Path(path) => {
                let prover_toml_path = crate_path.join(format!("{}.toml", PROVER_INPUT_FILE));
                fs::copy(path, prover_toml_path).expect("Unable to copy file");
            }

        }
    }
    crate_path
}

// Builds the testing crate with scarb
pub fn nargo_compile(file_path: &PathBuf) -> anyhow::Result<String> {
    let _: PathBuf = prepare_crate_for_exercise(file_path, None);
    match compile() {
        Ok(_) => Ok("".into()),
        Err(err) => anyhow::bail!("Failed to compile the program: {:?}", err),
    }
}

// Execute the crate with noir
pub fn nargo_execute(
    file_path: &PathBuf,
    prover_toml: TomlFile,
    exercise_name: String,
) -> anyhow::Result<String> {
    /*      Small version example
    let path = prepare_crate_for_exercise(file_path, Some(prover_toml));
    let witness_stack = run().unwrap();

    let witness_name = &exercise_name;
    let witness_path = save_witness_to_dir(witness_stack, witness_name, target_dir)?;
    println!(
        "[{}] Witness saved to {}",
        package.name,
        witness_path.display()
    );

    Ok("".into())
    */

    let crate_path = prepare_crate_for_exercise(file_path, Some(prover_toml));
    let toml_path = get_package_manifest(&crate_path)?;
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        PackageSelection::DefaultOrAll,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;
    let target_dir = &workspace.target_directory_path();

    // Compile the full workspace in order to generate any build artifacts.
    let default_options = CompileOptions::default();
    cli_compile_workspace_full(&workspace, &default_options)?;

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);
        let program: CompiledProgram =
            read_program_from_file(program_artifact_path.clone())?.into();

        let (return_value, witness_stack) = execute_program_and_decode(
            program,
            package,
            PROVER_INPUT_FILE,
            None,
            Some(workspace.root_dir.clone()),
            Some(package.name.to_string()),
        )?;

        println!("[{}] Circuit witness successfully solved", package.name);
        if let Some(return_value) = return_value {
            println!("[{}] Circuit output: {return_value:?}", package.name);
        }

        let witness_name = &exercise_name;
        let witness_path = save_witness_to_dir(witness_stack, witness_name, target_dir)?;
        println!(
            "[{}] Witness saved to {}",
            package.name,
            witness_path.display()
        );
    }
    anyhow::Ok("".into())
}

pub fn bb_prove(exercise_name: String) -> anyhow::Result<String> {
    // -b ./target/hello_world.json -w ./target/witness-name.gz -o ./target/proof-name
    println!("Creating proof with barretenberg");
    let output = Command::new("bb")
        .arg("prove")
        .arg("-b")
        .arg("runner_crate/target/runner_crate.json")
        .arg("-w")
        .arg(format!("runner_crate/target/{}.gz", exercise_name))
        .arg("-o")
        .arg(format!("runner_crate/target/proof-{}", exercise_name))
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "Failed to prove the program: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    } else{
        anyhow::Ok("".into())
    }
}

pub fn bb_verify(exercise_name: String) -> anyhow::Result<String> {
    // bb write_vk -b ./target/hello_world.json -o ./target/vk
    // bb verify -k ./target/vk -p ./target/proof
    println!("Exporting verification key with barretenberg (bb)");
    let output_write_vk = Command::new("bb")
        .arg("write_vk")
        .arg("-b")
        .arg("runner_crate/target/runner_crate.json")
        .arg("-o")
        .arg(format!("runner_crate/target/vk-{}", exercise_name))
        .output()?;
    println!("Verifying proof with barretenberg (bb)");
    let output_verify = Command::new("bb")
        .arg("verify")
        .arg("-k")
        .arg(format!("runner_crate/target/vk-{}", exercise_name))
        .arg("-p")
        .arg(format!("runner_crate/target/proof-{}", exercise_name))
        .output()?;
    if !output_write_vk.status.success() {
        anyhow::bail!(
            "Failed to verify the program: {}",
            String::from_utf8_lossy(&output_write_vk.stderr)
        );
    } else if !output_verify.status.success() {
        anyhow::bail!(
            "Failed to verify the program: {}",
            String::from_utf8_lossy(&output_verify.stderr)
        );
        
    }else{
        anyhow::Ok("".into())
    }
}

// Runs tests on the testing crate with nargo
pub fn nargo_test(file_path: &PathBuf) -> anyhow::Result<String> {
    let crate_path = prepare_crate_for_exercise(file_path, None);
    let toml_path = get_package_manifest(&crate_path)?;
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        PackageSelection::DefaultOrAll,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    let pattern = FunctionNameMatch::Anything;

    let test_reports: Vec<Vec<(String, TestStatus)>> = workspace
        .into_iter()
        .filter(|package| package.name.to_string() == "runner_crate")
        .map(|package| {
            run_tests::<Bn254BlackBoxSolver>(
                &workspace_file_manager,
                &parsed_files,
                package,
                pattern,
                false,
                None,
                Some(workspace.root_dir.clone()),
                Some(package.name.to_string()),
                &CompileOptions::default(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;

    let test_report: Vec<(String, TestStatus)> = test_reports.into_iter().flatten().collect();

    if test_report.iter().any(|(_, status)| status.failed()) {
        anyhow::bail!("Some tests failed");
    } else {
        Ok("".into())
    }
}
