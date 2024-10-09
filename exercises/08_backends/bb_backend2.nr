// PLEASE READ THE README to Install bb
// Fill the bb_backend1.toml with proper witnesses to continue ; ) 

// I AM NOT DONE!

// Here we have the same circuit as before in a new file (but the same toml file : bb_backend1.toml)
// but now we are using the bb command to also verify the proof

// bb  prove_and_verify -b ./target/runner_crate.json -w ./target/bb_backend2.gz
// more info https://github.com/AztecProtocol/aztec-packages/blob/barretenberg-v0.55.0/barretenberg/cpp/src/barretenberg/bb/main.cpp#L1369-L1512

// We could also do the following:
// bb prove -b ./target/runner_crate.json -w ./target/bb_backend2.gz -o ./target/proof-bb_backend2
// bb write_vk -b ./target/runner_crate.json -o ./target/vk-bb_backend2
// bb verify -k ./target/vk-bb_backend2 -p ./target/proof-bb_backend2

fn main(a : u8, b: u8) {
    let z = a + b;
    assert (z == 2);
}