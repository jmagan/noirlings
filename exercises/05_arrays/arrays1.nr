// arrays1.nr
// Your task is to create an array which holds three elements of type `Field`.
// The first element should be 0.
// Make me compile and pass the test!

// I AM NOT DONE

fn create_array() -> [Field; 3] {
    // FILL ME
}


// Don't change anything in the test
#[test]
fn test_array_len() {
    let a = create_array();
    assert(a.len() == 3, "Array length is not 3");
    assert(a[0] == 0, "First element is not 0");
}

