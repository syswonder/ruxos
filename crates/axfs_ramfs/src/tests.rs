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

fn test_ramfs_ops(devfs: &RamFileSystem) -> VfsResult {
    const N: usize = 32;
    const N_HALF: usize = N / 2;
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

    let node = root.lookup(&RelPath::new_canonicalized("////f1"))?;
    assert_eq!(node.get_attr()?.file_type(), VfsNodeType::File);
    assert!(!node.get_attr()?.is_dir());
    assert_eq!(node.get_attr()?.size(), 0);
    assert_eq!(node.read_at(0, &mut buf)?, 0);
    assert_eq!(buf, [1; N]);

    assert_eq!(node.write_at(N_HALF as _, &buf[..N_HALF])?, N_HALF);
    assert_eq!(node.read_at(0, &mut buf)?, N);
    assert_eq!(buf[..N_HALF], [0; N_HALF]);
    assert_eq!(buf[N_HALF..], [1; N_HALF]);
    assert_eq!(
        node.lookup(&RelPath::new_canonicalized("/")).err(),
        Some(VfsError::NotADirectory)
    );

    let foo = devfs
        .root_dir()
        .lookup(&RelPath::new_canonicalized(".///.//././/.////foo"))?;
    assert!(foo.get_attr()?.is_dir());
    assert_eq!(
        foo.read_at(10, &mut buf).err(),
        Some(VfsError::IsADirectory)
    );
    assert!(Arc::ptr_eq(
        &foo.clone().lookup(&RelPath::new_canonicalized("/f3"))?,
        &devfs
            .root_dir()
            .lookup(&RelPath::new_canonicalized(".//./foo///f3"))?,
    ));
    assert_eq!(
        foo.clone()
            .lookup(&RelPath::new_canonicalized("/bar//f4"))?
            .get_attr()?
            .file_type(),
        VfsNodeType::File
    );
    assert_eq!(
        foo.lookup(&RelPath::new_canonicalized("/bar///"))?
            .get_attr()?
            .file_type(),
        VfsNodeType::Dir
    );

    Ok(())
}

fn test_get_parent(devfs: &RamFileSystem) -> VfsResult {
    let root = devfs.root_dir();
    assert!(root.parent().is_none());

    let node = root.clone().lookup(&RelPath::new_canonicalized("f1"))?;
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
            .lookup(&RelPath::new_canonicalized("///foo//bar///../f3"))?,
        &root.lookup(&RelPath::new_canonicalized("foo/.//f3"))?,
    ));

    Ok(())
}

#[test]
fn test_ramfs() {
    // .
    // ├── foo
    // │   ├── bar
    // │   │   └── f4
    // │   └── f3
    // ├── f1
    // └── f2

    let ramfs = RamFileSystem::new();
    let root = ramfs.root_dir();
    root.create(&RelPath::new_canonicalized("f1"), VfsNodeType::File)
        .unwrap();
    root.create(&RelPath::new_canonicalized("f2"), VfsNodeType::File)
        .unwrap();
    root.create(&RelPath::new_canonicalized("foo"), VfsNodeType::Dir)
        .unwrap();

    let dir_foo = root.lookup(&RelPath::new_canonicalized("foo")).unwrap();
    dir_foo
        .create(&RelPath::new_canonicalized("f3"), VfsNodeType::File)
        .unwrap();
    dir_foo
        .create(&RelPath::new_canonicalized("bar"), VfsNodeType::Dir)
        .unwrap();

    let dir_bar = dir_foo.lookup(&RelPath::new_canonicalized("bar")).unwrap();
    dir_bar
        .create(&RelPath::new_canonicalized("f4"), VfsNodeType::File)
        .unwrap();

    let mut entries = ramfs.root_dir_node().get_entries();
    entries.sort();
    assert_eq!(entries, ["f1", "f2", "foo"]);

    test_ramfs_ops(&ramfs).unwrap();
    test_get_parent(&ramfs).unwrap();

    let root = ramfs.root_dir();
    assert_eq!(root.unlink(&RelPath::new_canonicalized("f1")), Ok(()));
    assert_eq!(root.unlink(&RelPath::new_canonicalized("//f2")), Ok(()));
    assert_eq!(
        root.unlink(&RelPath::new_canonicalized("f3")).err(),
        Some(VfsError::NotFound)
    );
    assert_eq!(
        root.unlink(&RelPath::new_canonicalized("foo")).err(),
        Some(VfsError::DirectoryNotEmpty)
    );
    assert_eq!(
        root.unlink(&RelPath::new_canonicalized("foo/..")).err(),
        Some(VfsError::InvalidInput)
    );
    assert_eq!(
        root.unlink(&RelPath::new_canonicalized("foo/./bar")).err(),
        Some(VfsError::DirectoryNotEmpty)
    );
    assert_eq!(
        root.unlink(&RelPath::new_canonicalized("foo/bar/f4")),
        Ok(())
    );
    assert_eq!(root.unlink(&RelPath::new_canonicalized("foo/bar")), Ok(()));
    assert_eq!(
        root.unlink(&RelPath::new_canonicalized("./foo//.//f3")),
        Ok(())
    );
    assert_eq!(root.unlink(&RelPath::new_canonicalized("./foo")), Ok(()));
    assert!(ramfs.root_dir_node().get_entries().is_empty());
}
