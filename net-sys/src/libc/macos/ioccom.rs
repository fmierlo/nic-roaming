// /Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include/sys/ioccom.h

// Ioctl's have the command encoded in the lower word, and the size of
// any in or out parameters in the upper word.  The high 3 bits of the
// upper word are used to encode the in/out status of the parameter.

use libc::c_ulong;

// param char 'i' as c_ulong
pub(crate) const I: c_ulong = 105;
// parameter length, at most 13 bits
const IOCPARM_MASK: c_ulong = 0x1fff;
// copy parameters out
const IOC_OUT: c_ulong = 0x40000000;
// copy parameters in
const IOC_IN: c_ulong = 0x80000000;
// copy parameters in and out
const IOC_INOUT: c_ulong = IOC_IN | IOC_OUT;

#[cfg(not(tarpaulin_include))]
const fn ioc(inout: c_ulong, group: c_ulong, num: c_ulong, len: c_ulong) -> c_ulong {
    inout | ((len & IOCPARM_MASK) << 16) | ((group) << 8) | (num)
}

#[cfg(not(tarpaulin_include))]
pub(crate) const fn iow(group: c_ulong, num: c_ulong, len: c_ulong) -> c_ulong {
    ioc(IOC_IN, group, num, len)
}

#[cfg(not(tarpaulin_include))]
pub(crate) const fn iorw(group: c_ulong, num: c_ulong, len: c_ulong) -> c_ulong {
    ioc(IOC_INOUT, group, num, len)
}
