use std::fmt::Debug;

use libc::c_int;

// https://github.com/apple/darwin-xnu/blob/xnu-7195.121.3/bsd/net/route.h#L420

const RTM_GET_SILENT: c_int = 0x11;
const RTM_GET_EXT: c_int = 0x15;

// Route Message
#[repr(i32)]
pub(crate) enum Rtm {
    // unix
    RtmAdd = libc::RTM_ADD,
    RtmDelete = libc::RTM_DELETE,
    RtmChange = libc::RTM_CHANGE,
    RtmGet = libc::RTM_GET,
    RtmLosing = libc::RTM_LOSING,
    RtmRedirect = libc::RTM_REDIRECT,
    RtmMiss = libc::RTM_MISS,
    // apple
    RtmLock = libc::RTM_LOCK,
    RtmOldadd = libc::RTM_OLDADD,
    RtmOlddel = libc::RTM_OLDDEL,
    RtmResolve = libc::RTM_RESOLVE,
    RtmNewaddr = libc::RTM_NEWADDR,
    RtmDeladdr = libc::RTM_DELADDR,
    RtmIfinfo = libc::RTM_IFINFO,
    RtmNewmaddr = libc::RTM_NEWMADDR,
    RtmDelmaddr = libc::RTM_DELMADDR,
    RtmGetSilentPrivate = RTM_GET_SILENT, // private
    RtmIfinfo2 = libc::RTM_IFINFO2,
    RtmNewmaddr2 = libc::RTM_NEWMADDR2,
    RtmGet2 = libc::RTM_GET2,
    RtmGetExtPrivate = RTM_GET_EXT, // private
    RtmInvalid(c_int),
}

impl From<c_int> for Rtm {
    fn from(value: c_int) -> Self {
        match value {
            libc::RTM_ADD => Rtm::RtmAdd,
            libc::RTM_DELETE => Rtm::RtmDelete,
            libc::RTM_CHANGE => Rtm::RtmChange,
            libc::RTM_GET => Rtm::RtmGet,
            libc::RTM_LOSING => Rtm::RtmLosing,
            libc::RTM_REDIRECT => Rtm::RtmRedirect,
            libc::RTM_MISS => Rtm::RtmMiss,
            libc::RTM_LOCK => Rtm::RtmLock,
            libc::RTM_OLDADD => Rtm::RtmOldadd,
            libc::RTM_OLDDEL => Rtm::RtmOlddel,
            libc::RTM_RESOLVE => Rtm::RtmResolve,
            libc::RTM_NEWADDR => Rtm::RtmNewaddr,
            libc::RTM_DELADDR => Rtm::RtmDeladdr,
            libc::RTM_IFINFO => Rtm::RtmIfinfo,
            libc::RTM_NEWMADDR => Rtm::RtmNewmaddr,
            libc::RTM_DELMADDR => Rtm::RtmDelmaddr,
            RTM_GET_SILENT => Rtm::RtmGetSilentPrivate,
            libc::RTM_IFINFO2 => Rtm::RtmIfinfo2,
            libc::RTM_NEWMADDR2 => Rtm::RtmNewmaddr2,
            libc::RTM_GET2 => Rtm::RtmGet2,
            RTM_GET_EXT => Rtm::RtmGetExtPrivate,
            value => Rtm::RtmInvalid(value),
        }
    }
}

impl Debug for Rtm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RtmAdd => write!(f, "RtmAdd"),
            Self::RtmDelete => write!(f, "RtmDelete"),
            Self::RtmChange => write!(f, "RtmChange"),
            Self::RtmGet => write!(f, "RtmGet"),
            Self::RtmLosing => write!(f, "RtmLosing"),
            Self::RtmRedirect => write!(f, "RtmRedirect"),
            Self::RtmMiss => write!(f, "RtmMiss"),
            Self::RtmLock => write!(f, "RtmLock"),
            Self::RtmOldadd => write!(f, "RtmOldadd"),
            Self::RtmOlddel => write!(f, "RtmOlddel"),
            Self::RtmResolve => write!(f, "RtmResolve"),
            Self::RtmNewaddr => write!(f, "RtmNewaddr"),
            Self::RtmDeladdr => write!(f, "RtmDeladdr"),
            Self::RtmIfinfo => write!(f, "RtmIfinfo"),
            Self::RtmNewmaddr => write!(f, "RtmNewmaddr"),
            Self::RtmDelmaddr => write!(f, "RtmDelmaddr"),
            Self::RtmGetSilentPrivate => write!(f, "RtmGetSilentPrivate"),
            Self::RtmIfinfo2 => write!(f, "RtmIfinfo2"),
            Self::RtmNewmaddr2 => write!(f, "RtmNewmaddr2"),
            Self::RtmGet2 => write!(f, "RtmGet2"),
            Self::RtmGetExtPrivate => write!(f, "RtmGetExtPrivate"),
            Self::RtmInvalid(value) => f
                .debug_tuple("RtmInvalid")
                .field(&format!("{:x}", value))
                .finish(),
        }
    }
}
