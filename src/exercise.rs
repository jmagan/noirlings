use regex::Regex;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use std::fmt::{self, Display, Formatter};
use std::fs::{remove_file, File};
use std::io::Read;
use std::path::PathBuf;
use std::process::{self};

use crate::noir::{bb_prove, bb_prove_and_verify, bb_prove_verify_saving_files, nargo_compile, nargo_execute, nargo_test};

const I_AM_DONE_REGEX: &str = r"(?m)^\s*///?\s*I\s+AM\s+NOT\s+DONE";
const CONTEXT: usize = 2;

// Get a temporary file name that is hopefully unique
#[inline]
fn temp_file() -> String {
    let thread_id: String = format!("{:?}", std::thread::current().id())
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();

    format!("./temp_{}_{thread_id}", process::id())
}

// The mode of the exercise.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    // Indicates that the exercise should be compiled as ACIR
    Build,
    /** Allow execution with export of witnesses.
    Need to specify the path of the toml file OR the toml content with its input values inlined like
    """
    { execute = {inlined = "a = '1' \nb = '2'"}}
    """
    OR
        """
    { execute = {path = "path/to/toml.toml"}}
    """
    */
    Execute(TomlFile),
    BbProve(TomlFile),
    /**
     *     """
    { bbverify = { toml_file = { path = "path/to/toml.toml"}, save_files = true }
    """
     */
    BbVerify(BbVerifyOptions),
    // Indicates that the exercise should be compile and tested from the written Rust-like test
    Test,
}

#[derive(Deserialize,Clone,Debug)]
pub struct BbVerifyOptions{
    pub toml_file: TomlFile,
    pub save_files: bool
}

#[derive(Clone, Debug)]
pub enum TomlFile {
    Inlined(String),
    Path(String)
}

impl TomlFile {
    pub fn to_string(&self) -> String {
        match self {
            TomlFile::Inlined(s) => s.clone(),
            TomlFile::Path(p) => {
                let mut file = File::open(p).unwrap_or_else(|_| {
                    panic!("We were unable to open the toml file! {:?}", p)
                });

                let mut s = String::new();
                file
                    .read_to_string(&mut s)
                    .expect("We were unable to read the toml file!");
                s
            }
        }
    }
}

impl<'de> Deserialize<'de> for TomlFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TomlFileVisitor;

        impl<'de> Visitor<'de> for TomlFileVisitor {
            type Value = TomlFile;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with either 'inlined' or 'path' key")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let key: String = map
                    .next_key()?
                    .ok_or_else(|| de::Error::custom("missing key"))?;
                match key.as_str() {
                    "inlined" => {
                        let value: String = map.next_value()?;
                        Ok(TomlFile::Inlined(value))
                    }
                    "path" => {
                        let value: String = map.next_value()?;
                        Ok(TomlFile::Path(value))
                    }
                    _ => Err(de::Error::unknown_field(&key, &["inlined", "path"])),
                }
            }
        }

        deserializer.deserialize_map(TomlFileVisitor)
    }
}



fn deserialize_mode<'de, D>(deserializer: D) -> Result<Mode, D::Error>
where
    D: Deserializer<'de>,
{
    struct ModeVisitor;

    impl<'de> Visitor<'de> for ModeVisitor {
        type Value = Mode;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a map representing a mode")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                "test" => Ok(Mode::Test),
                "build" => Ok(Mode::Build),
                _ => Err(de::Error::unknown_variant(value, &["test", "build"])),
            }
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let key: String = map
                .next_key()?
                .ok_or_else(|| de::Error::custom("missing key"))?;
            match key.as_str() {
                "execute" => {
                    let value: TomlFile = map.next_value()?;
                    Ok(Mode::Execute(value))
                },
                "bbprove" => {
                    let value : TomlFile = map.next_value()?;
                    Ok(Mode::BbProve(value))
                },
                "bbverify" => {
                    let value : BbVerifyOptions = map.next_value()?;
                    Ok(Mode::BbVerify(value))
                },
                _ => Err(de::Error::unknown_field(&key, &["execute","bbprove","bbverify"])),
            }
        }
    }

    deserializer.deserialize_any(ModeVisitor)
}

#[derive(Deserialize)]
pub struct ExerciseList {
    pub exercises: Vec<Exercise>,
}

// A representation of a starklings exercise.
// This is deserialized from the accompanying info.toml file
#[derive(Deserialize, Debug)]
pub struct Exercise {
    // Name of the exercise
    pub name: String,
    // The path to the file containing the exercise's source code
    pub path: PathBuf,
    // The mode of the exercise (Test/Build)
    #[serde(deserialize_with = "deserialize_mode")]
    pub mode: Mode,
    // The hint text associated with the exercise
    pub hint: String,
}

// An enum to track of the state of an Exercise.
// An Exercise can be either Done or Pending
#[derive(PartialEq, Debug)]
pub enum State {
    // The state of the exercise once it's been completed
    Done,
    // The state of the exercise while it's not completed yet
    Pending(Vec<ContextLine>),
}

// The context information of a pending exercise
#[derive(PartialEq, Debug)]
pub struct ContextLine {
    // The source code that is still pending completion
    pub line: String,
    // The line number of the source code still pending completion
    pub number: usize,
    // Whether or not this is important
    pub important: bool,
}

// A representation of an already executed binary
#[derive(Debug)]
pub struct ExerciseOutput {
    // The textual contents of the standard output of the binary
    pub stdout: String,
    // The textual contents of the standard error of the binary
    pub stderr: String,
}

struct FileHandle;

impl Drop for FileHandle {
    fn drop(&mut self) {
        clean();
    }
}

impl Exercise {
    pub fn build(&self) -> anyhow::Result<String> {
        nargo_compile(&self.path)
    }

    pub fn execute(&self, prover_toml: TomlFile) -> anyhow::Result<String> {
        nargo_execute(&self.path, prover_toml, self.name.clone())
    }

    pub fn create_proof(&self) -> anyhow::Result<String> {
        bb_prove(self.name.clone())
    }

    pub fn prove_verify_proof(&self, saving_files: bool) -> anyhow::Result<String> {
        if (saving_files) {
            return bb_prove_verify_saving_files(self.name.clone());
        } else {
            return bb_prove_and_verify(self.name.clone());
        }
    }

    pub fn test(&self) -> anyhow::Result<String> {
        nargo_test(&self.path)
    }

    pub fn state(&self) -> State {
        let mut source_file = File::open(&self.path).unwrap_or_else(|_| {
            panic!("We were unable to open the exercise file! {:?}", self.path)
        });

        let source = {
            let mut s = String::new();
            source_file
                .read_to_string(&mut s)
                .expect("We were unable to read the exercise file!");
            s
        };

        let re = Regex::new(I_AM_DONE_REGEX).unwrap();

        if !re.is_match(&source) {
            return State::Done;
        }

        let matched_line_index = source
            .lines()
            .enumerate()
            .find_map(|(i, line)| if re.is_match(line) { Some(i) } else { None })
            .expect("This should not happen at all");

        let min_line = ((matched_line_index as i32) - (CONTEXT as i32)).max(0) as usize;
        let max_line = matched_line_index + CONTEXT;

        let context = source
            .lines()
            .enumerate()
            .filter(|&(i, _)| i >= min_line && i <= max_line)
            .map(|(i, line)| ContextLine {
                line: line.to_string(),
                number: i + 1,
                important: i == matched_line_index,
            })
            .collect();

        State::Pending(context)
    }

    // Check that the exercise looks to be solved using self.state()
    // This is not the best way to check since
    // the user can just remove the "I AM NOT DONE" string from the file
    // without actually having solved anything.
    // The only other way to truly check this would to compile and run
    // the exercise; which would be both costly and counterintuitive
    pub fn looks_done(&self) -> bool {
        self.state() == State::Done
    }
}

impl Display for Exercise {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.path.to_str().unwrap())
    }
}

#[inline]
fn clean() {
    let _ignored = remove_file(temp_file());
}

#[cfg(test)]
mod test {
    use super::*;
    // use std::path::Path;

    // #[test]
    // fn test_finished_exercise() {
    //     let exercise = Exercise {
    //         name: "finished_exercise".into(),
    //         path: PathBuf::from("tests/fixture/noir/compilePass.nr"),
    //         mode: Mode::Build,
    //         hint: String::new(),
    //     };

    //     assert_eq!(exercise.state(), State::Done);
    // }

    #[test]
    fn test_noir_test_passes() {
        let exercise = Exercise {
            name: "testPass".into(),
            path: PathBuf::from("tests/fixture/noir/testPass.nr"),
            mode: Mode::Test,
            hint: String::new(),
        };

        assert_eq!(exercise.state(), State::Done);
    }
}
