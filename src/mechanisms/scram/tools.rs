use crate::gsasl::consts::GSASL_SCRAM_SALTED_PASSWORD;
use crate::gsasl::crypto::gsasl_hash_length;
use crate::gsasl::mechtools::{Gsasl_hash, _gsasl_hex_encode};
use crate::gsasl::property::gsasl_property_set;
use crate::session::SessionData;
use ::libc;
use digest::generic_array::{ArrayLength, GenericArray};
use digest::Mac;

pub fn hash_password<PRF>(password: &str, iterations: u32, salt: &[u8], out: &mut [u8])
where
    PRF: digest::Update + digest::FixedOutput + digest::KeyInit + Clone + Sync,
{
    pbkdf2::pbkdf2::<PRF>(password.as_bytes(), salt, iterations, out);
}

pub fn find_proofs<D, HMAC, HL>(
    username: &str,
    client_nonce: &[u8],
    server_first: &[u8],
    gs2headerb64: &str,
    combined_nonce: &[u8],
    salted_password: &[u8],
) -> (GenericArray<u8, HL>, GenericArray<u8, HL>)
where
    D: digest::Digest,
    HMAC: digest::Mac
        + digest::KeyInit
        + digest::OutputSizeUser
        + digest::OutputSizeUser<OutputSize = HL>,
    HL: ArrayLength<u8>,
{
    let mut salted_password_hmac =
        <HMAC as Mac>::new_from_slice(salted_password).expect("HMAC can work with any key size");
    salted_password_hmac.update(b"Client Key");
    let mut client_key = salted_password_hmac.finalize().into_bytes();

    let mut salted_password_hmac =
        <HMAC as Mac>::new_from_slice(salted_password).expect("HMAC can work with any key size");
    salted_password_hmac.update(b"Server Key");
    let server_key = salted_password_hmac.finalize().into_bytes();

    let stored_key = D::digest(client_key.as_ref());

    let auth_message_parts: [&[u8]; 10] = [
        b"n=",
        username.as_bytes(),
        b",r=",
        client_nonce,
        b",",
        server_first,
        b",c=",
        gs2headerb64.as_bytes(),
        b",r=",
        combined_nonce,
    ];
    let _o: Vec<u8> = auth_message_parts
        .iter()
        .map(|s| s.iter())
        .flatten()
        .map(|b| *b)
        .collect();

    let mut stored_key_hmac = <HMAC as Mac>::new_from_slice(stored_key.as_ref())
        .expect("HMAC can work with any key size");
    for part in auth_message_parts {
        stored_key_hmac.update(part);
    }
    let client_signature = stored_key_hmac.finalize().into_bytes();

    // Client Key => Client Proof
    {
        let client_key_mut = client_key.as_mut();
        client_key_mut
            .iter_mut()
            .zip(client_signature.iter())
            .for_each(|(a, b)| *a ^= b);
    }

    let mut server_key_hmac = <HMAC as Mac>::new_from_slice(server_key.as_ref())
        .expect("HMAC can work with any key size");
    for part in auth_message_parts {
        server_key_hmac.update(part);
    }
    let server_signature = server_key_hmac.finalize().into_bytes();

    (client_key, server_signature)
}

/* Hex encode HASHBUF which is HASH digest output and set salted
password property to the hex encoded value. */
pub unsafe fn set_saltedpassword(
    sctx: &mut SessionData,
    hash: Gsasl_hash,
    hashbuf: *const libc::c_char,
) -> libc::c_int {
    let mut hexstr: [libc::c_char; 65] = [0; 65];
    _gsasl_hex_encode(hashbuf, gsasl_hash_length(hash), hexstr.as_mut_ptr());
    return gsasl_property_set(sctx, GSASL_SCRAM_SALTED_PASSWORD, hexstr.as_mut_ptr());
}
