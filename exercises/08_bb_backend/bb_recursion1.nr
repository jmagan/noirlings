// PLEASE READ THE README to Install bb
// Remove the `// I AM NOT DONE!` comment to continue with the recursion example.

// I AM NOT DONE!

// Now we will do a proof recursion example in two parts. This is the first part.
// Here we will add the #[recursive] attribute to the main function to indicate that the circuit is recursion-friendly.
// We will generate the proof and verification key

// The subtility here is that we also have to convert the verification key to a Field array.
// to be able to pass it to the next circuit.

// This is done with vk_as_fields and proof_as_fields:
// cd runnner_crate (in another terminal if you are in watch mode)
// bb vk_as_fields -k ./target/vk-bb_recursion1
// bb proof_as_fields -p ./target/proof-bb_recursion1 -k ./target/vk-bb_recursion1

// RUN the two commands above inside the ./runner_crate for the next step !
// It will give you two files vk-bb_recursion1.json and proof-bb_recursion1.json in the target folder.

#[recursive]
fn main(x: Field, y: pub Field) {
    assert(x == y, "x and y are not equal");
}
// #[recursive] is a special attribute that can be applied to ensure this proof can be used in another proof.
// This marks the circuit as recursion-friendly and indicates that proofs generated from this circuit
// are intended for recursive verification.

