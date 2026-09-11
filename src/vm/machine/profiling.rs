#[cfg(feature = "profile")]
use crate::bytecode::chunk::OpCode;

use super::VirtualMachine;

#[cfg(feature = "profile")]
impl VirtualMachine {
    #[inline(always)]
    pub(crate) fn profile_instruction(&mut self, instruction: u8) {
        self.profile_counts[instruction as usize] += 1;
    }

    #[inline(always)]
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

                let opcode = OpCode::try_from(instruction as u8).ok()?;

                let sample_count = count.div_ceil(4096);

                let sampled_time = self.profile_times[instruction];

                let estimated_time = if sample_count > 0 {
                    sampled_time.mul_f64(4096.0)
                } else {
                    std::time::Duration::ZERO
                };

                let average_ns = if sample_count > 0 {
                    sampled_time.as_nanos() as f64 / sample_count as f64
                } else {
                    0.0
                };

                Some((format!("{opcode:?}"), count, estimated_time, average_ns))
            })
            .collect::<Vec<_>>();

        entries.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        let total: u64 = entries.iter().map(|entry| entry.1).sum();

        eprintln!();
        eprintln!("========== KASTEL VM PROFILE ==========");
        eprintln!("instructions: {total}");
        eprintln!("read_byte calls: {}", self.profile_read_bytes);

        if total != 0 {
            eprintln!(
                "read_byte / instruction: {:.3}",
                self.profile_read_bytes as f64 / total as f64
            );
        }

        eprintln!();
        eprintln!(
            "{:<20} {:>12} {:>14} {:>14}",
            "Opcode", "count", "est. time", "sample ns"
        );

        eprintln!("{:-<20} {:-<12} {:-<14} {:-<14}", "", "", "", "");

        for (name, count, estimated_time, average_ns) in entries {
            eprintln!(
                "{:<20} {:>12} {:>14?} {:>12.1} ns",
                name, count, estimated_time, average_ns
            );
        }

        eprintln!("========================================");
    }
}

#[cfg(not(feature = "profile"))]
#[allow(dead_code)]
impl VirtualMachine {
    #[inline(always)]
    pub(crate) fn profile_instruction(&mut self, _instruction: u8) {}

    #[inline(always)]
    pub(crate) fn profile_read_byte(&mut self) {}

    pub(crate) fn print_profile(&self) {}
}
