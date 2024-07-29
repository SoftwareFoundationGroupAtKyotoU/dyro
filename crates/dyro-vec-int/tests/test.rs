use dyro_vec_int::VecInt;

#[test]
fn test_push_and_get() {
    let vec = VecInt::new();
    let vec = vec.push(1);
    let vec = vec.push(2);
    let vec = vec.push(3);

    let (vec, result) = vec.get(0);
    assert_eq!(result, 1);

    let (vec, result) = vec.get(1);
    assert_eq!(result, 2);

    let (_vec, result) = vec.get(2);
    assert_eq!(result, 3);
}

#[test]
#[should_panic]
fn test_new_get() {
    let vec = VecInt::new();
    vec.get(0);
}

#[test]
#[should_panic]
fn test_out_of_bounds_get() {
    let vec = VecInt::new();
    let vec = vec.push(1);
    vec.get(1);
}
