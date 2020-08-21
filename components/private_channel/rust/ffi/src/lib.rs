use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::slice;

pub const KEY_SIZE: usize = 32;

macro_rules! assert_not_null {
    ($var:expr) => {
        assert!(!$var.is_null(), "{} is NULL", stringify!($var));
    };
}

#[repr(C)]
pub struct ResultChallenge {
    pub pkeys_ptr: *const u8,
    pub pkeys_byte_size: usize,
    pub skeys_ptr: *const u8,
    pub skeys_byte_size: usize,
    pub shared_pubkey_ptr: *const u8,
    pub shared_pkeys_byte_size: usize,
    pub encrypted_hashes_ptr: *const u8,
    pub encrypted_hashes_size: usize,
    pub key_size: usize,
    pub error: bool,
}

impl Default for ResultChallenge {
    fn default() -> ResultChallenge {
        let mock_vec = vec![];
        ResultChallenge {
            error: true,
            pkeys_ptr: mock_vec.as_ptr(),
            pkeys_byte_size: 0,
            skeys_ptr: mock_vec.as_ptr(),
            skeys_byte_size: 0,
            shared_pkeys_byte_size: 0,
            shared_pubkey_ptr: mock_vec.as_ptr(),
            encrypted_hashes_size: 0,
            encrypted_hashes_ptr: mock_vec.as_ptr(),
            key_size: 0,
        }
    }
}

#[repr(C)]
pub struct ResultSecondRound {
    pub encoded_partial_dec_ptr: *const u8,
    pub encoded_partial_dec_size: usize,
    pub encoded_proofs_ptr: *const u8,
    pub encoded_proofs_size: usize,
    pub random_vec_ptr: *const u8,
    pub random_vec_size: usize,
    pub error: bool,
}

impl Default for ResultSecondRound {
    fn default() -> ResultSecondRound {
        let mock_vec = vec![];
        ResultSecondRound {
            error: true,
            encoded_partial_dec_ptr: mock_vec.as_ptr(),
            encoded_partial_dec_size: 0,
            encoded_proofs_ptr: mock_vec.as_ptr(),
            encoded_proofs_size: 0,
            random_vec_ptr: mock_vec.as_ptr(),
            random_vec_size: 0,
        }
    }
}

/// Starts client attestation challenge;
#[no_mangle]
pub unsafe extern "C" fn client_start_challenge(
    input: *const *const c_char,
    input_size: c_int,
    server_pk_encoded: *const c_char,
) -> ResultChallenge {
    assert!(!input.is_null(), "Null pointers passed as input");
    assert!(
        !server_pk_encoded.is_null(),
        "Null pointers passed as input"
    );

    let server_pk = match CStr::from_ptr(server_pk_encoded).to_str() {
        Ok(pk) => pk.to_string(),
        Err(_) => return ResultChallenge::default(),
    };

    let mut v_out = Vec::new();
    let input_array = slice::from_raw_parts(input, input_size as usize);
    for n in 0..input_size {
        let s_ptr = CStr::from_ptr(input_array[n as usize]);
        v_out.push(s_ptr.to_str().unwrap().to_string());
    }

    let brave_private_channel::FirstRoundOutput {
        pkeys,
        skeys,
        shared_pks,
        enc_hashes,
    } = match brave_private_channel::start_challenge(v_out, server_pk) {
        Ok(result) => result,
        Err(_) => return ResultChallenge::default(),
    };

    let pkeys_buff = pkeys.into_boxed_slice();
    let pkeys_buff = std::mem::ManuallyDrop::new(pkeys_buff);

    let skeys_buff = skeys.into_boxed_slice();
    let skeys_buff = std::mem::ManuallyDrop::new(skeys_buff);

    let shared_pks_buff = shared_pks.into_boxed_slice();
    let shared_pks_buff = std::mem::ManuallyDrop::new(shared_pks_buff);

    let enc_hashes_buff = enc_hashes.into_boxed_slice();
    let enc_hashes_buff = std::mem::ManuallyDrop::new(enc_hashes_buff);

    ResultChallenge {
        pkeys_ptr: pkeys_buff.as_ptr(),
        pkeys_byte_size: pkeys_buff.len(),
        skeys_ptr: skeys_buff.as_ptr(),
        skeys_byte_size: skeys_buff.len(),
        shared_pubkey_ptr: shared_pks_buff.as_ptr(),
        shared_pkeys_byte_size: shared_pks_buff.len(),
        encrypted_hashes_ptr: enc_hashes_buff.as_ptr(),
        encrypted_hashes_size: enc_hashes_buff.len(),
        key_size: KEY_SIZE,
        error: false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn client_second_round(
    input: *const c_char,
    _input_size: c_int,
    client_sk_encoded: *const c_char,
) -> ResultSecondRound {
    assert!(!input.is_null(), "Null pointers passed as input");
    assert!(
        !client_sk_encoded.is_null(),
        "Null pointers passed as input"
    );

    let client_sk = match CStr::from_ptr(client_sk_encoded).to_str() {
        Ok(sk) => sk.to_string(),
        Err(_) => return ResultSecondRound::default(),
    };

    let raw_str = match CStr::from_ptr(input).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ResultSecondRound::default(),
    };
    let parsed_str = raw_str.replace(&['[', ']'][..], "");
    let v_enc: Vec<u8> = parsed_str
        .split(", ")
        .map(|s| s.parse::<u8>().unwrap())
        .collect();

    let brave_private_channel::SecondRoundOutput {
        partial_dec,
        proofs,
        rand_vec,
    } = match brave_private_channel::second_round(&v_enc, client_sk) {
        Ok(result) => result,
        Err(_) => return ResultSecondRound::default(),
    };

    let partial_enc_buff = partial_dec.into_boxed_slice();
    let partial_enc_buff = std::mem::ManuallyDrop::new(partial_enc_buff);

    let proofs_buff = proofs.into_boxed_slice();
    let proofs_buff = std::mem::ManuallyDrop::new(proofs_buff);

    let rand_vec_buff = rand_vec.into_boxed_slice();
    let rand_vec_buff = std::mem::ManuallyDrop::new(rand_vec_buff);

    ResultSecondRound {
        encoded_partial_dec_ptr: partial_enc_buff.as_ptr(),
        encoded_partial_dec_size: partial_enc_buff.len(),
        encoded_proofs_ptr: proofs_buff.as_ptr(),
        encoded_proofs_size: proofs_buff.len(),
        random_vec_ptr: rand_vec_buff.as_ptr(),
        random_vec_size: rand_vec_buff.len(),
        error: false,
    }
}

// By reconstructing the fileds of the structure in Rust and letting it out of scope,
// the Rust compiler will deallocate the memory contents
#[no_mangle]
pub unsafe extern "C" fn deallocate_first_round_result(result: ResultChallenge) {
    assert_not_null!(result.pkeys_ptr);
    let _pkeys = Box::from_raw(std::slice::from_raw_parts_mut(
        result.pkeys_ptr as *mut u8,
        result.pkeys_byte_size,
    ))
    .into_vec();

    assert_not_null!(result.skeys_ptr);
    let _skeys = Box::from_raw(std::slice::from_raw_parts_mut(
        result.skeys_ptr as *mut u8,
        result.skeys_byte_size,
    ))
    .into_vec();

    assert_not_null!(result.shared_pubkey_ptr);
    let _shared_key = Box::from_raw(std::slice::from_raw_parts_mut(
        result.shared_pubkey_ptr as *mut u8,
        KEY_SIZE,
    ))
    .into_vec();

    assert_not_null!(result.encrypted_hashes_ptr);
    let _shared_key = Box::from_raw(std::slice::from_raw_parts_mut(
        result.encrypted_hashes_ptr as *mut u8,
        result.encrypted_hashes_size,
    ))
    .into_vec();
}

#[no_mangle]
pub unsafe extern "C" fn deallocate_second_round_result(result: ResultSecondRound) {
    assert_not_null!(result.encoded_partial_dec_ptr);
    let _partial_dec = Box::from_raw(std::slice::from_raw_parts_mut(
        result.encoded_partial_dec_ptr as *mut u8,
        result.encoded_partial_dec_size,
    ))
    .into_vec();

    assert_not_null!(result.encoded_proofs_ptr);
    let _enc_proofs = Box::from_raw(std::slice::from_raw_parts_mut(
        result.encoded_proofs_ptr as *mut u8,
        result.encoded_proofs_size,
    ))
    .into_vec();

    assert_not_null!(result.random_vec_ptr);
    let _rand_vec = Box::from_raw(std::slice::from_raw_parts_mut(
        result.random_vec_ptr as *mut u8,
        result.random_vec_size,
    ))
    .into_vec();
}
