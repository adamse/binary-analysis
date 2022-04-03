#![feature(bool_to_option)]
#![feature(maybe_uninit_as_bytes)]
#![feature(new_uninit)]
#![feature(read_buf)]

use std::fs::File;
use std::mem::MaybeUninit;
use std::io::{ReadBuf, self, Read};

#[repr(C)]
#[derive(Debug)]
struct Elf64Header {
    /// Magic number and other info
    ident: [u8; 16],

    /// Object file type
    ///
    /// todo: change to enum
    r#type: u16,

    /// Architecture
    machine: u16,

    /// Object file version
    version: u32,

    /// Entry point virtual address
    entry: u64,

    /// Program header table file offset
    ///
    /// Offset from start of file (including this header).
    phoff: u64,

    /// Section header table file offset
    ///
    /// Offset from start of file (including this header).
    shoff: u64,

    /// Processor-specific flags
    flags: u32,

    /// ELF header size in bytes
    ehsize: u16,

    /// Program header table entry size
    phentsize: u16,

    /// Program header table entry count
    phnum: u16,

    /// Section header table entry size
    shentsize: u16,

    /// Section header table entry count
    shnum: u16,

    /// Section header string table index
    shstrndx: u16,
}

/// A heap allocated buffer aligned like T.
struct Buffer<T>{
    data: Box<MaybeUninit<T>>,
}

/// A zero zero intialised heap allocation to read data from a file into.
impl<T> Buffer<T> {

    ///
    pub fn new() -> Self {
        Buffer {
            data: Box::new_zeroed(),
        }
    }

    /// Get a reference to write into the buffer.
    pub fn read_buf(&mut self) -> ReadBuf<'_> {
        ReadBuf::uninit(self.data.as_bytes_mut())
    }

    /// Get the value.
    ///
    /// Safe if you've only written valid things into the buffer and/or all 0s
    /// are a valid value.
    pub unsafe fn get(self) -> Box<T> {
        self.data.assume_init()
    }
}

#[derive(Debug)]
enum Error {
    Io(io::Error),

    /// An error parsing the ELF
    ///
    /// TODO: enum instead of string
    ElfParser(String),
}

fn main() -> Result<(), Error> {
    let mut buf: Buffer<Elf64Header> = Buffer::new();

    let mut file = File::open("../simple").map_err(Error::Io)?;

    file.read_buf_exact(&mut buf.read_buf()).map_err(Error::Io)?;

    let header = unsafe { buf.get() };

    header.ident.starts_with(&[0x7f, 0x45, 0x4c, 0x46])
        .then_some(())
        .ok_or(
            Error::ElfParser(
                format!("header did not start with 0x7f ELF: {:x?}", header.ident)))?;

    (header.ehsize as usize == std::mem::size_of::<Elf64Header>())
        .then_some(())
        .ok_or(Error::ElfParser(
                format!(
                    "read header size does not match expected header size, expected: {}, got: {}",
                    std::mem::size_of::<Elf64Header>(),
                    header.ehsize)))?;

    println!("{:x?}", header);

    Ok(())
}
