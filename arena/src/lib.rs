#![feature(allocator_api)]
use std::alloc::{Allocator, AllocError, Layout};
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::ptr::addr_of;
use std::cell::UnsafeCell;

pub const KB: usize = 1024;
pub const MB: usize = 1024 * KB;

pub struct Arena<'a> {
    _tag: PhantomData<*mut &'a ()>,

    /// our data
    data: UnsafeCell<ArenaData>
}

#[derive(Debug)]
pub(crate) struct ArenaData {
    /// underlying storage for the arena
    pub(crate) storage: Vec<u8>,

    /// base address of the storage
    pub(crate) base_address: usize,

    /// current free offset into the storage
    pub(crate) offset: usize,
}


impl<'a> Arena<'a> {
    pub fn with<R>(bytes: usize, k: impl for <'b> FnOnce(&Arena<'b>) -> R) -> R {

        // TODO: could use uninitialised memory in non-debug scenario?
        let storage = vec![0u8; bytes];
        let base_address = addr_of!(storage[0]) as usize;
        let offset = 0;

        let data = UnsafeCell::new(ArenaData {
            storage,
            base_address,
            offset,
        });

        k(&Arena {
            _tag: PhantomData,
            data,
        })
    }

    pub(crate) unsafe fn get_data(&self) -> &mut ArenaData {
        &mut *self.data.get()
    }
}

unsafe impl<'a> Allocator for &Arena<'a> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let data = unsafe { self.get_data() };

        // get the address of the base
        let base = data.base_address;
        let alignment = layout.align();
        // align the base address
        let aligned = (base + (alignment - 1)) & (!alignment + 1);
        // calculate the amount we need to add to base to have a correctly aligned address
        let alignment = aligned - base;

        // size of the allocation adjusted for alignment at the front
        let total_size = alignment + layout.size();

        let current_offset: usize = data.offset;

        // space leftover in storage
        let space = data.storage.len() - current_offset;

        // check if there is enough space in the arena
        if total_size > space {
            return Err(AllocError)
        }

        // update offset
        data.offset += total_size;

        Ok(unsafe {
            NonNull::new_unchecked(
                &mut data.storage[current_offset+alignment..layout.size()])
        })
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // do nothing when allocating, could put something when in debug?
        ()
    }
}

#[cfg(test)]
mod tests {
    use std::vec::Vec;
    use crate::*;

    #[test]
    fn it_works() {
        Arena::with(4 * MB, |arena| {
            let mut vec: Vec<u8, _> = Vec::with_capacity_in(12, arena);
            vec.push(1);
            println!(
                "0x{:x?},\n0x{:x?}",
                unsafe { arena.get_data() }.base_address,
                addr_of!(vec[0]) as usize);

            println!("{:?}", vec);
        });
    }

    #[test]
    fn allocate_too_much() {

        Arena::with(4 * KB, |arena| {
            let mut vec = Vec::<u8, _>::new_in(arena);
            let res = vec.try_reserve(5 * KB);

            println!("{:?}", res);

            // reserve should fail
            assert!(matches!(res, Err(_)));
        });
    }

    #[test]
    fn aligned_ok() {

        #[derive(Debug)]
        #[repr(C, align(1024))]
        struct AlignMe(u8);

        Arena::with(4 * KB, |arena| {
            let b = Box::new_in(AlignMe(1), arena);

            println!("{:x?}", addr_of!(*b));
            println!("{:x?}", b);
            // 16 byte aligned
            assert!(0 == (addr_of!(*b) as usize & (1024 - 1)));

            let b = Box::new(AlignMe(2));

            println!("{:x?}", addr_of!(*b));
            println!("{:x?}", b);
            // 16 byte aligned
            assert!(0 == (addr_of!(*b) as usize & 0xf));
        });
    }
}
