/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Utilities for path manipulation.

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
};

/// Returns the canonical form of the path with all intermediate components
/// normalized.
///
/// It won't force convert the path to an absolute form.
///
/// # Examples
///
/// ```
/// use axfs_vfs::path::canonicalize;
///
/// assert_eq!(canonicalize("/path/./to//foo"), "/path/to/foo");
/// assert_eq!(canonicalize("/./path/to/../bar.rs"), "/path/bar.rs");
/// assert_eq!(canonicalize("./foo/./bar"), "foo/bar");
/// ```
pub fn canonicalize(path: &str) -> String {
    let mut buf = String::new();
    let is_absolute = path.starts_with('/');
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                while !buf.is_empty() {
                    if buf == "/" {
                        break;
                    }
                    let c = buf.pop().unwrap();
                    if c == '/' {
                        break;
                    }
                }
            }
            _ => {
                if buf.is_empty() {
                    if is_absolute {
                        buf.push('/');
                    }
                } else if &buf[buf.len() - 1..] != "/" {
                    buf.push('/');
                }
                buf.push_str(part);
            }
        }
    }
    if is_absolute && buf.is_empty() {
        buf.push('/');
    }
    buf
}

/// CANONICALIZED absolute path type, starting with '/'.
///
/// Using `Cow` type to avoid unnecessary allocations.
#[derive(Debug)]
pub struct AbsPath<'a>(Cow<'a, str>);

impl<'a> AbsPath<'a> {
    /// Simply wrap a string into a `AbsPath`.
    pub fn new(path: &'a str) -> Self {
        Self(Cow::Borrowed(path))
    }

    /// Parse and canonicalize an absolute path from a string.
    pub fn new_canonicalized(path: &str) -> Self {
        if !path.starts_with('/') {
            Self(Cow::Owned(canonicalize(&("/".to_owned() + path))))
        } else {
            Self(Cow::Owned(canonicalize(path)))
        }
    }
}

impl core::ops::Deref for AbsPath<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Display for AbsPath<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// CANONICALIZED relative path type, no starting '.' or '/'.
/// possibly starts with '..'.
///
/// Valid examples:
/// - ""
/// - ".."
/// - "../b"
/// - "../.."
/// - "a/b/c"
///
/// Using `Cow` type to avoid unnecessary allocations.
pub struct RelPath<'a>(Cow<'a, str>);

impl<'a> RelPath<'a> {
    /// Wrap a string into a `RelPath`.
    pub fn new(path: &'a str) -> Self {
        Self(Cow::Borrowed(path))
    }

    /// Parse and canonicalize a relative path from a string.
    pub fn new_canonicalized(path: &str) -> Self {
        Self(Cow::Owned(canonicalize(path.trim_start_matches("/"))))
    }
}

impl core::ops::Deref for RelPath<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Display for RelPath<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_canonicalize() {
        assert_eq!(canonicalize(""), "");
        assert_eq!(canonicalize("///"), "/");
        assert_eq!(canonicalize("//a//.//b///c//"), "/a/b/c");
        assert_eq!(canonicalize("/a/../"), "/");
        assert_eq!(canonicalize("/a/../..///"), "/");
        assert_eq!(canonicalize("a/../"), "");
        assert_eq!(canonicalize("a/..//.."), "");
        assert_eq!(canonicalize("././a"), "a");
        assert_eq!(canonicalize(".././a"), "a");
        assert_eq!(canonicalize("/././a"), "/a");
        assert_eq!(canonicalize("/abc/../abc"), "/abc");
        assert_eq!(canonicalize("/test"), "/test");
        assert_eq!(canonicalize("/test/"), "/test");
        assert_eq!(canonicalize("test/"), "test");
        assert_eq!(canonicalize("test"), "test");
        assert_eq!(canonicalize("/test//"), "/test");
        assert_eq!(canonicalize("/test/foo"), "/test/foo");
        assert_eq!(canonicalize("/test/foo/"), "/test/foo");
        assert_eq!(canonicalize("/test/foo/bar"), "/test/foo/bar");
        assert_eq!(canonicalize("/test/foo/bar//"), "/test/foo/bar");
        assert_eq!(canonicalize("/test//foo/bar//"), "/test/foo/bar");
        assert_eq!(canonicalize("/test//./foo/bar//"), "/test/foo/bar");
        assert_eq!(canonicalize("/test//./.foo/bar//"), "/test/.foo/bar");
        assert_eq!(canonicalize("/test//./..foo/bar//"), "/test/..foo/bar");
        assert_eq!(canonicalize("/test//./../foo/bar//"), "/foo/bar");
        assert_eq!(canonicalize("/test/../foo"), "/foo");
        assert_eq!(canonicalize("/test/bar/../foo"), "/test/foo");
        assert_eq!(canonicalize("../foo"), "foo");
        assert_eq!(canonicalize("../foo/"), "foo");
        assert_eq!(canonicalize("/../foo"), "/foo");
        assert_eq!(canonicalize("/../foo/"), "/foo");
        assert_eq!(canonicalize("/../../foo"), "/foo");
        assert_eq!(canonicalize("/bleh/../../foo"), "/foo");
        assert_eq!(canonicalize("/bleh/bar/../../foo"), "/foo");
        assert_eq!(canonicalize("/bleh/bar/../../foo/.."), "/");
        assert_eq!(canonicalize("/bleh/bar/../../foo/../meh"), "/meh");
    }
}
