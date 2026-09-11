#[cfg(feature = "profile")]
use crate::bytecode::chunk::OpCode;

use super::VirtualMachine;

#[cfg(feature = "profile")]
impl VirtualMachine {
    pub(crate) fn profile_instruction(&mut self, instruction: u8) {
        self.profile_counts[instruction as usize] += 1;
    }

    pub(crate) fn profile_read_byte(&mut self) {
        self.profile_read_bytes += 1;
    }

    pub(crate) fn print_profile(&self) {
        let mut entries = self
            .profile_counts
            .iter()
            .enumerate()
            .filter_map(|(instruction, &count)| {
                if count == 0 {
                    return None;
                }

                let name = OpCode::try_from(instruction as u8)
                    .map(|opcode| format!("{opcode:?}"))
                    .unwrap_or_else(|_| format!("0x{instruction:02X}"));

                Some((name, count))
            })
            .collect::<Vec<_>>();

        entries.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        let total: u64 = entries.iter().map(|(_, count)| *count).sum();

        eprintln!();
        eprintln!("========== KASTEL VM PROFILE ==========");
        eprintln!("instructions: {total}");
        eprintln!("read_byte calls: {}", self.profile_read_bytes);

        for (name, count) in entries {
            eprintln!("{count:>12}  {name}");
        }

        eprintln!("========================================");
    }
}

#[cfg(not(feature = "profile"))]
impl VirtualMachine {
    pub(crate) fn profile_instruction(&mut self, _instruction: u8) {}
    #[allow(dead_code)]
    pub(crate) fn profile_read_byte(&mut self) {}

    pub(crate) fn print_profile(&self) {}
}
