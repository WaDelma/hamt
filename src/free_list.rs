use hash_stream::BITS_IN_POINTER;
use std::mem::ManuallyDrop;
use std::mem;
use std::ptr;

// TODO: Implement block allocator
const BLOCK_SIZE: usize = 1024;
// TODO: Implement free list defragmentation
const DEFRAG_TRESHOLD: f32 = 0.4;

enum Link<T> {
    Cons(T, Box<Link<T>>),
    Nil
}

pub struct FreeList<T>([Link<Box<[ManuallyDrop<T>]>>; BITS_IN_POINTER]);

unsafe fn manualify<T>(slice: Box<[T]>) -> Box<[ManuallyDrop<T>]> {
    mem::transmute(slice)
}

unsafe fn automify<T>(slice: Box<[ManuallyDrop<T>]>) -> Box<[T]> {
    mem::transmute(slice)
}

impl<T> FreeList<T> {
    pub fn new() -> Self {
        FreeList(array![Link::Nil; BITS_IN_POINTER])
    }

    pub fn singleton(&mut self, uno: T) -> Box<[T]> {
        use self::Link::*;
        unsafe {
            match mem::replace(&mut self.0[0], Nil) {
                Cons(c, n) => {
                    self.0[0] = *n;
                    let mut res = automify(c);
                    
                    ptr::write(res.as_mut_ptr(), uno);
                    
                    res
                },
                Nil => vec![uno].into_boxed_slice(),
            }
        }
    }

    pub fn pair(&mut self, fst: T, snd: T) -> Box<[T]> {
        use self::Link::*;
        unsafe {
            match mem::replace(&mut self.0[1], Nil) {
                Cons(c, n) => {
                    self.0[1] = *n;
                    let mut res = automify(c);
                    
                    ptr::write(res.as_mut_ptr(), fst);
                    ptr::write(res.as_mut_ptr().offset(1), snd);
                    
                    res
                },
                Nil => vec![fst, snd].into_boxed_slice(),
            }
        }
    }

    pub fn push(&mut self, slice: Box<[T]>, new: T) -> Box<[T]> {
        use self::Link::*;
        assert!(!slice.is_empty());
        unsafe {
            let len = slice.len();
            match mem::replace(&mut self.0[len], Nil) {
                Cons(c, n) => {
                    self.0[len] = *n;
                    let mut res = automify(c);

                    ptr::copy_nonoverlapping(slice.as_ptr(), res.as_mut_ptr(), len);
                    self.free_internal(manualify(slice));
                    ptr::write(res.as_mut_ptr().offset(len as isize), new);
                    
                    res
                },
                Nil => {
                    let mut res = Vec::<T>::with_capacity(len + 1);
                    res.set_len(len + 1);

                    ptr::copy_nonoverlapping(slice.as_ptr(), res.as_mut_ptr(), len);
                    self.free_internal(manualify(slice));
                    ptr::write(res.as_mut_ptr().offset(len as isize), new);

                    res.into_boxed_slice()
                }
            }
        }
    }

    fn free_internal(&mut self, b: Box<[ManuallyDrop<T>]>) {
        use self::Link::*;
        assert!(!b.is_empty());
        let i = b.len() - 1;
        let old = mem::replace(&mut self.0[i], Nil);
        self.0[i] = Cons(b, Box::new(old));
    }

    pub fn free(&mut self, mut b: Box<[T]>) {
        unsafe {
            ptr::drop_in_place(&mut b as *mut _);
            self.free_internal(manualify(b));
        }
    }
}