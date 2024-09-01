// WARNING: This output format is intended for human consumers only
// and is subject to change without notice. Knock yourself out.
fn borrow_vec(_1: &Vec<i32>) -> () {
    debug v => _1;
    let mut _0: ();
    let _2: &std::vec::Vec<i32>;
    scope 1 (inlined std::hint::black_box::<&Vec<i32>>) {
    }

    bb0: {
        _2 = std::intrinsics::black_box::<&Vec<i32>>(move _1) -> [return: bb1, unwind unreachable];
    }

    bb1: {
        return;
    }
}

fn borrow_mut_vec(_1: &mut Vec<i32>) -> () {
    debug v => _1;
    let mut _0: ();
    let _2: ();

    bb0: {
        _2 = Vec::<i32>::push(move _1, const 4_i32) -> [return: bb1, unwind continue];
    }

    bb1: {
        return;
    }
}

fn move_vec(_1: Vec<i32>) -> () {
    debug v => _1;
    let mut _0: ();
    let mut _2: std::vec::Vec<i32>;
    scope 1 (inlined std::mem::drop::<Vec<i32>>) {
    }

    bb0: {
        _2 = move _1;
        drop(_2) -> [return: bb1, unwind continue];
    }

    bb1: {
        return;
    }
}

fn main() -> () {
    let mut _0: ();
    let mut _1: std::vec::Vec<i32>;
    let mut _2: std::boxed::Box<[i32]>;
    let mut _3: *mut u8;
    let mut _4: std::boxed::Box<[i32; 3]>;
    let _5: ();
    let _6: &std::vec::Vec<i32>;
    let _7: ();
    let mut _8: &mut std::vec::Vec<i32>;
    let _9: ();
    let mut _10: std::vec::Vec<i32>;
    let mut _11: bool;
    let mut _12: *const [i32; 3];
    let mut _13: usize;
    let mut _14: usize;
    let mut _15: bool;
    scope 1 {
        debug v => _1;
    }
    scope 2 (inlined slice::<impl [i32]>::into_vec::<std::alloc::Global>) {
    }

    bb0: {
        _3 = alloc::alloc::exchange_malloc(const 12_usize, const 4_usize) -> [return: bb1, unwind continue];
    }

    bb1: {
        _4 = ShallowInitBox(move _3, [i32; 3]);
        _12 = copy (((_4.0: std::ptr::Unique<[i32; 3]>).0: std::ptr::NonNull<[i32; 3]>).0: *const [i32; 3]);
        _13 = copy _12 as usize (Transmute);
        _14 = BitAnd(copy _13, const 3_usize);
        _15 = Eq(copy _14, const 0_usize);
        assert(copy _15, "misaligned pointer dereference: address must be a multiple of {} but is {}", const 4_usize, copy _13) -> [success: bb8, unwind unreachable];
    }

    bb2: {
        _8 = &mut _1;
        _7 = borrow_mut_vec(move _8) -> [return: bb3, unwind: bb7];
    }

    bb3: {
        _11 = const false;
        _10 = move _1;
        _9 = move_vec(move _10) -> [return: bb4, unwind: bb7];
    }

    bb4: {
        return;
    }

    bb5 (cleanup): {
        resume;
    }

    bb6 (cleanup): {
        drop(_1) -> [return: bb5, unwind terminate(cleanup)];
    }

    bb7 (cleanup): {
        switchInt(copy _11) -> [0: bb5, otherwise: bb6];
    }

    bb8: {
        (*_12) = [const 1_i32, const 2_i32, const 3_i32];
        _2 = copy _4 as std::boxed::Box<[i32]> (PointerCoercion(Unsize));
        _1 = slice::hack::into_vec::<i32, std::alloc::Global>(move _2) -> [return: bb9, unwind continue];
    }

    bb9: {
        _11 = const true;
        _6 = &_1;
        _5 = borrow_vec(move _6) -> [return: bb2, unwind: bb7];
    }
}
