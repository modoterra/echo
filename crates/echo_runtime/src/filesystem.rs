use std::path::PathBuf;

#[cfg(unix)]
pub(crate) fn path_bytes(path: PathBuf) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStringExt;

    Some(path.into_os_string().into_vec())
}

#[cfg(not(unix))]
pub(crate) fn path_bytes(path: PathBuf) -> Option<Vec<u8>> {
    path.into_os_string()
        .into_string()
        .ok()
        .map(String::into_bytes)
}

#[cfg(unix)]
pub(crate) fn push_path_component_from_bytes(path: &mut PathBuf, component: &[u8]) -> Option<()> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    path.push(OsStr::from_bytes(component));
    Some(())
}

#[cfg(not(unix))]
pub(crate) fn push_path_component_from_bytes(path: &mut PathBuf, component: &[u8]) -> Option<()> {
    path.push(std::str::from_utf8(component).ok()?);
    Some(())
}

#[cfg(unix)]
pub(crate) fn path_buf_from_bytes(bytes: &[u8]) -> Option<PathBuf> {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    Some(PathBuf::from(OsStr::from_bytes(bytes)))
}

#[cfg(not(unix))]
pub(crate) fn path_buf_from_bytes(bytes: &[u8]) -> Option<PathBuf> {
    std::str::from_utf8(bytes).ok().map(PathBuf::from)
}
