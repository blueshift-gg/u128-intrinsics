#![cfg_attr(any(target_arch = "bpf", target_os = "solana"), no_std)]

#[cfg(any(target_arch = "bpf", target_os = "solana"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

#[cfg(feature = "log")]
extern "C" {
    fn sol_log_data(data: *const u8, data_len: u64);
}

#[cfg(feature = "log")]
fn log(data: &[&[u8]]) {
    unsafe {
        sol_log_data(data as *const _ as *const u8, data.len() as u64);
    }
}

use core::mem::MaybeUninit;

use solana_program_error::ProgramError;

// JIT intrinsic for multi3 support
// This intrinsic computes a*b, and writes to result
#[inline(always)]
pub fn sol_multi3(result: *mut u128, a: u128, b: u128) {
    let sol_multi3: unsafe extern "C" fn(result: *mut u128, a: u128, b: u128) -> u64 = unsafe { core::mem::transmute(0xDB0F6D13usize) };
    unsafe {
        sol_multi3(result, a, b);
    }
}

#[no_mangle]
unsafe extern "C" fn entrypoint(ptr: *mut u8) -> u64 {
    let mut result = MaybeUninit::<u128>::uninit();
    sol_multi3(result.as_mut_ptr(), u128::MAX, unsafe { *(ptr.add(0x0010) as *const u128) } );
    #[cfg(feature = "log")]
    log(&[&result.assume_init().to_le_bytes()]);
    if unsafe { result.assume_init() } == 0 {
        return ProgramError::ArithmeticOverflow.into();
    }
    0
}

#[cfg(test)]
mod tests {
    use mollusk_svm::{Mollusk, result::Check};

    const PROGRAM_ID: [u8; 32] = [0x02; 32];

    #[test]
    fn test() {
        let mollusk = Mollusk::new(&PROGRAM_ID.into(), "target/deploy/u128_intrinsics");

        let instruction = solana_instruction::Instruction {
            program_id: PROGRAM_ID.into(),
            accounts: vec![],
            data: 1u128.to_le_bytes().to_vec(),
        };

        mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
    }

    #[test]
    fn test_overflow() {
        let mollusk = Mollusk::new(&PROGRAM_ID.into(), "target/deploy/u128_intrinsics");

        let instruction = solana_instruction::Instruction {
            program_id: PROGRAM_ID.into(),
            accounts: vec![],
            data: u128::MAX.to_le_bytes().to_vec(),
        };

        mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
    }
}
