extern crate libc;


/// For checking if file is still opened. Note that multiple fd's could point to the same file.
pub fn is_valid_fd(fd: i32) -> bool {
    unsafe {
        if libc::fcntl(fd, libc::F_GETFD) != -1 || *libc::__errno_location() != libc::EBADF {
            return true;
        }
        false
    }
}