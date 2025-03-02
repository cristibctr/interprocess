use std::borrow::Cow;
#[cfg(any(unix, target_vendor = "wasmer"))]
use std::ffi::OsStr;
#[cfg(windows)]
use widestring::U16CStr;

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum NameInner<'s> {
    #[cfg(windows)]
    NamedPipe(Cow<'s, U16CStr>),
    #[cfg(any(unix, target_vendor = "wasmer"))]
    UdSocketPath(Cow<'s, OsStr>),
    #[cfg(any(unix, target_vendor = "wasmer"))]
    UdSocketPseudoNs(Cow<'s, OsStr>),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    UdSocketNs(Cow<'s, [u8]>),
}

impl Default for NameInner<'_> {
    fn default() -> Self {
        #[cfg(windows)]
        {
            Self::NamedPipe(Cow::default())
        }
        #[cfg(any(unix, target_vendor = "wasmer"))]
        {
            Self::UdSocketPath(Cow::default())
        }
    }
}

macro_rules! map_cow {
    ($nm:ident in $var:expr => $e:expr) => {
        match $var {
            #[cfg(windows)]
            NameInner::NamedPipe($nm) => NameInner::NamedPipe($e),
            #[cfg(any(unix, target_vendor = "wasmer"))]
            NameInner::UdSocketPath($nm) => NameInner::UdSocketPath($e),
            #[cfg(any(unix, target_vendor = "wasmer"))]
            NameInner::UdSocketPseudoNs($nm) => NameInner::UdSocketPseudoNs($e),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            NameInner::UdSocketNs($nm) => NameInner::UdSocketNs($e),
        }
    };
}

impl NameInner<'_> {
    pub const fn is_namespaced(&self) -> bool {
        match self {
            #[cfg(windows)]
            Self::NamedPipe(..) => true,
            #[cfg(any(unix, target_vendor = "wasmer"))]
            Self::UdSocketPath(..) => false,
            #[cfg(any(unix, target_vendor = "wasmer"))]
            Self::UdSocketPseudoNs(..) => false,
            #[cfg(any(target_os = "linux", target_os = "android"))]
            Self::UdSocketNs(..) => true,
        }
    }
    pub const fn is_path(&self) -> bool {
        match self {
            #[cfg(windows)]
            Self::NamedPipe(..) => true,
            #[cfg(any(unix, target_vendor = "wasmer"))]
            Self::UdSocketPath(..) => true,
            #[cfg(any(unix, target_vendor = "wasmer"))]
            Self::UdSocketPseudoNs(..) => false,
            #[cfg(any(target_os = "linux", target_os = "android"))]
            Self::UdSocketNs(..) => false,
        }
    }

    #[inline]
    pub fn borrow(&self) -> NameInner<'_> { map_cow!(cow in self => Cow::Borrowed(cow)) }

    pub fn into_owned(self) -> NameInner<'static> {
        map_cow!(cow in self => Cow::Owned(cow.into_owned()))
    }
}
