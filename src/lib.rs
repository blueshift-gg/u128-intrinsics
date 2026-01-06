use pinocchio::program_error::ProgramError;
#[cfg(feature = "log")]
use pinocchio::log::sol_log_data;

use core::mem::MaybeUninit;
use std::u128;

#[repr(C)]
struct U128MulParams {
    a: u128,
    b: u128,
    result_lo: MaybeUninit<u128>,
    result_hi: MaybeUninit<u128>,
}

// JIT intrinsic for u128 multiplication
// Pass a pointer to a 48-byte buffer: [a: u128, b: u128, result: u128]
// The intrinsic reads a and b, computes a*b, and writes to result
#[inline(always)]
pub fn sol_u128_mul(a: u128, b: u128) -> (u128, u128) {
    let sol_u128_mul: unsafe extern "C" fn(*mut U128MulParams) -> u64 =
        unsafe { core::mem::transmute(0x2b1afaff_usize) };

    let mut params = U128MulParams {
        a,
        b,
        result_lo: MaybeUninit::uninit(),
        result_hi: MaybeUninit::uninit(),
    };

    unsafe {
        sol_u128_mul(&mut params);
        (params.result_lo.assume_init(), params.result_hi.assume_init())
    }
}

#[no_mangle]
unsafe extern "C" fn entrypoint(ptr: *mut u8) -> u64 {
    let y = sol_u128_mul(u128::MAX, unsafe { *(ptr.add(16) as *const u128) } );
    #[cfg(feature = "log")]
    sol_log_data(&[&y.0.to_le_bytes(), &y.1.to_le_bytes()]);
    if y.1 != 0 {
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
        let mut mollusk = Mollusk::new(&PROGRAM_ID.into(), "target/deploy/u128_intrinsics");

        // Increase compute unit limit for u128 intrinsic testing (address translation is expensive)
        mollusk.compute_budget.heap_size = 256 * 1024;
        mollusk.compute_budget.heap_cost = 8;
        mollusk.compute_budget.compute_unit_limit = 10_000_000; // 10M CUs

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

        mollusk.process_and_validate_instruction(&instruction, &[], &[Check::err(solana_program_error::ProgramError::ArithmeticOverflow)]);
    }
}
