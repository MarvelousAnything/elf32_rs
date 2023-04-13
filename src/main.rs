use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read, Result, Seek, SeekFrom};

#[derive(Debug)]
struct Elf<'a, R: Read + Seek> {
    buffer: &'a mut BufReader<R>,
    elf_header: ElfHeader,
    program_headers: Vec<ProgramHeader>,
    section_headers: Vec<SectionHeader>,
}

#[repr(C, packed)]
#[derive(Debug, Default, Clone, Copy)]
struct ElfHeader {
    ident: [u8; 16],
    typ: u16,
    machine: u16,
    version: u32,
    entry: u32,
    phoff: u32,
    shoff: u32,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[repr(C, packed)]
#[derive(Debug, Default)]
struct ProgramHeader {
    typ: u32,
    offset: u32,
    vaddr: u32,
    paddr: u32,
    filesz: u32,
    memsz: u32,
    flags: u32,
    align: u32,
}

#[repr(C, packed)]
#[derive(Debug, Default)]
struct SectionHeader {
    name: [u8; 4],
    typ: u32,
    flags: u32,
    addr: u32,
    offset: u32,
    size: u32,
    link: u32,
    info: u32,
    addralign: u32,
    entsize: u32,
}

impl<'a, R: Read + Seek> Elf<'a, R> {
    fn load_buffer(buffer: &'a mut BufReader<R>) -> Result<Self> {
        let elf_header = ElfHeader::read_elf_header(buffer)?;
        let program_headers = ProgramHeader::read_program_headers(buffer, &elf_header)?;
        let section_headers = SectionHeader::read_section_headers(buffer, &elf_header)?;
        Ok(Self {
            buffer,
            elf_header,
            program_headers,
            section_headers,
        })
    }

    fn read_program_bytes(&mut self, idx: usize) -> Result<Vec<u8>> {
        let program_header = self.program_headers.get(idx).unwrap();
        let mut out_buffer: Vec<u8> = Vec::with_capacity(program_header.filesz as usize);

        let offset = program_header.offset;

        self.buffer.seek(SeekFrom::Start(offset as u64))?;
        let mut handle = self.buffer.take(program_header.filesz as u64);
        handle.read_to_end(&mut out_buffer)?;
        Ok(out_buffer)
    }
}

impl ElfHeader {
    fn read_elf_header<R: Read>(buffer: &mut BufReader<R>) -> Result<ElfHeader> {
        let mut header: ElfHeader = unsafe { std::mem::zeroed() };
        let header_size = std::mem::size_of::<ElfHeader>();
        unsafe {
            let header_slice =
                std::slice::from_raw_parts_mut(&mut header as *mut _ as *mut u8, header_size);
            buffer.read_exact(header_slice)?;
        }
        Ok(header)
    }
}

impl ProgramHeader {
    fn read_program_headers<R: Read + Seek>(
        buffer: &mut BufReader<R>,
        elf_header: &ElfHeader,
    ) -> Result<Vec<ProgramHeader>> {
        let mut headers: Vec<ProgramHeader> = vec![];
        buffer.seek(SeekFrom::Start(elf_header.phoff as u64))?;
        for _ in 0..elf_header.phnum {
            let mut header: ProgramHeader = unsafe { std::mem::zeroed() };
            unsafe {
                let header_slice = std::slice::from_raw_parts_mut(
                    &mut header as *mut _ as *mut u8,
                    elf_header.phentsize as usize,
                );
                buffer.read_exact(header_slice)?;
                headers.push(header);
            }
        }

        Ok(headers)
    }
}

impl SectionHeader {
    fn read_section_headers<R: Read + Seek>(
        buffer: &mut BufReader<R>,
        elf_header: &ElfHeader,
    ) -> Result<Vec<SectionHeader>> {
        let mut headers = vec![];
        buffer.seek(SeekFrom::Start(elf_header.shoff as u64))?;
        for _ in 0..elf_header.shnum {
            let mut header: SectionHeader = unsafe { std::mem::zeroed() };
            unsafe {
                let header_slice = std::slice::from_raw_parts_mut(
                    &mut header as *mut _ as *mut u8,
                    elf_header.shentsize as usize,
                );
                buffer.read_exact(header_slice)?;
                headers.push(header);
            }
        }

        Ok(headers)
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut elf = Elf::load_buffer(&mut reader)?;
    let prog_bytes = elf.read_program_bytes(0);
    println!("{elf:#x?}");
    println!("{prog_bytes:#x?}");
    Ok(())
}
