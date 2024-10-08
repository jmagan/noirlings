// arrays2.nr
// Your task is to make the test pass without modifying the `create_array` function.
// Make me compile and pass the test!

// I AM NOT DONE

// Don't modify this function
fn create_array() -> [Field; 1] {
    [42]
}

fn change_first_element(a: &mut [Field; 1]) {
    // FIX THIS
}

#[test]
fn test_arrays2() {
    let mut a = create_array();
    assert(a[0] == 42, "First element is not 42");

    change_first_element(&mut a);

    assert(a[0] != 42, "First element is still 42");
}