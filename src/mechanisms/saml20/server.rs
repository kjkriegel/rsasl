use std::ptr::NonNull;
use ::libc;
use libc::size_t;
use crate::gsasl::callback::gsasl_callback;
use crate::gsasl::consts::{GSASL_AUTHZID, GSASL_MALLOC_ERROR, GSASL_MECHANISM_CALLED_TOO_MANY_TIMES, GSASL_MECHANISM_PARSE_ERROR, GSASL_NEEDS_MORE, GSASL_NO_SAML20_REDIRECT_URL, GSASL_OK, GSASL_SAML20_IDP_IDENTIFIER, GSASL_SAML20_REDIRECT_URL, GSASL_VALIDATE_SAML20};
use crate::gsasl::property::{gsasl_property_get, gsasl_property_set, gsasl_property_set_raw};
use crate::session::SessionData;
use crate::Shared;

extern "C" {
    fn memcpy(_: *mut libc::c_void, _: *const libc::c_void, _: libc::c_ulong)
     -> *mut libc::c_void;
    fn rpl_free(_: *mut libc::c_void);
    fn strlen(_: *const libc::c_char) -> size_t;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn calloc(_: libc::c_ulong, _: libc::c_ulong) -> *mut libc::c_void;
    /* mechtools.h --- Helper functions available for use by any mechanism.
 * Copyright (C) 2010-2021 Simon Josefsson
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
    /* Get size_t. */
    /* Get bool. */
    fn _gsasl_parse_gs2_header(data: *const libc::c_char, len: size_t,
                               authzid: *mut *mut libc::c_char,
                               headerlen: *mut size_t) -> libc::c_int;
}
/* server.c --- SAML20 mechanism, server side.
 * Copyright (C) 2010-2021 Simon Josefsson
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
/* Get specification. */
/* Get strdup, strlen. */
/* Get free. */
/* Get _gsasl_parse_gs2_header. */
#[derive(Copy, Clone)]
#[repr(C)]
pub struct saml20_server_state {
    pub step: libc::c_int,
}

pub(crate) unsafe fn _gsasl_saml20_server_start(_sctx: &Shared,
                                         mech_data: &mut Option<NonNull<()>>,
) -> libc::c_int
{
    let mut state: *mut saml20_server_state = 0 as *mut saml20_server_state;
    state =
        calloc(::std::mem::size_of::<saml20_server_state>() as libc::c_ulong,
               1 as libc::c_int as libc::c_ulong) as *mut saml20_server_state;
    if state.is_null() { return GSASL_MALLOC_ERROR as libc::c_int }
    *mech_data = NonNull::new(state as *mut ());
    return GSASL_OK as libc::c_int;
}

pub unsafe fn _gsasl_saml20_server_step(mut sctx: &mut SessionData,
                                        mut mech_data: Option<NonNull<()>>,
                                        mut input: Option<&[u8]>,
                                        mut output: *mut *mut libc::c_char,
                                        mut output_len: *mut size_t
) -> libc::c_int
{
    let mech_data = mech_data
        .map(|ptr| ptr.as_ptr())
        .unwrap_or_else(std::ptr::null_mut);

    let mut input_len = input.map(|i| i.len()).unwrap_or(0);
    let mut input: *const libc::c_char =
        input.map(|i| i.as_ptr().cast()).unwrap_or(std::ptr::null());

    let mut state: *mut saml20_server_state =
        mech_data as *mut saml20_server_state;
    let mut res: libc::c_int =
        GSASL_MECHANISM_CALLED_TOO_MANY_TIMES as libc::c_int;
    *output_len = 0 as libc::c_int as size_t;
    *output = 0 as *mut libc::c_char;
    match (*state).step {
        0 => {
            let mut p: *const libc::c_char = 0 as *const libc::c_char;
            let mut authzid: *mut libc::c_char = 0 as *mut libc::c_char;
            let mut headerlen: size_t = 0;
            if input_len == 0 {
                return GSASL_NEEDS_MORE as libc::c_int
            }
            res =
                _gsasl_parse_gs2_header(input, input_len, &mut authzid,
                                        &mut headerlen);
            if res != GSASL_OK as libc::c_int { return res }
            if !authzid.is_null() {
                res = gsasl_property_set(sctx, GSASL_AUTHZID, authzid);
                rpl_free(authzid as *mut libc::c_void);
                if res != GSASL_OK as libc::c_int { return res }
            }
            input = input.offset(headerlen as isize);
            input_len =
                (input_len as libc::c_ulong).wrapping_sub(headerlen as u64) as size_t
                    as size_t;
            res =
                gsasl_property_set_raw(sctx, GSASL_SAML20_IDP_IDENTIFIER,
                                       input, input_len);
            if res != GSASL_OK as libc::c_int { return res }
            p = gsasl_property_get(sctx, GSASL_SAML20_REDIRECT_URL);
            if p.is_null() || *p == 0 {
                return GSASL_NO_SAML20_REDIRECT_URL as libc::c_int
            }
            *output_len = strlen(p);
            *output = malloc(*output_len as u64) as *mut libc::c_char;
            if (*output).is_null() {
                return GSASL_MALLOC_ERROR as libc::c_int
            }
            memcpy(*output as *mut libc::c_void, p as *const libc::c_void,
                   *output_len as u64);
            res = GSASL_NEEDS_MORE as libc::c_int;
            (*state).step += 1
        }
        1 => {
            if !(input_len == 1 && *input as libc::c_int == '=' as i32) {
                return GSASL_MECHANISM_PARSE_ERROR as libc::c_int
            }
            res =
                gsasl_callback(0 as *mut Shared, sctx, GSASL_VALIDATE_SAML20);
            if res != GSASL_OK as libc::c_int { return res }
            *output = 0 as *mut libc::c_char;
            *output_len = 0 as libc::c_int as size_t;
            res = GSASL_OK as libc::c_int;
            (*state).step += 1
        }
        _ => { }
    }
    return res;
}
/* saml20.h --- Prototypes for SAML20.
 * Copyright (C) 2010-2021 Simon Josefsson
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
pub unsafe fn _gsasl_saml20_server_finish(mech_data: Option<NonNull<()>>)
{
    let mech_data = mech_data
        .map(|ptr| ptr.as_ptr())
        .unwrap_or_else(std::ptr::null_mut);

    let mut state: *mut saml20_server_state =
        mech_data as *mut saml20_server_state;
    if state.is_null() { return }
    rpl_free(state as *mut libc::c_void);
}
