use lsp_types::Uri;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(not(windows))]
pub use std::fs::canonicalize as strict_canonicalize;

/// On Windows, rewrites the wide path prefix `\\?\C:` to `C:`  
/// Source: https://stackoverflow.com/a/70970317
#[inline]
#[cfg(windows)]
fn strict_canonicalize<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
    use std::io;

    fn impl_(path: PathBuf) -> std::io::Result<PathBuf> {
        let head = path
            .components()
            .next()
            .ok_or(io::Error::new(io::ErrorKind::Other, "empty path"))?;
        let disk_;
        let head = if let std::path::Component::Prefix(prefix) = head {
            if let std::path::Prefix::VerbatimDisk(disk) = prefix.kind() {
                disk_ = format!("{}:", disk as char);
                Path::new(&disk_).components().next().ok_or(io::Error::new(
                    io::ErrorKind::Other,
                    "failed to parse disk component",
                ))?
            } else {
                head
            }
        } else {
            head
        };
        Ok(std::iter::once(head)
            .chain(path.components().skip(1))
            .collect())
    }

    let canon = std::fs::canonicalize(path)?;
    impl_(canon)
}

mod sealed {
    pub trait Sealed {}
}

/// Provide methods to [`lsp_types::Uri`] to fill blanks left by
/// `fluent_uri` (the underlying type) especially when converting to and from file paths.
pub trait UriExt: Sized + sealed::Sealed {
    /// Assuming the URL is in the `file` scheme or similar,
    /// convert its path to an absolute `std::path::Path`.
    ///
    /// **Note:** This does not actually check the URL’s `scheme`, and may
    /// give nonsensical results for other schemes. It is the user’s
    /// responsibility to check the URL’s scheme before calling this.
    ///
    /// e.g. `Uri("file:///etc/passwd")` becomes `PathBuf("/etc/passwd")`
    fn to_file_path(&self) -> Option<Cow<Path>>;

    /// Convert a file path to a [`lsp_types::Uri`].
    ///
    /// Create a [`lsp_types::Uri`] from a file path.
    ///
    /// Returns `None` if the file does not exist.
    fn from_file_path<A: AsRef<Path>>(path: A) -> Option<Self>;
}

impl sealed::Sealed for lsp_types::Uri {}

impl UriExt for lsp_types::Uri {
    fn to_file_path(&self) -> Option<Cow<Path>> {
        let path = match self.path().as_estr().decode().into_string_lossy() {
            Cow::Borrowed(ref_) => Cow::Borrowed(Path::new(ref_)),
            Cow::Owned(owned) => Cow::Owned(PathBuf::from(owned)),
        };

        if cfg!(windows) {
            let authority = self.authority().expect("url has no authority component");
            let host = authority.host().as_str();
            if host.is_empty() {
                // very high chance this is a `file:///` uri
                // in which case the path will include a leading slash we need to remove
                let host = path.to_string_lossy();
                let host = &host[1..];
                return Some(Cow::Owned(PathBuf::from(host)));
            }

            let host = format!("{host}:");
            Some(Cow::Owned(
                Path::new(&host)
                    .components()
                    .chain(path.components())
                    .collect(),
            ))
        } else {
            Some(path)
        }
    }

    fn from_file_path<A: AsRef<Path>>(path: A) -> Option<Self> {
        let path = path.as_ref();

        let fragment = if path.is_absolute() {
            Cow::Borrowed(path)
        } else {
            match strict_canonicalize(path) {
                Ok(path) => Cow::Owned(path),
                Err(_) => return None,
            }
        };

        let raw_uri = if cfg!(windows) {
            // we want to parse a triple-slash path for Windows paths
            // it's a shorthand for `file://localhost/C:/Windows` with the `localhost` omitted
            format!("file:///{}", fragment.to_string_lossy().replace("\\", "/"))
        } else {
            format!("file://{}", fragment.to_string_lossy())
        };

        Uri::from_str(&raw_uri).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::strict_canonicalize;
    use crate::UriExt;
    use lsp_types::Uri;
    use std::path::Path;

    #[test]
    #[cfg(windows)]
    fn test_idempotent_canonicalization() {
        let lhs = strict_canonicalize(Path::new(".")).unwrap();
        let rhs = strict_canonicalize(&lhs).unwrap();
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn test_path_roundtrip_conversion() {
        let src = strict_canonicalize(Path::new(".")).unwrap();
        let conv = Uri::from_file_path(&src).unwrap();
        let roundtrip = conv.to_file_path().unwrap();
        assert_eq!(src, roundtrip, "conv={conv:?}",);
    }

    #[test]
    #[cfg(windows)]
    fn test_windows_uri_roundtrip_conversion() {
        use std::str::FromStr;

        let uri = Uri::from_str("file:///C:/Windows").unwrap();
        let path = uri.to_file_path().unwrap();
        assert_eq!(&path, Path::new("C:/Windows"), "uri={uri:?}");

        let conv = Uri::from_file_path(&path).unwrap();

        assert_eq!(
            uri,
            conv,
            "path={path:?} left={} right={}",
            uri.as_str(),
            conv.as_str()
        );
    }
}
