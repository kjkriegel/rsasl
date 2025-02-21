use crate::gsasl::error::{gsasl_strerror, gsasl_strerror_name};
use crate::property::Property;
use crate::validate::Validation;
use crate::{Mechname, PropertyQ};
use base64::DecodeError;
use std::cmp::min;
use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};
use std::{fmt, io};

pub type Result<T> = std::result::Result<T, SASLError>;

static UNKNOWN_ERROR: &'static str = "The given error code is unknown to gsasl";

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
/// Statically sized mechanism name container
pub struct MechanismArray {
    len: u8,
    data: [u8; 20],
}
impl MechanismArray {
    pub fn new(name: &Mechname) -> Self {
        let len = min(name.len(), 20);
        let mut data = [0u8; 20];
        (&mut data[0..len]).copy_from_slice(name.as_bytes());

        Self {
            len: len as u8,
            data,
        }
    }

    pub fn as_mechname(&self) -> &Mechname {
        // Safe because the only way to construct `Self` is from a valid Mechname
        Mechname::new_unchecked(&self.data[0..(self.len as usize)])
    }
}
impl Display for MechanismArray {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_mechname(), f)
    }
}

/// Different high-level kinds of errors that can happen in mechanisms
pub enum MechanismErrorKind {
    /// Parsing failed for the given reason (syntactical error)
    Parse,

    /// The messages where syntactically valid but a semantical error occurred during handling
    Protocol,

    /// While the exchange did complete correctly, the authentication itself failed for some
    /// reason or another.
    Outcome,
}

/// Errors specific to a certain mechanism
pub trait MechanismError: Debug + Display {
    fn kind(&self) -> MechanismErrorKind;
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Gsasl(pub i32);
impl Debug for Gsasl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(rsasl_errname_to_str(self.0 as u32).unwrap_or("UNKNOWN_ERROR"))
    }
}
impl Display for Gsasl {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(rsasl_err_to_str(self.0).unwrap_or("an unknown error was encountered"))
    }
}
impl MechanismError for Gsasl {
    fn kind(&self) -> MechanismErrorKind {
        // TODO: match self and return proper type
        MechanismErrorKind::Protocol
    }
}

#[derive(Debug)]
pub enum SessionError {
    Io {
        source: std::io::Error,
    },

    #[cfg(feature = "provider_base64")]
    Base64 {
        source: base64::DecodeError,
    },

    NoSecurityLayer,

    /// Authentication exchange as syntactically valid but failed. Returned e.g. if the provided
    /// password didn't match the provided user.
    AuthenticationFailure,

    // Common Mechanism Errors:
    /// Mechanism was called without input data when requiring some
    InputDataRequired,

    MechanismError(Box<dyn MechanismError>),

    NoCallback {
        property: Property,
    },
    NoValidate {
        validation: Validation,
    },

    NoProperty {
        property: Property,
    },
}
impl SessionError {
    pub fn no_property<P: PropertyQ>() -> Self {
        Self::NoProperty {
            property: P::property(),
        }
    }

    pub fn no_validate(validation: Validation) -> Self {
        Self::NoValidate { validation }
    }

    pub fn input_required() -> Self {
        Self::InputDataRequired
    }

    pub fn is_mechanism_error(&self) -> bool {
        match self {
            Self::MechanismError(_) => true,
            _ => false,
        }
    }
}
impl Display for SessionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { source } => write!(f, "I/O error occured: {}", source),

            #[cfg(feature = "provider_base64")]
            Self::Base64 { source } => {
                write!(f, "failed to decode base64-encoded input: {}", source)
            }

            Self::NoSecurityLayer => f.write_str("no security layer is installed"),
            Self::InputDataRequired => f.write_str("input data is required but None supplied"),

            Self::MechanismError(source) => Display::fmt(source, f),
            Self::NoCallback { property } => write!(
                f,
                "callback could not provide the requested property {:?}",
                property
            ),
            Self::NoValidate { validation } => {
                write!(f, "no validation callback for {} installed", validation)
            }
            Self::NoProperty { property } => write!(f, "required property {} is not set", property),
            SessionError::AuthenticationFailure => f.write_str("authentication failed"),
        }
    }
}

impl PartialEq for SessionError {
    fn eq(&self, other: &Self) -> bool {
        use SessionError::*;
        match (self, other) {
            (Base64 { source: a }, Base64 { source: b }) => a == b,
            (NoSecurityLayer, NoSecurityLayer) => true,
            (AuthenticationFailure, AuthenticationFailure) => true,
            (InputDataRequired, InputDataRequired) => true,
            (NoCallback { property: a }, NoCallback { property: b }) => a == b,
            (NoValidate { validation: a }, NoValidate { validation: b }) => a == b,
            (NoProperty { property: a }, NoProperty { property: b }) => a == b,
            _ => false,
        }
    }
}

impl From<io::Error> for SessionError {
    fn from(source: io::Error) -> Self {
        Self::Io { source }
    }
}

#[cfg(feature = "provider_base64")]
impl From<base64::DecodeError> for SessionError {
    fn from(source: base64::DecodeError) -> Self {
        Self::Base64 { source }
    }
}

impl<T: MechanismError + 'static> From<T> for SessionError {
    fn from(e: T) -> Self {
        Self::MechanismError(Box::new(e))
    }
}

// Contain mostly fat pointers so 16 bytes. Try to not be (much) bigger than that
pub enum SASLError {
    // outside errors
    Io { source: std::io::Error },
    Base64DecodeError { source: base64::DecodeError },

    // setup errors
    UnknownMechanism(MechanismArray),
    NoSharedMechanism,
    MechanismNameError(MechanismNameError),
    Gsasl(i32),
}

impl SASLError {
    pub fn unknown_mechanism(name: &Mechname) -> Self {
        Self::UnknownMechanism(MechanismArray::new(name))
    }
}

impl Debug for SASLError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SASLError::Io { source } => Debug::fmt(source, f),
            SASLError::UnknownMechanism(mecharray) => {
                write!(f, "UnknownMechanism(\"{}\")", mecharray)
            }
            SASLError::Base64DecodeError { source } => Debug::fmt(source, f),
            SASLError::MechanismNameError(e) => Debug::fmt(e, f),
            SASLError::Gsasl(n) => write!(
                f,
                "{}[{}]",
                rsasl_errname_to_str(*n as u32).unwrap_or("UNKNOWN_ERROR"),
                n
            ),
            SASLError::NoSharedMechanism => f.write_str("NoSharedMechanism"),
        }
    }
}

impl Display for SASLError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SASLError::Io { source } => Display::fmt(source, f),
            SASLError::UnknownMechanism(mecharray) => {
                write!(f, "mechanism {} is not implemented", mecharray)
            }
            SASLError::Base64DecodeError { source } => Display::fmt(source, f),
            SASLError::MechanismNameError(e) => Display::fmt(e, f),
            SASLError::Gsasl(n) => write!(
                f,
                "({}): {}",
                rsasl_errname_to_str(*n as u32).unwrap_or("UNKNOWN_ERROR"),
                gsasl_err_to_str_internal(*n)
            ),
            SASLError::NoSharedMechanism => f.write_str("no shared mechanism found to use"),
        }
    }
}

impl std::error::Error for SASLError {}

impl From<MechanismNameError> for SASLError {
    fn from(e: MechanismNameError) -> Self {
        SASLError::MechanismNameError(e)
    }
}

impl From<base64::DecodeError> for SASLError {
    fn from(source: DecodeError) -> Self {
        SASLError::Base64DecodeError { source }
    }
}

impl From<std::io::Error> for SASLError {
    fn from(source: std::io::Error) -> Self {
        SASLError::Io { source }
    }
}

impl From<i32> for SASLError {
    fn from(e: i32) -> Self {
        SASLError::Gsasl(e)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum MechanismNameError {
    /// Mechanism name longer than 20 characters
    TooLong,

    /// Mechanism name shorter than 1 character
    TooShort,

    /// Mechanism name contained a character outside of [A-Z0-9-_]
    InvalidChars(u8),
}

impl Display for MechanismNameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MechanismNameError::TooLong => {
                f.write_str("a mechanism name longer than 20 characters was provided")
            }
            MechanismNameError::TooShort => f.write_str("mechanism name can't be an empty string"),
            MechanismNameError::InvalidChars(byte)
                if !byte.is_ascii() || byte.is_ascii_whitespace() || byte.is_ascii_control() =>
            {
                write!(f, "mechanism name contains invalid character {:#x}", byte)
            }
            MechanismNameError::InvalidChars(byte) => write!(
                f,
                "mechanism name contains invalid character '{char}'",
                char = char::from_u32(*byte as u32).unwrap()
            ),
        }
    }
}

/// Convert an error code to a human readable description of that error
pub fn rsasl_err_to_str(err: libc::c_int) -> Option<&'static str> {
    // gsasl returns the normal zero-terminated string
    let cstr = unsafe {
        let ptr = gsasl_strerror(err as libc::c_int);
        if ptr.is_null() {
            return None;
        }

        CStr::from_ptr(ptr)
    };
    // Yes, this could potentially fail. But we're talking about an array of static, compiled-in
    // strings here. If they aren't UTF-8 that's clearly a bug.
    Some(
        cstr.to_str()
            .expect("GSASL library contains bad UTF-8 error descriptions"),
    )
}

/// Convert an error code to a human readable description of that error
#[deprecated(since = "1.1.0", note = "Use rsasl_err_to_str as replacement")]
pub fn gsasl_err_to_str(err: libc::c_int) -> &'static str {
    gsasl_err_to_str_internal(err)
}

fn gsasl_err_to_str_internal(err: libc::c_int) -> &'static str {
    // gsasl returns the normal zero-terminated string
    let cstr = unsafe {
        let ptr = gsasl_strerror(err);
        if ptr.is_null() {
            return UNKNOWN_ERROR;
        }

        CStr::from_ptr(ptr)
    };
    // Yes, this could potentially fail. But we're talking about an array of static, compiled-in
    // strings here. If they aren't UTF-8 that's clearly a bug.
    cstr.to_str()
        .expect("GSASL library contains bad UTF-8 error descriptions")
}

/// Convert an error type to the human readable name of that error.
/// i.e. rsasl_errname_to_str(GSASL_OK) -> "GSASL_OK". Returns `None` when an invalid libc::c_int is
/// passed.
pub fn rsasl_errname_to_str(err: libc::c_uint) -> Option<&'static str> {
    // gsasl returns the normal zero-terminated string
    let cstr = unsafe {
        let ptr = gsasl_strerror_name(err as libc::c_int);
        if ptr.is_null() {
            return None;
        }

        CStr::from_ptr(ptr)
    };
    // Yes, this could potentially fail. But we're talking about an array of static, compiled-in
    // strings here. If they aren't UTF-8 that's clearly a bug.
    Some(
        cstr.to_str()
            .expect("GSASL library contains bad UTF-8 error descriptions"),
    )
}

/// Convert an error code to the human readable name of that error.
/// i.e. gsasl_errname_to_str(GSASL_OK) -> "GSASL_OK"
#[deprecated]
pub fn gsasl_errname_to_str(err: libc::c_int) -> &'static str {
    // gsasl returns the normal zero-terminated string
    let cstr = unsafe {
        let ptr = gsasl_strerror_name(err);
        if ptr.is_null() {
            return UNKNOWN_ERROR;
        }

        CStr::from_ptr(ptr)
    };
    // Yes, this could potentially fail. But we're talking about an array of static, compiled-in
    // strings here. If they aren't UTF-8 that's clearly a bug.
    cstr.to_str()
        .expect("GSASL library contians bad UTF-8 error names")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gsasl::consts::*;

    #[test]
    fn errname_to_str_valid() {
        assert_eq!(rsasl_errname_to_str(GSASL_OK), Some("GSASL_OK"));
        assert_eq!(
            rsasl_errname_to_str(GSASL_NEEDS_MORE),
            Some("GSASL_NEEDS_MORE")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_UNKNOWN_MECHANISM),
            Some("GSASL_UNKNOWN_MECHANISM")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_MECHANISM_CALLED_TOO_MANY_TIMES),
            Some("GSASL_MECHANISM_CALLED_TOO_MANY_TIMES")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_MALLOC_ERROR),
            Some("GSASL_MALLOC_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_BASE64_ERROR),
            Some("GSASL_BASE64_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_CRYPTO_ERROR),
            Some("GSASL_CRYPTO_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_SASLPREP_ERROR),
            Some("GSASL_SASLPREP_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_MECHANISM_PARSE_ERROR),
            Some("GSASL_MECHANISM_PARSE_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_AUTHENTICATION_ERROR),
            Some("GSASL_AUTHENTICATION_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_INTEGRITY_ERROR),
            Some("GSASL_INTEGRITY_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_CLIENT_CODE),
            Some("GSASL_NO_CLIENT_CODE")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_SERVER_CODE),
            Some("GSASL_NO_SERVER_CODE")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_CALLBACK),
            Some("GSASL_NO_CALLBACK")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_ANONYMOUS_TOKEN),
            Some("GSASL_NO_ANONYMOUS_TOKEN")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_AUTHID),
            Some("GSASL_NO_AUTHID")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_AUTHZID),
            Some("GSASL_NO_AUTHZID")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_PASSWORD),
            Some("GSASL_NO_PASSWORD")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_PASSCODE),
            Some("GSASL_NO_PASSCODE")
        );
        assert_eq!(rsasl_errname_to_str(GSASL_NO_PIN), Some("GSASL_NO_PIN"));
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_SERVICE),
            Some("GSASL_NO_SERVICE")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_HOSTNAME),
            Some("GSASL_NO_HOSTNAME")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_CB_TLS_UNIQUE),
            Some("GSASL_NO_CB_TLS_UNIQUE")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_SAML20_IDP_IDENTIFIER),
            Some("GSASL_NO_SAML20_IDP_IDENTIFIER")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_SAML20_REDIRECT_URL),
            Some("GSASL_NO_SAML20_REDIRECT_URL")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_OPENID20_REDIRECT_URL),
            Some("GSASL_NO_OPENID20_REDIRECT_URL")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_RELEASE_BUFFER_ERROR),
            Some("GSASL_GSSAPI_RELEASE_BUFFER_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_IMPORT_NAME_ERROR),
            Some("GSASL_GSSAPI_IMPORT_NAME_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_INIT_SEC_CONTEXT_ERROR),
            Some("GSASL_GSSAPI_INIT_SEC_CONTEXT_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_ACCEPT_SEC_CONTEXT_ERROR),
            Some("GSASL_GSSAPI_ACCEPT_SEC_CONTEXT_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_UNWRAP_ERROR),
            Some("GSASL_GSSAPI_UNWRAP_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_WRAP_ERROR),
            Some("GSASL_GSSAPI_WRAP_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_ACQUIRE_CRED_ERROR),
            Some("GSASL_GSSAPI_ACQUIRE_CRED_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_DISPLAY_NAME_ERROR),
            Some("GSASL_GSSAPI_DISPLAY_NAME_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_UNSUPPORTED_PROTECTION_ERROR),
            Some("GSASL_GSSAPI_UNSUPPORTED_PROTECTION_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_SECURID_SERVER_NEED_ADDITIONAL_PASSCODE),
            Some("GSASL_SECURID_SERVER_NEED_ADDITIONAL_PASSCODE")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_SECURID_SERVER_NEED_NEW_PIN),
            Some("GSASL_SECURID_SERVER_NEED_NEW_PIN")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_ENCAPSULATE_TOKEN_ERROR),
            Some("GSASL_GSSAPI_ENCAPSULATE_TOKEN_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_DECAPSULATE_TOKEN_ERROR),
            Some("GSASL_GSSAPI_DECAPSULATE_TOKEN_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_INQUIRE_MECH_FOR_SASLNAME_ERROR),
            Some("GSASL_GSSAPI_INQUIRE_MECH_FOR_SASLNAME_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_TEST_OID_SET_MEMBER_ERROR),
            Some("GSASL_GSSAPI_TEST_OID_SET_MEMBER_ERROR")
        );
        assert_eq!(
            rsasl_errname_to_str(GSASL_GSSAPI_RELEASE_OID_SET_ERROR),
            Some("GSASL_GSSAPI_RELEASE_OID_SET_ERROR")
        );
    }

    #[test]
    fn errname_to_str_invalid() {
        assert_eq!(rsasl_errname_to_str(u32::MAX), None);
        assert_eq!(
            rsasl_errname_to_str(GSASL_NO_OPENID20_REDIRECT_URL as libc::c_uint + 1),
            None
        );
    }
}
