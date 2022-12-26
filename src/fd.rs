use serde::{Serialize, Serializer};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use zbus::zvariant::{Signature, Type};

const SIGNATURE_STR: &str = "h";

#[derive(Debug)]
pub(crate) struct Fd<'f>(BorrowedFd<'f>);

impl<'f, T: AsFd + 'f> From<&'f T> for Fd<'f> {
    fn from(fd: &'f T) -> Self {
        Fd(fd.as_fd())
    }
}

impl<'f> Serialize for Fd<'f> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.0.as_fd().as_raw_fd())
    }
}

impl<'f> Type for Fd<'f> {
    fn signature() -> Signature<'static> {
        Signature::from_static_str_unchecked(SIGNATURE_STR)
    }
}
