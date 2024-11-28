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
    format,
    string::{String, ToString},
};

/// Canonicalized absolute path type.
///
/// - Starting with `/`
/// - No `.` or `..` components
/// - No redundant or tailing `/`
/// - Valid examples: "/", "/root/foo/bar"
///
/// Using `Cow` type to avoid unnecessary allocations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsPath<'a>(Cow<'a, str>);

impl AbsPath<'static> {
    /// Simply wrap a string into a `AbsPath`.
    ///
    /// Caller should ensure that the path is absolute and canonicalized.
    pub const fn new_owned(path: String) -> Self {
        Self(Cow::Owned(path))
    }

    /// Parse and canonicalize an absolute path from a string.
    ///
    /// - If the given path is not canonicalized, it will be canonicalized.
    /// - If the given path is not absolute, it will be prefixed with `/`.
    pub fn new_canonicalized(path: &str) -> Self {
        if !path.starts_with('/') {
            Self(Cow::Owned(canonicalize(&("/".to_owned() + path))))
        } else {
            Self(Cow::Owned(canonicalize(path)))
        }
    }
}

impl<'a> AbsPath<'a> {
    /// Simply wrap a str slice into a `AbsPath`.
    ///
    /// Caller should ensure that the path is absolute and canonicalized.
    pub const fn new(path: &'a str) -> Self {
        Self(Cow::Borrowed(path))
    }

    /// Trim the starting `/` to transform this `AbsPath` into a `RelPath`.
    pub fn to_rel(&self) -> RelPath {
        RelPath(Cow::Borrowed(self.0.trim_start_matches('/')))
    }

    /// Create a new `AbsPath` with 'static lifetime.
    pub fn to_owned(&self) -> AbsPath<'static> {
        AbsPath::new_owned(self.0.to_string())
    }

    /// Transform this `AbsPath` into a raw str slice.
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    /// Transform this `AbsPath` into a raw string.
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Concatenate a `RelPath` to this `AbsPath`.
    pub fn join(&self, rel: &RelPath) -> AbsPath<'static> {
        AbsPath::new_canonicalized(&format!("{}/{}", self.0, rel.0))
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

/// Canonicalized relative path type.
///
/// - No starting '/'
/// - No `.` components
/// - No redundant or tailing '/'
/// - Possibly starts with '..'
/// - Valid examples: "", "..", "../b", "../.."
///
/// Using `Cow` type to avoid unnecessary allocations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelPath<'a>(Cow<'a, str>);

impl RelPath<'static> {
    /// Simply wrap a string into a `RelPath`.
    ///
    /// Caller should ensure that the path is relative and canonicalized.
    pub const fn new_owned(path: String) -> Self {
        Self(Cow::Owned(path))
    }

    /// Parse and canonicalize a relative path from a string.
    ///
    /// - If the given path is not canonicalized, it will be canonicalized.
    /// - If the given path is absolute, the starting '/' will be trimmed.
    pub fn new_canonicalized(path: &str) -> Self {
        Self(Cow::Owned(canonicalize(path.trim_start_matches('/'))))
    }
}

impl<'a> RelPath<'a> {
    /// Simply wrap a string into a `RelPath`.
    ///
    /// Caller should ensure that the path is relative and canonicalized.
    pub const fn new(path: &'a str) -> Self {
        Self(Cow::Borrowed(path))
    }

    /// Wrap a string into a `RelPath` with possibly leading '/' trimmed.
    ///
    /// Caller should ensure that the path is canonicalized.
    pub fn new_trimmed(path: &'a str) -> Self {
        Self(Cow::Borrowed(path.trim_start_matches('/')))
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
/// assert_eq!(canonicalize("../foo/.."), "..");
/// ```
fn canonicalize(path: &str) -> String {
    let mut buf = String::new();
    let is_absolute = path.starts_with('/');
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                if !is_absolute && buf.is_empty() {
                    buf.push_str("..");
                    continue;
                }
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
        assert_eq!(canonicalize("a/..//.."), "..");
        assert_eq!(canonicalize("././a"), "a");
        assert_eq!(canonicalize(".././a"), "../a");
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
        assert_eq!(canonicalize("../foo"), "../foo");
        assert_eq!(canonicalize("../foo/"), "../foo");
        assert_eq!(canonicalize("/../foo"), "/foo");
        assert_eq!(canonicalize("/../foo/"), "/foo");
        assert_eq!(canonicalize("/../../foo"), "/foo");
        assert_eq!(canonicalize("/bleh/../../foo"), "/foo");
        assert_eq!(canonicalize("/bleh/bar/../../foo"), "/foo");
        assert_eq!(canonicalize("/bleh/bar/../../foo/.."), "/");
        assert_eq!(canonicalize("/bleh/bar/../../foo/../meh"), "/meh");
    }
}
