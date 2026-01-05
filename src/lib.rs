use pinocchio::log::sol_log_data;
use core::mem::MaybeUninit;

#[repr(C)]
struct U128MulParams {
    a: u128,
    b: u128,
    result: MaybeUninit<u128>,
}

// JIT intrinsic for u128 multiplication
// Pass a pointer to a 48-byte buffer: [a: u128, b: u128, result: u128]
// The intrinsic reads a and b, computes a*b, and writes to result
#[inline(always)]
pub fn sol_u128_mul(a: u128, b: u128) -> u128 {
    let syscall: unsafe extern "C" fn(*mut U128MulParams) -> u64 =
        unsafe { core::mem::transmute(0x2b1afaff_usize) };

    let mut params = U128MulParams {
        a,
        b,
        result: MaybeUninit::uninit(),
    };

    unsafe {
        syscall(&mut params);
        params.result.assume_init()
    }
}

#[no_mangle]
unsafe extern "C" fn entrypoint(_ptr: *mut u8) -> u64 {
    let x = *{ _ptr.add(16) as *const u128 };
    let y = sol_u128_mul(1000, x);
    sol_log_data(&[&y.to_le_bytes()]);
    if y % 4 != 0 {
        return 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use mollusk_svm::Mollusk;

    const PROGRAM_ID: [u8; 32] = [0x02; 32];

    #[test]
    fn test() {
        let mollusk = Mollusk::new(&PROGRAM_ID.into(), "target/deploy/u128_intrinsics");

        let instruction = solana_instruction::Instruction {
            program_id: PROGRAM_ID.into(),
            accounts: vec![],
            data: 1000u128.to_le_bytes().to_vec(),
        };

        mollusk.process_instruction(&instruction, &[]);
    }
}
