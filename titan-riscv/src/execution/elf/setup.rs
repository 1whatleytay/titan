use crate::assembler::registers::RegisterSlot;
use crate::cpu::State;
use crate::cpu::memory::Mountable;
use crate::cpu::memory::Region;
use crate::cpu::memory::section::{ListenResponder, SectionMemory};
use crate::cpu::registers::registers::RawRegisters;
use crate::elf::Elf;
use num_traits::ToPrimitive;
use titan_shared::cpu::memory::section::AlignmentBehaviour;

pub const SMALL_HEAP_SIZE: u32 = 0x10000u32;

pub fn create_simple_state<T: ListenResponder>(
    elf: &Elf,
    heap_size: u32,
) -> State<SectionMemory<T>, RawRegisters> {
    let mut memory = SectionMemory::new_with_align(AlignmentBehaviour::SlowReadOnAlign);

    for header in &elf.program_headers {
        let region = Region {
            start: header.virtual_address,
            data: header.data.clone(),
        };

        memory.mount(region)
    }

    let heap_end = 0x7FFFFFFCu32;

    let heap = Region {
        start: heap_end - heap_size,
        data: vec![0; heap_size as usize],
    };

    memory.mount(heap);

    let registers = RawRegisters {
        pc: elf.header.program_entry,
        ..Default::default()
    };

    let mut state = State::new(registers, memory);
    state.registers.line[RegisterSlot::StackPointer.to_usize().unwrap()] = heap_end;

    state
}
