/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use std::sync::Arc;

use axfs_vfs::{RelPath, VfsError, VfsNodeType, VfsResult};

use crate::*;

fn test_devfs_ops(devfs: &DeviceFileSystem) -> VfsResult {
    const N: usize = 32;
    let mut buf = [1; N];

    let root = devfs.root_dir();
    assert!(root.get_attr()?.is_dir());
    assert_eq!(root.get_attr()?.file_type(), VfsNodeType::Dir);
    assert_eq!(
        root.clone()
            .lookup(&RelPath::new_canonicalized("urandom"))
            .err(),
        Some(VfsError::NotFound)
    );

    let node = root.lookup(&RelPath::new_canonicalized("////null"))?;
    assert_eq!(node.get_attr()?.file_type(), VfsNodeType::CharDevice);
    assert!(!node.get_attr()?.is_dir());
    assert_eq!(node.get_attr()?.size(), 0);
    assert_eq!(node.read_at(0, &mut buf)?, 0);
    assert_eq!(buf, [1; N]);
    assert_eq!(node.write_at(N as _, &buf)?, N);
    assert_eq!(
        node.lookup(&RelPath::new_canonicalized("/")).err(),
        Some(VfsError::NotADirectory)
    );

    let node = devfs
        .root_dir()
        .lookup(&RelPath::new_canonicalized(".///.//././/.////zero"))?;
    assert_eq!(node.get_attr()?.file_type(), VfsNodeType::CharDevice);
    assert!(!node.get_attr()?.is_dir());
    assert_eq!(node.get_attr()?.size(), 0);
    assert_eq!(node.read_at(10, &mut buf)?, N);
    assert_eq!(buf, [0; N]);
    assert_eq!(node.write_at(0, &buf)?, N);

    let foo = devfs
        .root_dir()
        .lookup(&RelPath::new_canonicalized(".///.//././/.////foo"))?;
    assert!(foo.get_attr()?.is_dir());
    assert_eq!(
        foo.read_at(10, &mut buf).err(),
        Some(VfsError::IsADirectory)
    );
    assert!(Arc::ptr_eq(
        &foo.clone().lookup(&RelPath::new_canonicalized("/f2"))?,
        &devfs
            .root_dir()
            .lookup(&RelPath::new_canonicalized(".//./foo///f2"))?,
    ));
    assert_eq!(
        foo.clone()
            .lookup(&RelPath::new_canonicalized("/bar//f1"))?
            .get_attr()?
            .file_type(),
        VfsNodeType::CharDevice
    );
    assert_eq!(
        foo.lookup(&RelPath::new_canonicalized("/bar///"))?
            .get_attr()?
            .file_type(),
        VfsNodeType::Dir
    );

    Ok(())
}

fn test_get_parent(devfs: &DeviceFileSystem) -> VfsResult {
    let root = devfs.root_dir();
    assert!(root.parent().is_none());

    let node = root.clone().lookup(&RelPath::new_canonicalized("null"))?;
    assert!(node.parent().is_none());

    let node = root
        .clone()
        .lookup(&RelPath::new_canonicalized(".//foo/bar"))?;
    assert!(node.parent().is_some());
    let parent = node.parent().unwrap();
    assert!(Arc::ptr_eq(
        &parent,
        &root.clone().lookup(&RelPath::new_canonicalized("foo"))?
    ));
    assert!(parent.lookup(&RelPath::new_canonicalized("bar")).is_ok());

    let node = root.clone().lookup(&RelPath::new_canonicalized("foo/.."))?;
    assert!(Arc::ptr_eq(
        &node,
        &root.clone().lookup(&RelPath::new_canonicalized("."))?
    ));

    assert!(Arc::ptr_eq(
        &root
            .clone()
            .lookup(&RelPath::new_canonicalized("/foo/.."))?,
        &devfs
            .root_dir()
            .lookup(&RelPath::new_canonicalized(".//./foo/././bar/../.."))?,
    ));
    assert!(Arc::ptr_eq(
        &root.clone().lookup(&RelPath::new_canonicalized(
            "././/foo//./../foo//bar///..//././"
        ))?,
        &devfs
            .root_dir()
            .lookup(&RelPath::new_canonicalized(".//./foo/"))?,
    ));
    assert!(Arc::ptr_eq(
        &root
            .clone()
            .lookup(&RelPath::new_canonicalized("///foo//bar///../f2"))?,
        &root.lookup(&RelPath::new_canonicalized("foo/.//f2"))?,
    ));

    Ok(())
}

#[test]
fn test_devfs() {
    // .
    // ├── foo
    // │   ├── bar
    // │   │   └── f1 (null)
    // │   └── f2 (zero)
    // ├── null
    // └── zero

    let devfs = DeviceFileSystem::new();
    devfs.add("null", Arc::new(NullDev));
    devfs.add("zero", Arc::new(ZeroDev));

    let dir_foo = devfs.mkdir("foo");
    dir_foo.add("f2", Arc::new(ZeroDev));
    let dir_bar = dir_foo.mkdir("bar");
    dir_bar.add("f1", Arc::new(NullDev));

    test_devfs_ops(&devfs).unwrap();
    test_get_parent(&devfs).unwrap();
}
