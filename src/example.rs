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
use bugeutils::list::{ReusableIndexVec, ID};

use std::io;

pub fn main() {
    println!("It works!");

    let mut mylist = ReusableIndexVec::<String>::new();
    
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)
            .expect("Failed to read line");

        let input: Vec<&str> = input.split(" ").collect();
        let command: &str = input[0];

        match command.to_uppercase().trim() {
            "ADD" => {
                let add_what = String::from(input[1].trim());

                mylist.add(add_what);
            },
            "REM" => {
                let cycle_stamp: u32 = input[1].trim().parse().unwrap_or_else(|err| {
                    println!("ERROR: {} ({})", err, input[1]);
                    println!("INFO: Removing with cycle stamp 0");

                    0
                });
                let index: usize = input[2].trim().parse().unwrap_or_else(|err| {
                    println!("ERROR: {} ({})", err, input[2]);
                    println!("INFO: Removing with index 0");

                    0
                });

                if let Err(e) = mylist.remove(ID(cycle_stamp, index)) {
                    println!("ERROR: {}", e);
                }
            },
            "GET" => {
                let cycle_stamp: u32 = input[1].trim().parse().unwrap_or_else(|err| {
                    println!("ERROR: {} ({})", err, input[1]);
                    println!("INFO: Getting with cycle stamp 0");

                    0
                });
                let index: usize = input[2].trim().parse().unwrap_or_else(|err| {
                    println!("ERROR: {} ({})", err, input[2]);
                    println!("INFO: Getting with index 0");

                    0
                });

                match mylist.get(ID(cycle_stamp, index)) {
                    Some(val) => {
                        println!("GOT={}", val);
                    },
                    None => {
                        println!("EXPIRED");
                    }
                }
            },
            "SHW" => {
                println!("{:?}", mylist);
            },
            "BYE" => break,
            _ => (),
        }
    }
}
