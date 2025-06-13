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
            .ok_or(io::Error::other("empty path"))?;
        let disk_;
        let head = if let std::path::Component::Prefix(prefix) = head {
            if let std::path::Prefix::VerbatimDisk(disk) = prefix.kind() {
                disk_ = format!("{}:", disk as char);
                Path::new(&disk_)
                    .components()
                    .next()
                    .ok_or(io::Error::other("failed to parse disk component"))?
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

#[cfg(windows)]
fn capitalize_drive_letter(path: &str) -> String {
    // Check if it's a Windows path starting with a drive letter like "c:/"
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        let mut chars = path.chars();
        let drive_letter = chars.next().unwrap().to_ascii_uppercase();
        let rest: String = chars.collect();
        format!("{}{}", drive_letter, rest)
    } else {
        path.to_string()
    }
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
            let auth_host = self
                .authority()
                .map(|auth| auth.host().as_str())
                .unwrap_or_default();

            if auth_host.is_empty() {
                // very high chance this is a `file:///c:/...` uri
                // in which case the path will include a leading slash we
                // need to remove to get `c:/...`
                let host = path.to_string_lossy();
                let host = &host[1..];
                return Some(Cow::Owned(PathBuf::from(host)));
            }

            Some(Cow::Owned(
                // `file://server/...` becomes `server:/`
                Path::new(&format!("{auth_host}:"))
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

        #[cfg(windows)]
        let raw_uri = {
            // we want to parse a triple-slash path for Windows paths
            // it's a shorthand for `file://localhost/C:/Windows` with the `localhost` omitted
            format!(
                "file:///{}",
                capitalize_drive_letter(&fragment.to_string_lossy()).replace('\\', "/")
            )
        };

        #[cfg(not(windows))]
        let raw_uri = { format!("file://{}", fragment.to_string_lossy()) };

        Self::from_str(&raw_uri).ok()
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

        let uris = [
            Uri::from_str("file:///C:/some/path/to/file.txt").unwrap(),
            Uri::from_str("file:///c:/some/path/to/file.txt").unwrap(),
            Uri::from_str("file:///c%3A/some/path/to/file.txt").unwrap(),
        ];

        let final_uri = Uri::from_str("file:///C:/some/path/to/file.txt").unwrap();

        for uri in uris {
            let path = uri.to_file_path().unwrap();
            assert_eq!(
                &path,
                Path::new("C:\\some\\path\\to\\file.txt"),
                "uri={uri:?}"
            );

            let conv = Uri::from_file_path(&path).unwrap();

            assert_eq!(
                final_uri,
                conv,
                "path={path:?} left={} right={}",
                final_uri.as_str(),
                conv.as_str()
            );
        }
    }
}
