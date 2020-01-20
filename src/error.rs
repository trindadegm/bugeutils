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
use std::{error, fmt};

// Dynamic error type, usually for return values
type DynErr = dyn error::Error + 'static;

#[derive(Debug)]
pub struct Error {
    error_type: ErrorType,
    error_desc: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    /// Used when a function receives invalid parameters.
    InvalidParameter,

    /// Used when a resource was not found.
    NotFound,
    /// Used when a resource expires.
    Expired,

    /// Used when none of the other options fit. Something unexpected.
    UnexpectedError,
}

impl Error {
    pub fn new(error_type: ErrorType, error_desc: &str) -> Self {
        Self {
            error_type,
            error_desc: String::from(error_desc),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}, {}", self.error_type, self.error_desc)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&DynErr> {
        None
    }
}
