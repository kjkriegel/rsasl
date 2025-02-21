use crate::gsasl::consts::{
    GSASL_AUTHID, GSASL_AUTHZID, GSASL_MALLOC_ERROR, GSASL_MECHANISM_CALLED_TOO_MANY_TIMES,
    GSASL_NO_AUTHID, GSASL_NO_PASSCODE, GSASL_NO_PIN, GSASL_OK, GSASL_PASSCODE, GSASL_PIN,
    GSASL_SUGGESTED_PIN,
};
use crate::gsasl::gl::free::rpl_free;
use crate::gsasl::property::{gsasl_property_get, gsasl_property_set_raw};
use crate::session::SessionData;
use crate::Shared;
use ::libc;
use libc::{malloc, memcmp, memcpy, size_t, strlen};
use std::ptr::NonNull;

pub(crate) unsafe fn _gsasl_securid_client_start(
    mut _sctx: &Shared,
    mech_data: &mut Option<NonNull<()>>,
) -> libc::c_int {
    let step;
    step = malloc(::std::mem::size_of::<libc::c_int>()) as *mut libc::c_int;
    if step.is_null() {
        return GSASL_MALLOC_ERROR as libc::c_int;
    }
    *step = 0 as libc::c_int;
    *mech_data = NonNull::new(step as *mut ());
    return GSASL_OK as libc::c_int;
}

pub unsafe fn _gsasl_securid_client_step(
    sctx: &mut SessionData,
    mech_data: Option<NonNull<()>>,
    input: Option<&[u8]>,
    output: *mut *mut libc::c_char,
    output_len: *mut size_t,
) -> libc::c_int {
    let mech_data = mech_data
        .map(|ptr| ptr.as_ptr())
        .unwrap_or_else(std::ptr::null_mut);

    let input_len = input.map(|i| i.len()).unwrap_or(0);
    let input: *const libc::c_char = input.map(|i| i.as_ptr().cast()).unwrap_or(std::ptr::null());

    let step: *mut libc::c_int = mech_data as *mut libc::c_int;
    let authzid;
    let authid;
    let passcode;
    let mut pin: *const libc::c_char = 0 as *const libc::c_char;
    let authzidlen;
    let authidlen;
    let passcodelen;
    let mut pinlen: size_t = 0 as libc::c_int as size_t;
    let mut do_pin: libc::c_int = 0 as libc::c_int;
    let mut res: libc::c_int = 0;
    let current_block_53: u64;
    match *step {
        1 => {
            if input_len == strlen(b"passcode\x00" as *const u8 as *const libc::c_char)
                && memcmp(
                    input as *const libc::c_void,
                    b"passcode\x00" as *const u8 as *const libc::c_char as *const libc::c_void,
                    strlen(b"passcode\x00" as *const u8 as *const libc::c_char),
                ) == 0 as libc::c_int
            {
                *step = 0 as libc::c_int;
                current_block_53 = 7859779714627992552;
            } else if input_len >= strlen(b"pin\x00" as *const u8 as *const libc::c_char)
                && memcmp(
                    input as *const libc::c_void,
                    b"pin\x00" as *const u8 as *const libc::c_char as *const libc::c_void,
                    strlen(b"pin\x00" as *const u8 as *const libc::c_char),
                ) == 0 as libc::c_int
            {
                do_pin = 1 as libc::c_int;
                *step = 0 as libc::c_int;
                current_block_53 = 7859779714627992552;
            } else {
                *output_len = 0 as libc::c_int as size_t;
                res = GSASL_OK as libc::c_int;
                current_block_53 = 10930818133215224067;
            }
        }
        0 => {
            current_block_53 = 7859779714627992552;
        }
        2 => {
            *output_len = 0 as libc::c_int as size_t;
            *output = 0 as *mut libc::c_char;
            *step += 1;
            res = GSASL_OK as libc::c_int;
            current_block_53 = 10930818133215224067;
        }
        _ => {
            res = GSASL_MECHANISM_CALLED_TOO_MANY_TIMES as libc::c_int;
            current_block_53 = 10930818133215224067;
        }
    }
    match current_block_53 {
        7859779714627992552 =>
        /* fall through */
        {
            authzid = gsasl_property_get(sctx, GSASL_AUTHZID);
            if !authzid.is_null() {
                authzidlen = strlen(authzid)
            } else {
                authzidlen = 0 as libc::c_int as size_t
            }
            authid = gsasl_property_get(sctx, GSASL_AUTHID);
            if authid.is_null() {
                return GSASL_NO_AUTHID as libc::c_int;
            }
            authidlen = strlen(authid);
            passcode = gsasl_property_get(sctx, GSASL_PASSCODE);
            if passcode.is_null() {
                return GSASL_NO_PASSCODE as libc::c_int;
            }
            passcodelen = strlen(passcode);
            if do_pin != 0 {
                if input_len > strlen(b"pin\x00" as *const u8 as *const libc::c_char) {
                    res =
                        gsasl_property_set_raw(
                            sctx,
                            GSASL_SUGGESTED_PIN,
                            &*input.offset(
                                strlen(b"pin\x00" as *const u8 as *const libc::c_char) as isize
                            ),
                            input_len.wrapping_sub(strlen(
                                b"pin\x00" as *const u8 as *const libc::c_char,
                            )),
                        );
                    if res != GSASL_OK as libc::c_int {
                        return res;
                    }
                }
                pin = gsasl_property_get(sctx, GSASL_PIN);
                if pin.is_null() {
                    return GSASL_NO_PIN as libc::c_int;
                }
                pinlen = strlen(pin)
            }
            *output_len = authzidlen
                .wrapping_add(1)
                .wrapping_add(authidlen)
                .wrapping_add(1)
                .wrapping_add(passcodelen)
                .wrapping_add(1);
            if do_pin != 0 {
                *output_len = *output_len.wrapping_add(pinlen.wrapping_add(1))
            }
            *output = malloc(*output_len) as *mut libc::c_char;
            if (*output).is_null() {
                return GSASL_MALLOC_ERROR as libc::c_int;
            }
            if !authzid.is_null() {
                memcpy(
                    *output as *mut libc::c_void,
                    authzid as *const libc::c_void,
                    authzidlen,
                );
            }
            *(*output).offset(authzidlen as isize) = '\u{0}' as i32 as libc::c_char;
            memcpy(
                (*output)
                    .offset(authzidlen as isize)
                    .offset(1 as libc::c_int as isize) as *mut libc::c_void,
                authid as *const libc::c_void,
                authidlen,
            );
            *(*output).offset(authzidlen.wrapping_add(1).wrapping_add(authidlen) as isize) =
                '\u{0}' as i32 as libc::c_char;
            memcpy(
                (*output)
                    .offset(authzidlen as isize)
                    .offset(1)
                    .offset(authidlen as isize)
                    .offset(1) as *mut libc::c_void,
                passcode as *const libc::c_void,
                passcodelen,
            );
            *(*output).offset(
                authzidlen
                    .wrapping_add(1)
                    .wrapping_add(authidlen)
                    .wrapping_add(1)
                    .wrapping_add(passcodelen) as isize,
            ) = '\u{0}' as i32 as libc::c_char;
            if do_pin != 0 {
                memcpy(
                    (*output)
                        .offset(authzidlen as isize)
                        .offset(1)
                        .offset(authidlen as isize)
                        .offset(1)
                        .offset(passcodelen as isize)
                        .offset(1) as *mut libc::c_void,
                    pin as *const libc::c_void,
                    pinlen,
                );
                *(*output).offset(
                    authzidlen
                        .wrapping_add(1)
                        .wrapping_add(authidlen)
                        .wrapping_add(1)
                        .wrapping_add(passcodelen)
                        .wrapping_add(1)
                        .wrapping_add(pinlen) as isize,
                ) = '\u{0}' as i32 as libc::c_char
            }
            *step += 1;
            res = GSASL_OK as libc::c_int
        }
        _ => {}
    }
    return res;
}
/* securid.h --- Prototypes for SASL mechanism SECURID as defined in RFC 2808.
 * Copyright (C) 2002-2021 Simon Josefsson
 *
 * This file is part of GNU SASL Library.
 *
 * GNU SASL Library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public License
 * as published by the Free Software Foundation; either version 2.1 of
 * the License, or (at your option) any later version.
 *
 * GNU SASL Library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with GNU SASL Library; if not, write to the Free
 * Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */
pub unsafe fn _gsasl_securid_client_finish(mech_data: Option<NonNull<()>>) {
    let mech_data = mech_data
        .map(|ptr| ptr.as_ptr())
        .unwrap_or_else(std::ptr::null_mut);

    let step: *mut libc::c_int = mech_data as *mut libc::c_int;
    rpl_free(step as *mut libc::c_void);
}
