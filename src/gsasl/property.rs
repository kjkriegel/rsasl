use crate::gsasl::consts::*;
use crate::gsasl::consts::{Gsasl_property, GSASL_OK};
use crate::property::*;
use crate::session::SessionData;
use libc::{size_t, strlen};
use std::ffi::CString;
use std::sync::Arc;

pub unsafe fn gsasl_property_set(
    mut sctx: &mut SessionData,
    mut prop: Gsasl_property,
    mut data: *const libc::c_char,
) -> libc::c_int {
    return gsasl_property_set_raw(
        sctx,
        prop,
        data,
        if !data.is_null() {
            strlen(data) as usize
        } else {
            0
        },
    );
}

pub unsafe fn gsasl_property_set_raw(
    mut sctx: &mut SessionData,
    mut prop: Gsasl_property,
    mut data: *const libc::c_char,
    mut len: size_t,
) -> libc::c_int {
    let bytes = std::slice::from_raw_parts(data as *const u8, len);
    let mut vec = Vec::with_capacity(len);
    vec.extend_from_slice(bytes);
    let cstring = CString::new(vec)
        .expect("gsasl_property_set_raw called with NULL-containing string")
        .into_string()
        .expect("gsasl_propery_set_raw called with non-UTF8 string");
    sctx.set_property_raw(prop, Arc::new(cstring));

    return GSASL_OK as libc::c_int;
}
/* *
 * gsasl_property_fast:
 * @sctx: session handle.
 * @prop: enumerated value of Gsasl_property type, indicating the
 *        type of data in @data.
 *
 * Retrieve the data stored in the session handle for given property
 * @prop.
 *
 * The pointer is to live data, and must not be deallocated or
 * modified in any way.
 *
 * This function will not invoke the application callback.
 *
 * Return value: Return property value, if known, or NULL if no value
 *   known.
 *
 * Since: 0.2.0
 **/
unsafe fn gsasl_property_fast(sctx: &mut SessionData, prop: Gsasl_property) -> *const libc::c_char {
    if GSASL_OPENID20_OUTCOME_DATA == prop {
        if let Some(prop) = sctx.get_property::<OpenID20OutcomeData>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_OPENID20_REDIRECT_URL == prop {
        if let Some(prop) = sctx.get_property::<OpenID20RedirectUrl>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_SAML20_REDIRECT_URL == prop {
        if let Some(prop) = sctx.get_property::<SAML20RedirectUrl>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_SAML20_IDP_IDENTIFIER == prop {
        if let Some(prop) = sctx.get_property::<SAML20IDPIdentifier>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_CB_TLS_UNIQUE == prop {
        if let Some(prop) = sctx.get_property::<CBTlsUnique>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_SCRAM_STOREDKEY == prop {
        if let Some(prop) = sctx.get_property::<ScramStoredkey>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_SCRAM_SERVERKEY == prop {
        if let Some(prop) = sctx.get_property::<ScramServerkey>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_SCRAM_SALTED_PASSWORD == prop {
        if let Some(prop) = sctx.get_property::<ScramSaltedPassword>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_SCRAM_SALT == prop {
        if let Some(prop) = sctx.get_property::<ScramSalt>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_SCRAM_ITER == prop {
        if let Some(prop) = sctx.get_property::<ScramIter>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_QOP == prop {
        if let Some(it) = sctx.get_property::<Qop>() {
            let ptr = it.as_ptr();
            println!("ret {:?} @ {:?}", it, ptr);
            ptr
        } else {
            std::ptr::null()
        }
    } else if GSASL_QOPS == prop {
        if let Some(prop) = sctx.get_property::<Qops>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_DIGEST_MD5_HASHED_PASSWORD == prop {
        if let Some(prop) = sctx.get_property::<DigestMD5HashedPassword>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_REALM == prop {
        if let Some(prop) = sctx.get_property::<Realm>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_PIN == prop {
        if let Some(prop) = sctx.get_property::<Pin>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_SUGGESTED_PIN == prop {
        if let Some(prop) = sctx.get_property::<SuggestedPin>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_PASSCODE == prop {
        if let Some(prop) = sctx.get_property::<Passcode>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_GSSAPI_DISPLAY_NAME == prop {
        if let Some(prop) = sctx.get_property::<GssapiDisplayName>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_HOSTNAME == prop {
        if let Some(prop) = sctx.get_property::<Hostname>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_SERVICE == prop {
        if let Some(prop) = sctx.get_property::<Service>() {
            prop.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_ANONYMOUS_TOKEN == prop {
        if let Some(prop) = sctx.get_property::<AnonymousToken>() {
            let cstr = Box::leak(Box::new(CString::new(prop.as_bytes().to_owned()).unwrap()));
            cstr.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_PASSWORD == prop {
        if let Some(prop) = sctx.get_property::<Password>() {
            let cstr = Box::leak(Box::new(CString::new(prop.as_bytes().to_owned()).unwrap()));
            cstr.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_AUTHZID == prop {
        if let Some(prop) = sctx.get_property::<AuthzId>() {
            let cstr = Box::leak(Box::new(CString::new(prop.as_bytes().to_owned()).unwrap()));
            cstr.as_ptr()
        } else {
            std::ptr::null()
        }
    } else if GSASL_AUTHID == prop {
        if let Some(prop) = sctx.get_property::<AuthId>() {
            let cstr = Box::leak(Box::new(CString::new(prop.as_bytes().to_owned()).unwrap()));
            (*cstr).as_ptr()
        } else {
            std::ptr::null()
        }
    } else {
        std::ptr::null()
    }
}

pub unsafe fn gsasl_property_get(
    sctx: &mut SessionData,
    prop: Gsasl_property,
) -> *const libc::c_char {
    let mut ptr = gsasl_property_fast(sctx, prop);
    if ptr.is_null() {
        let _ = sctx.callback_raw(prop);
        ptr = gsasl_property_fast(sctx, prop);
    }
    ptr
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanisms::plain::mechinfo::PLAIN;
    use crate::Side;
    use std::ffi::CStr;
    use std::sync::Arc;

    #[test]
    fn property_get_set() {
        let mut session = SessionData::new(None, &PLAIN, Side::Client);

        unsafe {
            let ptr = gsasl_property_fast(&mut session, GSASL_QOP);
            assert!(ptr.is_null());
        }
        session.set_property::<Qop>(Arc::new(CString::new("testservice").unwrap()));
        let cstr = session.get_property::<Qop>();
        println!("cstr {:?}", cstr);
        unsafe {
            let ptr = gsasl_property_fast(&mut session, GSASL_QOP);
            println!("after {:?}", ptr);
            assert!(!ptr.is_null());
            let slc = std::slice::from_raw_parts(ptr as *const u8, 11);
            println!("Manual {}", std::str::from_utf8_unchecked(slc));
            let cstr = CStr::from_ptr(ptr);
            println!("fast {:?} {:?}", cstr, cstr.as_ptr());
            assert_eq!(cstr.to_str().unwrap(), "testservice");
        }
    }
}
