// WARNING: This output format is intended for human consumers only
// and is subject to change without notice. Knock yourself out.
fn borrow_vec(_1: &Vec<i32>) -> () {
    debug v => _1;
    let mut _0: ();
    let _2: &std::vec::Vec<i32>;

    bb0: {
        _2 = std::hint::black_box::<&Vec<i32>>(copy _1) -> [return: bb1, unwind continue];
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
        _2 = Vec::<i32>::push(copy _1, const 4_i32) -> [return: bb1, unwind continue];
    }

    bb1: {
        return;
    }
}

fn move_vec(_1: Vec<i32>) -> () {
    debug v => _1;
    let mut _0: ();
    let _2: ();

    bb0: {
        _2 = std::mem::drop::<Vec<i32>>(move _1) -> [return: bb1, unwind continue];
    }

    bb1: {
        return;
    }
}

fn main() -> () {
    let mut _0: ();
    let mut _1: std::vec::Vec<i32>;
    let mut _2: std::boxed::Box<[i32]>;
    let mut _3: usize;
    let mut _4: usize;
    let mut _5: *mut u8;
    let mut _6: std::boxed::Box<[i32; 3]>;
    let _7: ();
    let _8: &std::vec::Vec<i32>;
    let _9: ();
    let mut _10: &mut std::vec::Vec<i32>;
    let _11: ();
    let mut _12: std::vec::Vec<i32>;
    let mut _13: bool;
    let mut _14: *const [i32; 3];
    let mut _15: *const ();
    let mut _16: usize;
    let mut _17: usize;
    let mut _18: usize;
    let mut _19: usize;
    let mut _20: bool;
    scope 1 {
        debug v => _1;
    }

    bb0: {
        _13 = const false;
        _3 = SizeOf([i32; 3]);
        _4 = AlignOf([i32; 3]);
        _5 = alloc::alloc::exchange_malloc(move _3, move _4) -> [return: bb1, unwind continue];
    }

    bb1: {
        _6 = ShallowInitBox(move _5, [i32; 3]);
        _14 = copy (((_6.0: std::ptr::Unique<[i32; 3]>).0: std::ptr::NonNull<[i32; 3]>).0: *const [i32; 3]);
        _15 = copy _14 as *const () (PtrToPtr);
        _16 = copy _15 as usize (Transmute);
        _17 = AlignOf([i32; 3]);
        _18 = Sub(copy _17, const 1_usize);
        _19 = BitAnd(copy _16, copy _18);
        _20 = Eq(copy _19, const 0_usize);
        assert(copy _20, "misaligned pointer dereference: address must be a multiple of {} but is {}", copy _17, copy _16) -> [success: bb9, unwind unreachable];
    }

    bb2: {
        _13 = const true;
        _8 = &_1;
        _7 = borrow_vec(copy _8) -> [return: bb3, unwind: bb8];
    }

    bb3: {
        _10 = &mut _1;
        _9 = borrow_mut_vec(copy _10) -> [return: bb4, unwind: bb8];
    }

    bb4: {
        _13 = const false;
        _12 = move _1;
        _11 = move_vec(move _12) -> [return: bb5, unwind: bb8];
    }

    bb5: {
        _13 = const false;
        return;
    }

    bb6 (cleanup): {
        resume;
    }

    bb7 (cleanup): {
        drop(_1) -> [return: bb6, unwind terminate(cleanup)];
    }

    bb8 (cleanup): {
        switchInt(copy _13) -> [0: bb6, otherwise: bb7];
    }

    bb9: {
        (*_14) = [const 1_i32, const 2_i32, const 3_i32];
        _2 = move _6 as std::boxed::Box<[i32]> (PointerCoercion(Unsize));
        _1 = slice::<impl [i32]>::into_vec::<std::alloc::Global>(move _2) -> [return: bb2, unwind continue];
    }
}
