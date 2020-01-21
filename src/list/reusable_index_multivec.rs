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

use crate::list::{ListResult, CycleStamp, Index, ID};

use std::collections::HashMap;
use std::any::TypeId;

pub struct ReusableIndexMultivec {
    //bookkeeper: Vec<
    vector_map: HashMap<TypeId, usize>,
    top_size: usize,
}

impl ReusableIndexMultivec {
    pub fn insert_row<K>(&mut self) -> ListResult<()>
    where K: Sized + 'static {
        let id = TypeId::of::<K>();
        if self.vector_map.contains_key(&id) {
            Err(BugeError::new(BugeErrorType::InvalidParameter, &format!("Key already exists")))
        } else {
            let vec_on_heap = Box::new(Vec::<K>::new());
            self.vector_map.insert(id, 0);
            Ok(())
        }
    }

    //pub fn get_row<K>(&mut self) -> Option<T>
    //where K: ?Sized + 'static {
    //    let id = TypeId::of::<K>();
    //    if Some(addr_usize) = self.vector_map.get(id) {
    //    } else {
    //    }
    //}
}
