// PLEASE READ THE README to Install bb
// Fill the bb_backend1.toml with proper witnesses to continue ; ) 

// I AM NOT DONE!

use dep::std;


// Good ! Second step ! (No worries, this could be automatized easily in the future)
// We now have two files ! (if you don't have them, go back to the previous exercise by putting // I am not done! in the previous exercise)
// - vk-bb_recursion1.json
// - proof-bb_recursion1.json


// Now we will nicely copy all the elements in the fields format to the bb_recursion2.toml file
// - the Verification Key
// - the proof to verify
// - the public inputs of the proof
// - the hash of the verification key


// Note: the vk.json file we previously exported (from binary) contains 115 elements... uhmm
// it seems the first one is the hash of the verification key ! (we should remove it from the verification_key and put it in the proper argument !)

// Same we have 94 elements in the proof,
// but this one looks quite similar to the public inputs...
// We should copy it in the public inputs ; )  (and remove it from the proof)

// FILL the bb_recursion2.toml with the instruction above !

// Once you finish it should be able to prove and verify ! yeah !
// PS: it may take several seconds or even minutes on old laptops

fn main(
    verification_key : [Field; 114], 
    proof : [Field; 93], 
    public_inputs : [Field; 1], 
    key_hash : Field,
) {
    // the verify_proof function is a black-box function which is implemented by the backend   
    std::verify_proof(
        verification_key.as_slice(), 
        proof.as_slice(), 
        public_inputs.as_slice(), 
        key_hash,
    );
}