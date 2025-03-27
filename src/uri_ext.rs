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

/// Provide more methods to [`fluent_uri::Uri`]. Especially to convert to and
/// from file paths.
pub trait UriExt {
    /// Convert the path component of a [`fluent_uri::Uri`] to a file path.
    fn to_file_path(&self) -> Option<Cow<Path>>;
}

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
}

/// Create a [`fluent_uri::Uri`] from a file path.
pub fn uri_from_file_path(path: &Path) -> Option<Uri> {
    let fragment = if !path.is_absolute() {
        Cow::from(strict_canonicalize(path).ok()?)
    } else {
        Cow::from(path)
    };

    if cfg!(windows) {
        // we want to parse a triple-slash path for Windows paths
        // it's a shorthand for `file://localhost/C:/Windows` with the `localhost` omitted
        let raw = format!("file:///{}", fragment.to_string_lossy().replace("\\", "/"));
        Uri::from_str(&raw).ok()
    } else {
        Uri::from_str(&format!("file://{}", fragment.to_string_lossy())).ok()
    }
}
