/* *****************************************************************************
 MIT License
 
 Copyright (c) 2020 trindadegm
 
 Permission is hereby granted, free of charge, to any person obtaining a copy
 of this software and associated documentation files (the "Software"), to deal
 in the Software without restriction, including without limitation the rights
 to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 copies of the Software, and to permit persons to whom the Software is
 furnished to do so, subject to the following conditions:
 
 The above copyright notice and this permission notice shall be included in all
 copies or substantial portions of the Software.
 
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 SOFTWARE.
***************************************************************************** */
use crate::error::{Error as BugeError, ErrorType as BugeErrorType};

/**
 * Ok, this module is very unsafe. This still needs a lot of testing.
 */

use std::any::TypeId;
use std::num::NonZeroUsize;

type BlackBoxResult<T> = Result<T, BugeError>;

/// This struct is intended to be used when the programmer needs something similar to a C void
/// pointer, or a Java Object. It is inferior to a `Box` in almost every single way. The
/// only advantage it has is that it is flexible with the type it holds.
///
/// A `BlackBox` can be created to hold a value in the heap. References to that value can be
/// retrieved with `get_ref` and `get_mut_ref`.
///
/// ```
/// use bugeutils::black_box::BlackBox;
///
/// // Storing an u64 in the heap
/// let boxed_u64: BlackBox = BlackBox::new(123_u64);
///
/// // Storing an f32 in the heap
/// let boxed_f32: BlackBox = BlackBox::new(0.5_f32);
///
/// // Notice that the types of `boxed_u64` and `boxed_f32` are the same: `BlackBox`. Even though
/// // they hold the ownership to values for different types (and sizes!).
///
/// // This prints "This is a 64 bit unsigned integer: 123"
/// println!("This is a 64 bit unsigned integer: {}", boxed_u64.get_ref::<u64>().unwrap());
///
/// // This prints "This is a 32 bit floating point value: 0.5"
/// println!("This is a 32 bit floating point value: {}", boxed_f32.get_ref::<f32>().unwrap());
///
/// // You must request the correct type when retrieving the value
/// assert!(boxed_f32.get_ref::<f32>().is_ok());
/// assert!(boxed_f32.get_ref::<f64>().is_err());
/// ```
///
/// In essence, a `BlackBox` does not know which type it holds. This means it is possible to store
/// any type in it, without the need for a template argument. The downside is the lack of type
/// safety, as it checks for those in runtime instead of compile time, meaning it may return an
/// error variant when trying to retrieve the value. It must not, however, cause undefined
/// behaviour.
///
/// When dropped, the `BlackBox` will drop the element it owns correctly.
pub struct BlackBox {
    type_id: TypeId,
    dropper: Box<dyn Fn(NonZeroUsize)>,
    content_ptr: NonZeroUsize,
}

impl BlackBox {
    /// Creates a new `BlackBox`, taking ownership of `value` and storing it on the heap.
    pub fn new<T>(value: T) -> Self
    where T: 'static {
        let boxed_value = Box::new(value);
        let value_heap_ptr: *mut T = Box::into_raw(boxed_value);

        let dropper = Box::new(|usize_ptr: NonZeroUsize| {
            let t_ptr: *mut T = usize_ptr.get() as *mut T;
            // XXX Important! This pointer MUST be a good pointer. Look above, the pointer
            // should've been created like that.
            let _reboxed = unsafe { Box::from_raw(t_ptr) };
            // Reboxed should then be dropped at the end of the closure.
        });

        Self {
            type_id: TypeId::of::<T>(),
            dropper,
            content_ptr: NonZeroUsize::new(value_heap_ptr as usize).unwrap(),
        }
    }

    /// If `T` is the type of the value owned by `BlackBox`, returns an `Ok` variant with a
    /// reference to that value. Otherwise returns an `Err` variant.
    pub fn get_ref<T>(&self) -> BlackBoxResult<&T>
    where T: 'static {
        if TypeId::of::<T>() == self.type_id {
            let content_ptr = self.content_ptr.get();
            let content_t_ptr = content_ptr as *const T;

            // XXX Important! This pointer MUST be a good pointer.
            unsafe {
                Ok(&(*content_t_ptr))
            }
        } else {
            Err(BugeError::new(BugeErrorType::NotCompatible, &format!("Incorrect unboxing type")))
        }
    }

    /// If `T` is the type of the value owned by `BlackBox`, returns an `Ok` variant with a
    /// mutable reference to that value. Otherwise returns an `Err` variant.
    pub fn get_mut_ref<T>(&mut self) -> BlackBoxResult<&mut T>
    where T: 'static {
        if TypeId::of::<T>() == self.type_id {
            let content_ptr = self.content_ptr.get();
            let content_t_ptr = content_ptr as *mut T;

            // XXX Important! This pointer MUST be a good pointer.
            unsafe {
                Ok(&mut (*content_t_ptr))
            }
        } else {
            Err(BugeError::new(BugeErrorType::NotCompatible, &format!("Incorrect unboxing type")))
        }
    }

    /// Returns a reference `&T` to the value owned by this `BlackBox`.
    ///
    /// # Safety
    /// This function does not check whether or not the value it holds a pointer to is or is not of
    /// type `T`. This means calling this with the wrong type `T` will cause it to reinterpret the
    /// type of the value. This may cause several problems.
    ///
    /// Therefore, make sure the `BlackBox` was created to hold a value of type `T` before calling
    /// this method.
    ///
    /// # Example
    /// ```
    /// use bugeutils::black_box::BlackBox;
    /// // Created a BlackBox that holds a value of type [f32; 4]
    /// let array_in_heap: BlackBox = BlackBox::new::<[f32; 4]>([1.0, 2.0, 3.0, 4.0]);
    ///
    /// // Now let's try to retrieve a reference of type [f64; 8], which is much bigger.
    /// // This is safe, it returns an error.
    /// let retrieved = array_in_heap.get_ref::<[f64; 8]>(); // Unwrapping this would cause a panic
    ///
    /// // Now the unsafe get_ref_unchecked variant:
    /// unsafe {
    ///     // get_ref_unchecked will allow the reinterpretation of the pointer.
    ///     let retrieved = array_in_heap.get_ref_unchecked::<[f64; 8]>();
    ///     // Now 'retrieved' contains a bad reference, and reading it would cause undefined
    ///     // behaviour.
    /// }
    /// ```
    pub unsafe fn get_ref_unchecked<T>(&self) -> &T {
        let content_ptr = self.content_ptr.get();
        let content_t_ptr = content_ptr as *const T;

        // XXX Important! This pointer MUST be a good pointer.
        &(*content_t_ptr)
    }

    /// Returns a reference `&mut T` to the value owned by this `BlackBox`.
    ///
    /// # Safety
    /// This function does not check whether or not the value it holds a pointer to is or is not of
    /// type `T`. This means calling this with the wrong type `T` will cause it to reinterpret the
    /// type of the value. This may cause several problems.
    ///
    /// Therefore, make sure the `BlackBox` was created to hold a value of type `T` before calling
    /// this method.
    ///
    /// # Example
    /// ```
    /// use bugeutils::black_box::BlackBox;
    /// // Created a BlackBox that holds a value of type [f32; 4]
    /// let mut array_in_heap: BlackBox = BlackBox::new::<[f32; 4]>([1.0, 2.0, 3.0, 4.0]);
    ///
    /// // Now let's try to retrieve a reference of type [f64; 8], which is much bigger.
    /// // This is safe, it returns an error.
    /// let retrieved = array_in_heap.get_mut_ref::<[f64; 8]>();
    /// // Unwrapping that would cause a panic.
    ///
    /// // Now the unsafe get_mut_ref_unchecked variant:
    /// unsafe {
    ///     // get_mut_ref_unchecked will allow the reinterpretation of the pointer.
    ///     let retrieved = array_in_heap.get_mut_ref_unchecked::<[f64; 8]>();
    ///     // Now 'retrieved' contains a bad reference, and reading it would cause undefined
    ///     // behaviour.
    /// }
    /// ```
    pub unsafe fn get_mut_ref_unchecked<T>(&mut self) -> &mut T {
        let content_ptr = self.content_ptr.get();
        let content_t_ptr = content_ptr as *mut T;

        // XXX Important! This pointer MUST be a good pointer.
        &mut (*content_t_ptr)
    }
}

impl std::fmt::Debug for BlackBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlackBox {{ type_id: {:?}, content_ptr: {:?}, dropper: ... }}", self.type_id, self.content_ptr)
    }
}

impl Drop for BlackBox {
    fn drop(&mut self) {
        // XXX Important! This should make the cleanup correctly.
        (self.dropper)(self.content_ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyDropZST { }

    impl Drop for DummyDropZST {
        fn drop(&mut self) {
            println!("Dropping DummyDropZST {:?}", self as *const DummyDropZST);
        }
    }

    struct DummyDropST {
        dummy_text: String,
    }

    impl Drop for DummyDropST {
        fn drop(&mut self) {
            println!("Dropping DummyDropST {:?}: {}", self as *const DummyDropST, self.dummy_text);
        }
    }

    #[test]
    fn creation_and_destruction_test() {
        let mut boxed: BlackBox = BlackBox::new(200_u32);

        assert_eq!(*boxed.get_ref::<u32>().unwrap(), 200_u32);

        *boxed.get_mut_ref::<u32>().unwrap() = 300;

        assert_ne!(*boxed.get_ref::<u32>().unwrap(), 200_u32);
        assert_eq!(*boxed.get_ref::<u32>().unwrap(), 300_u32);
    }

    #[test]
    fn dropping_prints_test() {
        let _boxed1: BlackBox = BlackBox::new(DummyDropZST { });
        let _boxed2: BlackBox = BlackBox::new(DummyDropZST { });
        let _boxed3: BlackBox = BlackBox::new(DummyDropZST { });
        let _boxed4: BlackBox = BlackBox::new(DummyDropZST { });

        let _boxed5: BlackBox = BlackBox::new(DummyDropST { dummy_text: String::from("Some dummy text 5") });
        let _boxed6: BlackBox = BlackBox::new(DummyDropST { dummy_text: String::from("Some dummy text 6") });
        let _boxed7: BlackBox = BlackBox::new(DummyDropST { dummy_text: String::from("Some dummy text 7") });
        let _boxed8: BlackBox = BlackBox::new(DummyDropST { dummy_text: String::from("Some dummy text 8") });
    }

    #[test]
    fn wrong_unboxing_test() {
        let mut boxed: BlackBox = BlackBox::new(128_u32);

        assert!(boxed.get_ref::<u32>().is_ok());
        assert!(boxed.get_ref::<i32>().is_err());
        assert!(boxed.get_ref::<String>().is_err());
        assert!(boxed.get_ref::<Vec<i32>>().is_err());

        assert!(boxed.get_mut_ref::<u32>().is_ok());
        assert!(boxed.get_mut_ref::<i32>().is_err());
        assert!(boxed.get_mut_ref::<String>().is_err());
        assert!(boxed.get_mut_ref::<Vec<i32>>().is_err());
    }

    #[test]
    fn unsafe_methods_test() {
        let mut boxed: BlackBox = BlackBox::new(512_u64);

        unsafe {
            assert_eq!(*boxed.get_ref_unchecked::<u64>(), 512_u64);
        }

        unsafe {
            assert_eq!(*boxed.get_mut_ref_unchecked::<u64>(), 512_u64);
        }
    }
}
