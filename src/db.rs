use std::io::ErrorKind;

use std::io::SeekFrom;
use std::iter;

use crate::machine::MachineDescription;

use std::io::Result;

use std::path::Path;

use std::fs::File;

use std::io::Read;

use std::io::Seek;

#[repr(C)]
#[derive(Debug)]
pub struct Header {
    pub undecided_time_count: u32,
    pub undecided_size_count: u32,
    pub undecided_total: u32,
    pub lexicographic_sorting: bool,
}

pub struct Db<D: Seek + Read> {
    pub header: Header,
    pub data: D,
}

pub struct Index<D: Seek + Read> {
    pub data: D,
}

fn u32_from_be(slice: &[u8]) -> u32 {
    let array = <[u8; 4]>::try_from(slice).unwrap();
    u32::from_be_bytes(array)
}

impl Db<File> {
    pub fn open(path: impl AsRef<Path>) -> Result<Db<File>> {
        let mut data = File::open(path)?;

        let header_bytes = &mut [0; 13];
        data.read_exact(header_bytes)?;
        let header = Header {
            undecided_time_count: u32_from_be(&header_bytes[..4]),
            undecided_size_count: u32_from_be(&header_bytes[4..8]),
            undecided_total: u32_from_be(&header_bytes[8..12]),
            lexicographic_sorting: header_bytes[12] == 1,
        };

        assert!(header_bytes[12] == 0 || header_bytes[12] == 1);
        assert_eq!(
            header.undecided_total,
            header.undecided_size_count + header.undecided_time_count
        );

        Ok(Db { header, data })
    }
}

impl<D: Seek + Read> Db<D> {
    pub fn read(&mut self, tm: u32) -> Result<MachineDescription> {
        if tm >= self.header.undecided_total {
            panic!(
                "Out of boudns read, tm index: {tm} but we only have {} machines",
                self.header.undecided_total
            );
        }

        self.data.seek(SeekFrom::Start(30 * (tm + 1) as u64))?;
        let machine_bytes = &mut [0; 30];
        self.data.read_exact(machine_bytes)?;

        println!("Machine bytes: {:x?}", machine_bytes);

        Ok(MachineDescription::from_bytes(machine_bytes).clone())
    }
}

impl Index<File> {
    pub fn open(path: impl AsRef<Path>) -> Result<Index<File>> {
        Ok(Index {
            data: File::open(path)?,
        })
    }
}

impl<D: Seek + Read> Index<D> {
    pub fn iter(&mut self) -> impl Iterator<Item = u32> + '_ {
        self.data
            .seek(SeekFrom::Start(0))
            .expect("Failed to seek index");

        iter::from_fn(|| {
            let mut buf = [0; 4];
            match self.data.read_exact(&mut buf) {
                Ok(()) => Some(u32::from_be_bytes(buf)),
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => None,
                Err(e) => panic!("Failed to read index: {e}"),
            }
        })
    }
}
