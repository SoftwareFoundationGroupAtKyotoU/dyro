// WARNING: This output format is intended for human consumers only
// and is subject to change without notice. Knock yourself out.
fn borrow_vec(_1: &Vec<i32>) -> () {
    debug v => _1;
    let mut _0: ();
    let _2: &std::vec::Vec<i32>;
    let mut _3: &std::vec::Vec<i32>;

    bb0: {
        StorageLive(_2);
        StorageLive(_3);
        _3 = copy _1;
        _2 = std::hint::black_box::<&Vec<i32>>(move _3) -> [return: bb1, unwind continue];
    }

    bb1: {
        StorageDead(_3);
        StorageDead(_2);
        _0 = const ();
        return;
    }
}

fn borrow_mut_vec(_1: &mut Vec<i32>) -> () {
    debug v => _1;
    let mut _0: ();
    let _2: ();
    let mut _3: &mut std::vec::Vec<i32>;

    bb0: {
        StorageLive(_2);
        StorageLive(_3);
        _3 = &mut (*_1);
        _2 = Vec::<i32>::push(move _3, const 4_i32) -> [return: bb1, unwind continue];
    }

    bb1: {
        StorageDead(_3);
        StorageDead(_2);
        _0 = const ();
        return;
    }
}

fn move_vec(_1: Vec<i32>) -> () {
    debug v => _1;
    let mut _0: ();
    let _2: ();
    let mut _3: std::vec::Vec<i32>;

    bb0: {
        StorageLive(_2);
        StorageLive(_3);
        _3 = move _1;
        _2 = std::mem::drop::<Vec<i32>>(move _3) -> [return: bb1, unwind: bb2];
    }

    bb1: {
        StorageDead(_3);
        StorageDead(_2);
        _0 = const ();
        return;
    }

    bb2 (cleanup): {
        resume;
    }
}

fn main() -> () {
    let mut _0: ();
    let mut _1: std::vec::Vec<i32>;
    let mut _2: std::boxed::Box<[i32]>;
    let mut _3: std::boxed::Box<[i32; 3]>;
    let mut _4: usize;
    let mut _5: usize;
    let mut _6: *mut u8;
    let mut _7: std::boxed::Box<[i32; 3]>;
    let _8: ();
    let mut _9: &std::vec::Vec<i32>;
    let _10: &std::vec::Vec<i32>;
    let _11: ();
    let mut _12: &mut std::vec::Vec<i32>;
    let mut _13: &mut std::vec::Vec<i32>;
    let _14: ();
    let mut _15: std::vec::Vec<i32>;
    let mut _16: bool;
    let mut _17: *const [i32; 3];
    let mut _18: *const ();
    let mut _19: usize;
    let mut _20: usize;
    let mut _21: usize;
    let mut _22: usize;
    let mut _23: bool;
    scope 1 {
        debug v => _1;
    }

    bb0: {
        _16 = const false;
        StorageLive(_1);
        StorageLive(_2);
        StorageLive(_3);
        _4 = SizeOf([i32; 3]);
        _5 = AlignOf([i32; 3]);
        _6 = alloc::alloc::exchange_malloc(move _4, move _5) -> [return: bb1, unwind continue];
    }

    bb1: {
        StorageLive(_7);
        _7 = ShallowInitBox(move _6, [i32; 3]);
        _17 = copy (((_7.0: std::ptr::Unique<[i32; 3]>).0: std::ptr::NonNull<[i32; 3]>).0: *const [i32; 3]);
        _18 = copy _17 as *const () (PtrToPtr);
        _19 = copy _18 as usize (Transmute);
        _20 = AlignOf([i32; 3]);
        _21 = Sub(copy _20, const 1_usize);
        _22 = BitAnd(copy _19, copy _21);
        _23 = Eq(copy _22, const 0_usize);
        assert(copy _23, "misaligned pointer dereference: address must be a multiple of {} but is {}", copy _20, copy _19) -> [success: bb9, unwind unreachable];
    }

    bb2: {
        _16 = const true;
        StorageDead(_2);
        StorageLive(_8);
        StorageLive(_9);
        StorageLive(_10);
        _10 = &_1;
        _9 = &(*_10);
        _8 = borrow_vec(move _9) -> [return: bb3, unwind: bb8];
    }

    bb3: {
        StorageDead(_9);
        StorageDead(_10);
        StorageDead(_8);
        StorageLive(_11);
        StorageLive(_12);
        StorageLive(_13);
        _13 = &mut _1;
        _12 = &mut (*_13);
        _11 = borrow_mut_vec(move _12) -> [return: bb4, unwind: bb8];
    }

    bb4: {
        StorageDead(_12);
        StorageDead(_13);
        StorageDead(_11);
        StorageLive(_14);
        StorageLive(_15);
        _16 = const false;
        _15 = move _1;
        _14 = move_vec(move _15) -> [return: bb5, unwind: bb8];
    }

    bb5: {
        StorageDead(_15);
        StorageDead(_14);
        _0 = const ();
        _16 = const false;
        StorageDead(_1);
        return;
    }

    bb6 (cleanup): {
        resume;
    }

    bb7 (cleanup): {
        drop(_1) -> [return: bb6, unwind terminate(cleanup)];
    }

    bb8 (cleanup): {
        switchInt(copy _16) -> [0: bb6, otherwise: bb7];
    }

    bb9: {
        (*_17) = [const 1_i32, const 2_i32, const 3_i32];
        _3 = move _7;
        _2 = move _3 as std::boxed::Box<[i32]> (PointerCoercion(Unsize));
        StorageDead(_7);
        StorageDead(_3);
        _1 = slice::<impl [i32]>::into_vec::<std::alloc::Global>(move _2) -> [return: bb2, unwind: bb6];
    }
}
