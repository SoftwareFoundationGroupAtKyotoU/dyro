fn borrow_vec(v: &Vec<i32>) {
    std::hint::black_box(v);
}

fn borrow_mut_vec(v: &mut Vec<i32>) {
    v.push(4);
}

fn move_vec(v: Vec<i32>) {
    drop(v);
}

fn main() {
    let mut v = vec![1, 2, 3];

    borrow_vec(&v);
    borrow_mut_vec(&mut v);
    move_vec(v);
}
