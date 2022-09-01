use std::{io::{Seek, Read, Result, SeekFrom, ErrorKind}, path::Path, fs::File, mem, iter};

#[repr(C)]
#[derive(Debug)]
struct Header {
    undecided_time_count: u32,
    undecided_size_count: u32,
    undecided_total: u32,
    lexicographic_sorting: bool,
}

struct Db<D: Seek + Read> {
    header: Header,
    data: D,
}

struct Index<D: Seek + Read> {
    data: D,
}

fn u32_from_be(slice: &[u8]) -> u32 {
    let array = <[u8; 4]>::try_from(slice).unwrap();
    u32::from_be_bytes(array)
}

impl Db<File> {
    fn open(path: impl AsRef<Path>) -> Result<Db<File>> {
        let mut data = File::open(path)?;

        let header_bytes = &mut [0; 13];
        data.read_exact(header_bytes)?;
        let header =  Header {
            undecided_time_count: u32_from_be(&header_bytes[.. 4]),
            undecided_size_count: u32_from_be(&header_bytes[4.. 8]),
            undecided_total: u32_from_be(&header_bytes[8.. 12]),
            lexicographic_sorting: header_bytes[12] == 1,
        };

        assert!(header_bytes[12] == 0 || header_bytes[12] == 1);
        assert_eq!(header.undecided_total, header.undecided_size_count + header.undecided_time_count);

        Ok(Db {
            header,
            data
        })
    }
}

impl<D: Seek + Read> Db<D> {
    fn read(&mut self, tm: u32) -> Result<MachineDescription> {
        if tm >= self.header.undecided_total {
            panic!("Out of boudns read, tm index: {tm} but we only have {} machines", self.header.undecided_total);
        }

        self.data.seek(SeekFrom::Start(30 * (tm + 1) as u64))?;
        let machine_bytes = &mut [0; 30];
        self.data.read_exact(machine_bytes)?;

        println!("Machine bytes: {:x?}", machine_bytes);

        Ok(MachineDescription::from_bytes(machine_bytes).clone())
    }
}

impl Index<File> {
    fn open(path: impl AsRef<Path>) -> Result<Index<File>> {
        Ok(Index{ data: File::open(path)? })
    }
}

impl<D: Seek + Read> Index<D> {
    fn iter(&mut self) -> impl Iterator<Item = u32> + '_ {
        self.data.seek(SeekFrom::Start(0)).expect("Failed to seek index");

        iter::from_fn(|| {
            let mut buf = [0;  4];
            match self.data.read_exact(&mut buf) {
                Ok(()) => Some(u32::from_be_bytes(buf)),
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => None,
                Err(e) => panic!("Failed to read index: {e}"),
            }
        })
    }
}

#[allow(dead_code)] // False positive
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum Dir {
    R = 0,
    L = 1,
}

#[allow(dead_code)] // False positive
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum NewState {
    Undef = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum State {
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Transition {
    out: bool,
    dir: Dir,
    new_state: NewState,
}

#[repr(C)]
#[derive(Clone, Debug)]
struct MachineDescription {
    transitions: [Transition; 10]
}

impl MachineDescription {
    fn from_bytes(bytes: &[u8]) -> &MachineDescription {
        assert_eq!(bytes.len(), 30);
        assert_eq!(30, mem::size_of::<MachineDescription>());
        assert_eq!(1, mem::align_of::<MachineDescription>());

        unsafe{ &* (bytes.as_ptr() as *const MachineDescription) }
    }
}

struct Machine {
    description: MachineDescription,
    head: usize,
    cells_below_zero: usize,
    state: State,
    tape: Vec<bool>,
}

impl Transition {
    fn state(&self) -> Option<State> {
        match self.new_state {
            NewState::Undef => None,
            NewState::A => Some(State::A),
            NewState::B => Some(State::B),
            NewState::C => Some(State::C),
            NewState::D => Some(State::D),
            NewState::E => Some(State::E),
        }
    }
}

impl Machine {
    fn new(description: MachineDescription) -> Self {
        Machine {
            description,
            head: 0,
            cells_below_zero: 0,
            state: State::A,
            tape: vec![false]
        }
    }

    /// Returns true if the machine has halted
    fn step(&mut self) -> bool {
        let transition_index = (self.state as usize - 1) * 2 + self.tape[self.head] as usize;

        let transition = self.description.transitions[transition_index];
        self.state = match transition.state() {
            Some(s) => s,
            None => return true,
        };

        self.tape[self.head] = transition.out;
        match (self.head, transition.dir) {
            (0, Dir::L) => {
                self.tape.insert(0, false);
                self.cells_below_zero += 1;
            }
            (head, Dir::L) => self.head = head - 1,
            (head, Dir::R) => {
                if head + 1 == self.tape.len() {
                    self.tape.push(false);
                }
                self.head = head + 1;
            },
        }

        false
    }

    fn tape_str(&self, left_padding: usize) -> String {
        if left_padding < self.cells_below_zero {
            panic!("Out of space to print");
        }

        let rem_padding = left_padding - self.cells_below_zero;
        let mut out = String::new();
        for _ in 0.. rem_padding {
            out.push('_');
        }
        for (i, val) in self.tape.iter().copied().enumerate() {
            if i == self.head {
                let mut state_char = format!("{:?}", self.state);
                if !val {
                    state_char = state_char.to_lowercase();
                }
                out.push_str(&state_char)
            } else {
                if val {
                    out.push('■')
                } else {
                    out.push('□')
                }
            }
        }
        out
    }
}

fn main() {
    let mut db = Db::open("all_5_states_undecided_machines_with_global_header")
        .expect("Failed to open db");

    let mut undecided_index = Index::open("bb5_undecided_index")
        .expect("Failed to open index");

    for index in undecided_index.iter().take(5) {
        let description = db.read(index)
            .expect("Failed to read machine");

        for transition in description.transitions {
            println!("{transition:?}");
        }

        let mut machine = Machine::new(description);

        println!("{:?} {}", machine.state, machine.tape_str(10));
        for _ in 0.. 10 {
            if machine.step() {
                println!("Halted");
                break
            }
            println!("{:?} {}", machine.state, machine.tape_str(10));
        }
    }
}
