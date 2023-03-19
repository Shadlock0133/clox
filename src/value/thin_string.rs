use std::{
    ops::Deref,
    ptr::{addr_of, NonNull},
};

pub struct ThinString(NonNull<Header>);

#[repr(C)]
struct Header {
    capacity: usize,
    len: usize,
}

impl ThinString {
    fn buffer_ptr(&self) -> NonNull<u8> {
        todo!()
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            let ptr = self.0.as_ptr().cast::<HeaderWithData>();
            let slice = std::slice::from_raw_parts(
                addr_of!((*ptr).data).cast(),
                (*ptr).len,
            );
            std::str::from_utf8_unchecked(slice)
        }
    }
}

impl Deref for ThinString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl PartialEq for ThinString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

mod test_impl {
    fn foo() {
        use std::{
            alloc::{alloc, dealloc, Layout},
            ptr::NonNull,
        };
        #[repr(C)]
        struct Header {
            cap: usize,
            len: usize,
        }

        let cap = 4;
        let (layout, offset) = Layout::new::<Header>()
            .extend(Layout::array::<u8>(cap).unwrap())
            .unwrap();

        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            panic!()
        }
        let ptr = NonNull::new(ptr.cast::<Header>()).unwrap();

        unsafe { ptr.as_ptr().cast::<u8>().wrapping_add(offset).write(0) };

        unsafe { dealloc(ptr.as_ptr().cast::<u8>(), layout) };
    }
}
