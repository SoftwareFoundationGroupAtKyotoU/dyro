// Opaque, probably some kind of annotation in the language
struct Unique<T>(pub T);

// Opaque, but we kind of need an implementation to test here
fn alloc(length: i32) -> *mut i32 {
    let ptr = unsafe {
        std::alloc::alloc(std::alloc::Layout::from_size_align(length as usize * 4, 4).unwrap())
    };
    ptr as *mut i32
}

// Opaque, but we kind of need an implementation to test here
fn dealloc(ptr: *mut i32, length: i32) {
    unsafe {
        std::alloc::dealloc(
            ptr as *mut u8,
            std::alloc::Layout::from_size_align(length as usize * 4, 4).unwrap(),
        );
    }
}

/// Since the new language has only i32, i32 is used instead of usize
pub struct VecInt {
    ptr: Unique<*mut i32>,
    cap: i32,
    len: i32,
}

// We accept &mut self here but we will accept self in the language
impl Drop for VecInt {
    fn drop(&mut self) {
        if self.cap != 0 {
            dealloc(self.ptr.0, self.cap);
        }
    }
}

impl VecInt {
    pub fn new() -> Self {
        VecInt {
            // Should be enough alignment
            ptr: Unique(8 as *mut i32),
            cap: 0,
            len: 0,
        }
    }

    pub fn get(self, index: i32) -> (Self, i32) {
        if index < 0 || index >= self.len {
            panic!("index out of bounds");
        }

        let val = unsafe { *self.ptr.0.offset(index.try_into().unwrap()) };
        (self, val)
    }

    fn grow_one(self) -> Self {
        let cap = (self.cap * 2).max(4);
        let new_ptr = alloc(cap);
        for i in 0..self.len {
            unsafe {
                *new_ptr.offset(i.try_into().unwrap()) = *self.ptr.0.offset(i.try_into().unwrap());
            }
        }
        if self.cap != 0 {
            dealloc(self.ptr.0, self.cap);
        }
        VecInt {
            ptr: Unique(new_ptr),
            cap,
            len: self.len,
        }
    }

    pub fn push(mut self, value: i32) -> Self {
        let len = self.len;
        if len == self.cap {
            self = self.grow_one();
        }
        unsafe {
            *self.ptr.0.offset(len.try_into().unwrap()) = value;
            self.len += 1;
        }
        self
    }
}
