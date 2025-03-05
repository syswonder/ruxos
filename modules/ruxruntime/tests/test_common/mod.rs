/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use axio as io;
use ruxfs::{
    api::{self as fs, Directory},
    AbsPath,
};

use fs::{File, FileType, OpenOptions};
use io::{prelude::*, Error, Result};

macro_rules! assert_err {
    ($expr: expr) => {
        assert!(($expr).is_err())
    };
    ($expr: expr, $err: ident) => {
        assert_eq!(($expr).err(), Some(Error::$err))
    };
}

fn test_read_write_file() -> Result<()> {
    let fname = fs::absolute_path("///very/long//.././long//./path/./test.txt")?;
    println!("read and write file {:?}:", fname);

    // read and write
    let mut file = File::options().read(true).write(true).open(&fname)?;
    let file_size = file.get_attr()?.size();
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    print!("{}", contents);
    assert_eq!(contents.len(), file_size as usize);
    assert_eq!(file.write(b"Hello, world!\n")?, 14); // append
    drop(file);

    // read again and check
    let new_contents = fs::read_to_string(&fname)?;
    print!("{}", new_contents);
    assert_eq!(new_contents, contents + "Hello, world!\n");

    // append and check
    let mut file = OpenOptions::new().write(true).append(true).open(&fname)?;
    assert_eq!(file.write(b"new line\n")?, 9);
    drop(file);

    let new_contents2 = fs::read_to_string(&fname)?;
    print!("{}", new_contents2);
    assert_eq!(new_contents2, new_contents + "new line\n");

    // open a non-exist file
    assert_err!(File::open(&AbsPath::new("/not/exist/file")), NotFound);

    println!("test_read_write_file() OK!");
    Ok(())
}

fn test_read_dir() -> Result<()> {
    let dir = fs::absolute_path("/././//./")?;
    println!("list directory {:?}:", dir);
    for entry in Directory::open(&dir)? {
        let entry = entry?;
        println!("   {}", entry.file_name());
    }
    println!("test_read_dir() OK!");
    Ok(())
}

fn test_file_permission() -> Result<()> {
    let fname = fs::absolute_path("./short.txt")?;
    println!("test permission {:?}:", fname);

    // write a file that open with read-only mode
    let mut buf = [0; 256];
    let mut file = File::open(&fname)?;
    let n = file.read(&mut buf)?;
    assert_err!(file.write(&buf), PermissionDenied);
    drop(file);

    // read a file that open with write-only mode
    let mut file = File::create(&fname)?;
    assert_err!(file.read(&mut buf), PermissionDenied);
    assert!(file.write(&buf[..n]).is_ok());
    drop(file);

    // open with empty options
    assert_err!(OpenOptions::new().open(&fname), InvalidInput);

    // read as a directory
    assert_err!(Directory::open(&fname), NotADirectory);

    println!("test_file_permisson() OK!");
    Ok(())
}

fn test_create_file_dir() -> Result<()> {
    // create a file and test existence
    let fname = fs::absolute_path("././/very-long-dir-name/..///new-file.txt")?;
    println!("test create file {:?}:", fname);
    assert_err!(fs::get_attr(&fname), NotFound);
    let contents = "create a new file!\n";
    fs::write(&fname, contents)?;

    let dirents = Directory::open(&fs::absolute_path(".")?)?
        .map(|e| e.unwrap().file_name())
        .collect::<Vec<_>>();
    println!("dirents = {:?}", dirents);
    assert!(dirents.contains(&"new-file.txt".into()));
    assert_eq!(fs::read_to_string(&fname)?, contents);
    assert_err!(File::create_new(&fname), AlreadyExists);

    // create a directory and test existence
    let dirname = fs::absolute_path("///././/very//.//long/./new-dir")?;
    println!("test create dir {:?}:", dirname);
    assert_err!(fs::get_attr(&dirname), NotFound);
    fs::create_dir(&dirname)?;

    let dirents = Directory::open(&fs::absolute_path("./very/long")?)?
        .map(|e| e.unwrap().file_name())
        .collect::<Vec<_>>();
    println!("dirents = {:?}", dirents);
    assert!(dirents.contains(&"new-dir".into()));
    assert!(fs::get_attr(&dirname)?.is_dir());
    assert_err!(fs::create_dir(&dirname), AlreadyExists);

    println!("test_create_file_dir() OK!");
    Ok(())
}

fn test_remove_file_dir() -> Result<()> {
    // remove a file and test existence
    let fname = fs::absolute_path("//very-long-dir-name/..///new-file.txt")?;
    println!("test remove file {:?}:", fname);
    assert_err!(fs::remove_dir(&fname), NotADirectory);
    assert!(fs::remove_file(&fname).is_ok());
    assert_err!(fs::get_attr(&fname), NotFound);
    assert_err!(fs::remove_file(&fname), NotFound);

    // remove a directory and test existence
    let dirname = fs::absolute_path("very//.//long/../long/.//./new-dir////")?;
    println!("test remove dir {:?}:", dirname);
    assert_err!(fs::remove_file(&dirname), IsADirectory);
    assert!(fs::remove_dir(&dirname).is_ok());
    assert_err!(fs::get_attr(&dirname), NotFound);
    assert_err!(fs::remove_dir(&fname), NotFound);

    // error cases
    assert_err!(fs::remove_dir(&fs::absolute_path("/")?), DirectoryNotEmpty);
    assert_err!(
        fs::remove_file(&fs::absolute_path("///very/./")?),
        IsADirectory
    );
    assert_err!(
        fs::remove_dir(&fs::absolute_path("/./very///")?),
        DirectoryNotEmpty
    );

    println!("test_remove_file_dir() OK!");
    Ok(())
}

fn test_devfs_ramfs() -> Result<()> {
    const N: usize = 32;
    let mut buf = [1; N];

    // list '/' and check if /dev and /tmp exist
    let dirents = Directory::open(&fs::absolute_path("././//.//")?)?
        .map(|e| e.unwrap().file_name())
        .collect::<Vec<_>>();
    assert!(dirents.contains(&"dev".into()));
    assert!(dirents.contains(&"tmp".into()));

    // read and write /dev/null
    let mut file = File::options()
        .read(true)
        .write(true)
        .open(&fs::absolute_path("/dev/./null")?)?;
    assert_eq!(file.read_to_end(&mut Vec::new())?, 0);
    assert_eq!(file.write(&buf)?, N);
    assert_eq!(buf, [1; N]);

    // read and write /dev/zero
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&fs::absolute_path("////dev/zero")?)?;
    assert_eq!(file.read(&mut buf)?, N);
    assert!(file.write_all(&buf).is_ok());
    assert_eq!(buf, [0; N]);

    // list /dev
    let dirents = Directory::open(&fs::absolute_path("/dev")?)?
        .map(|e| e.unwrap().file_name())
        .collect::<Vec<_>>();
    assert!(dirents.contains(&"null".into()));
    assert!(dirents.contains(&"zero".into()));

    // stat /dev
    let dname = &fs::absolute_path("/dev")?;
    let dir = Directory::open(dname)?;
    let md = dir.get_attr()?;
    println!("metadata of {:?}: {:?}", dname, md);
    assert_eq!(md.file_type(), FileType::Dir);
    assert!(!md.is_file());
    assert!(md.is_dir());

    // error cases
    assert_err!(fs::create_dir(&fs::absolute_path("dev")?), AlreadyExists);
    assert_err!(
        File::create_new(&fs::absolute_path("/dev/")?),
        AlreadyExists
    );
    assert_err!(
        fs::create_dir(&fs::absolute_path("/dev/zero")?),
        AlreadyExists
    );
    assert_err!(
        fs::write(&fs::absolute_path("/dev/stdout")?, "test"),
        PermissionDenied
    );
    assert_err!(
        fs::create_dir(&fs::absolute_path("/dev/test")?),
        PermissionDenied
    );
    assert_err!(
        fs::remove_file(&fs::absolute_path("/dev/null")?),
        PermissionDenied
    );
    assert_err!(
        fs::remove_dir(&fs::absolute_path("./dev")?),
        PermissionDenied
    );

    // parent of '/dev'
    assert_eq!(
        fs::create_dir(&fs::absolute_path("///dev//..//233//")?),
        Ok(())
    );
    assert_eq!(
        fs::write(
            &fs::absolute_path(".///dev//..//233//.///test.txt")?,
            "test"
        ),
        Ok(())
    );
    assert_eq!(
        fs::remove_file(&fs::absolute_path("./dev//../..//233//.///test.txt")?),
        Ok(())
    );
    assert_err!(
        fs::remove_file(&fs::absolute_path("./dev//..//233//../233/./test.txt")?),
        NotFound
    );
    assert_err!(
        fs::remove_dir(&fs::absolute_path("very/../dev//")?),
        PermissionDenied
    );

    // tests in /tmp
    assert_eq!(
        fs::get_attr(&fs::absolute_path("tmp")?)?.file_type(),
        FileType::Dir
    );
    assert_eq!(
        fs::create_dir(&fs::absolute_path(".///tmp///././dir")?),
        Ok(())
    );
    assert_eq!(
        Directory::open(&fs::absolute_path("tmp")?).unwrap().count(),
        3
    );
    assert_eq!(
        fs::write(&fs::absolute_path(".///tmp///dir//.///test.txt")?, "test"),
        Ok(())
    );
    assert_eq!(
        fs::read(&fs::absolute_path("tmp//././/dir//.///test.txt")?),
        Ok("test".into())
    );
    assert_err!(
        fs::remove_dir(&fs::absolute_path("/tmp/dir/../dir")?),
        DirectoryNotEmpty
    );
    assert_eq!(
        fs::remove_file(&fs::absolute_path("./tmp//dir//test.txt")?),
        Ok(())
    );
    assert_eq!(
        fs::remove_dir(&fs::absolute_path("tmp/dir/.././dir///")?),
        Ok(())
    );
    assert_eq!(
        Directory::open(&fs::absolute_path("tmp")?).unwrap().count(),
        2
    );

    println!("test_devfs_ramfs() OK!");
    Ok(())
}

pub fn test_all() {
    test_read_write_file().expect("test_read_write_file() failed");
    test_read_dir().expect("test_read_dir() failed");
    test_file_permission().expect("test_file_permission() failed");
    test_create_file_dir().expect("test_create_file_dir() failed");
    test_remove_file_dir().expect("test_remove_file_dir() failed");
    test_devfs_ramfs().expect("test_devfs_ramfs() failed");
}
