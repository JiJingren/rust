//! Random data on Apple platforms.
//!
//! `CCRandomGenerateBytes` calls into `CCRandomCopyBytes` with `kCCRandomDefault`.
//! `CCRandomCopyBytes` manages a CSPRNG which is seeded from the kernel's CSPRNG.
//! We use `CCRandomGenerateBytes` instead of `SecCopyBytes` because it is accessible via
//! `libSystem` (libc) while the other needs to link to `Security.framework`.
//!
//! Note that technically, `arc4random_buf` is available as well, but that calls
//! into the same system service anyway, and `CCRandomGenerateBytes` has been
//! proven to be App Store-compatible.

pub fn fill_bytes(bytes: &mut [u8]) {
    let ret = unsafe { libc::CCRandomGenerateBytes(bytes.as_mut_ptr().cast(), bytes.len()) };
    assert_eq!(ret, libc::kCCSuccess, "failed to generate random data");
}

#[cfg(target_os = "ios")]
#[allow(non_snake_case, dead_code)]
mod legacy_ios_shim {
    #![allow(dead_code)]

    #[unsafe(no_mangle)]
    extern "C" fn CCRandomGenerateBytes(
        bytes: *mut core::ffi::c_void,
        size: libc::size_t,
    ) -> libc::CCRNGStatus {
        const URANDOM: &[u8] = b"/dev/urandom\0";

        if bytes.is_null() {
            return if size == 0 { 0 } else { -1 };
        }

        let fd = unsafe { libc::open(URANDOM.as_ptr().cast(), libc::O_RDONLY) };
        if fd < 0 {
            return -1;
        }

        let mut status: libc::CCRNGStatus = 0;
        let mut filled = 0;
        while filled < size {
            let dst = unsafe { (bytes.cast::<u8>()).add(filled).cast() };
            let n = unsafe { libc::read(fd, dst, size - filled) };
            if n <= 0 {
                status = -1;
                break;
            }
            filled += n as usize;
        }
        unsafe { libc::close(fd) };

        status
    }
}
