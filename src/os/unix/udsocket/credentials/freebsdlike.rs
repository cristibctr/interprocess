#[cfg(uds_sockcred)]
use libc::sockcred;
use libc::{c_int, c_short, cmsgcred, gid_t, pid_t, uid_t};
use std::{cmp::min, marker::PhantomData, mem::size_of, ptr::addr_of};
use to_method::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Credentials<'a> {
    Cmsgcred(&'a cmsgcred_packed),
    #[cfg(uds_sockcred)]
    Sockcred(&'a sockcred_packed),
}
impl<'a> Credentials<'a> {
    pub fn euid(self) -> Option<uid_t> {
        Some(match self {
            Credentials::Cmsgcred(c) => c.cmcred_euid,
            #[cfg(uds_sockcred)]
            Credentials::Sockcred(c) => c.sc_euid,
        })
    }
    pub fn ruid(self) -> Option<uid_t> {
        Some(match self {
            Credentials::Cmsgcred(c) => c.cmcred_uid,
            #[cfg(uds_sockcred)]
            Credentials::Sockcred(c) => c.sc_uid,
        })
    }
    pub fn egid(self) -> Option<gid_t> {
        match self {
            Credentials::Cmsgcred(_) => None,
            #[cfg(uds_sockcred)]
            Credentials::Sockcred(c) => Some(c.sc_egid),
        }
    }
    pub fn rgid(self) -> Option<gid_t> {
        Some(match self {
            Credentials::Cmsgcred(c) => c.cmcred_gid,
            #[cfg(uds_sockcred)]
            Credentials::Sockcred(c) => c.sc_gid,
        })
    }
    pub fn pid(self) -> Option<pid_t> {
        match self {
            Credentials::Cmsgcred(c) => Some(c.cmcred_pid),
            #[cfg(uds_sockcred)]
            Credentials::Sockcred(_) => None,
        }
    }
    fn ptr_to_gids(&self) -> *const gid_packed {
        match self {
            Credentials::Cmsgcred(c) => <*const _>::cast::<gid_packed>(addr_of!(c.cmcred_groups)),
            #[cfg(uds_sockcred)]
            Credentials::Sockcred(c) => <*const _>::cast::<gid_packed>(addr_of!(c.sc_groups)),
        }
    }
    pub fn groups(&self) -> Groups<'a> {
        let n_groups = match self {
            Credentials::Cmsgcred(c) => min(c.cmcred_ngroups, libc::CMGROUP_MAX as _).to::<c_int>(),
            #[cfg(uds_sockcred)]
            Credentials::Sockcred(c) => c.sc_ngroups.to::<c_int>(),
        }
        .try_to::<usize>()
        .unwrap();
        let cur = self.ptr_to_gids();
        let end = unsafe {
            // SAFETY: this puts us one byte past the last one
            cur.add(n_groups)
        };
        Groups {
            cur,
            end,
            _phantom: PhantomData,
        }
    }
}

#[allow(non_camel_case_types)]
type gid_packed = [u8; size_of::<gid_t>()];

#[derive(Clone, Debug)]
pub struct Groups<'a> {
    cur: *const gid_packed,
    end: *const gid_packed,
    _phantom: PhantomData<Credentials<'a>>,
}
impl Iterator for Groups<'_> {
    type Item = gid_t;
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.end {
            return None;
        }
        let gid_bytes = unsafe { *self.cur };
        self.cur = unsafe {
            // SAFETY: furthest this can go is one byte past the end, which is allowed
            self.cur.add(1)
        };
        Some(gid_t::from_ne_bytes(gid_bytes))
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}
impl ExactSizeIterator for Groups<'_> {
    #[inline]
    fn len(&self) -> usize {
        unsafe { self.end.offset_from(self.cur) as usize }
    }
}

// The two below are pub(crate) solely to allow the enum to have them

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct cmsgcred_packed {
    pub cmcred_pid: pid_t,
    pub cmcred_uid: uid_t,
    pub cmcred_euid: uid_t,
    pub cmcred_gid: gid_t,
    pub cmcred_ngroups: c_short,
    pub __pad0: c_short,
    pub cmcred_groups: [gid_t; 16],
}
impl AsRef<cmsgcred_packed> for cmsgcred {
    fn as_ref(&self) -> &cmsgcred_packed {
        const _: () = {
            if size_of::<cmsgcred_packed>() != size_of::<cmsgcred>() {
                panic!("size of `cmsgcred_packed` did not match that of `cmsgcred`");
            }
        };
        unsafe {
            // SAFETY: the two types have the same layout, save for stricter padding of the input
            &*<*const _>::cast(self)
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg(uds_sockcred)]
pub(crate) struct sockcred_packed {
    pub sc_uid: uid_t,
    pub sc_euid: uid_t,
    pub sc_gid: gid_t,
    pub sc_egid: gid_t,
    pub sc_ngroups: c_int,
    pub sc_groups: [gid_t; 1],
}
#[cfg(uds_sockcred)]
impl AsRef<sockcred_packed> for sockcred {
    fn as_ref(&self) -> &sockcred_packed {
        const _: () = {
            if size_of::<sockcred_packed>() != size_of::<sockcred>() {
                panic!("size of `sockcred_packed` did not match that of `sockcred`");
            }
        };
        unsafe {
            // SAFETY: as above
            &*<*const _>::cast(self)
        }
    }
}
