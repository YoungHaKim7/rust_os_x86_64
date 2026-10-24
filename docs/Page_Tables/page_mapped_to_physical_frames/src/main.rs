use std::collections::HashMap;

const PAGE_SIZE: u64 = 4096;
const PAGE_MASK: u64 = PAGE_SIZE - 1;

/// A 4 KiB virtual memory page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Page {
    start: u64,
}

impl Page {
    fn containing(address: u64) -> Self {
        Self {
            start: address & !PAGE_MASK,
        }
    }

    fn start(self) -> u64 {
        self.start
    }
}

/// A 4 KiB physical memory frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhysFrame {
    start: u64,
}

/// A simple demo allocator that hands out physical frames.
struct FrameAllocator {
    next: u64,
}

impl FrameAllocator {
    fn new(start: u64) -> Self {
        assert_eq!(start & PAGE_MASK, 0, "Frame address must be aligned");
        Self { next: start }
    }

    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = PhysFrame { start: self.next };
        self.next = self.next.checked_add(PAGE_SIZE)?;
        Some(frame)
    }
}

#[derive(Debug, Clone, Copy)]
struct Mapping {
    frame: PhysFrame,
    writable: bool,
}

/// A software simulation of a page-table mapper.
struct Mapper {
    mappings: HashMap<u64, Mapping>,
}

impl Mapper {
    fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    fn map(&mut self, page: Page, frame: PhysFrame, writable: bool) -> Result<(), &'static str> {
        if self.mappings.contains_key(&page.start()) {
            return Err("Virtual page is already mapped");
        }

        self.mappings
            .insert(page.start(), Mapping { frame, writable });

        Ok(())
    }

    fn translate(&self, virtual_address: u64) -> Option<u64> {
        let page = Page::containing(virtual_address);
        let mapping = self.mappings.get(&page.start())?;
        let offset = virtual_address & PAGE_MASK;

        Some(mapping.frame.start + offset)
    }

    fn print_mapping(&self, page: Page) {
        match self.mappings.get(&page.start()) {
            Some(mapping) => {
                println!(
                    "Virtual page {:#x} -> physical frame {:#x}, writable={}",
                    page.start(),
                    mapping.frame.start,
                    mapping.writable,
                );
            }
            None => println!("Virtual page {:#x} is unmapped", page.start()),
        }
    }
}

fn main() -> Result<(), &'static str> {
    // Demo physical RAM starts at 0x100000.
    let mut allocator = FrameAllocator::new(0x100000);
    let mut mapper = Mapper::new();

    // Map the virtual page containing address 0x4000.
    let page = Page::containing(0x4000);

    // Obtain a physical frame.
    let frame = allocator
        .allocate_frame()
        .ok_or("Physical frame allocation failed")?;

    // Install the simulated mapping.
    mapper.map(page, frame, true)?;

    // Inspect the mapping.
    mapper.print_mapping(page);

    let virtual_address = 0x4234;
    let physical_address = mapper
        .translate(virtual_address)
        .ok_or("Virtual address is not mapped")?;

    println!("Virtual address:  {virtual_address:#x}");
    println!("Physical address: {physical_address:#x}");

    Ok(())
}
