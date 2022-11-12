use hdrepresentation::*;
use std::env;
use penguincrab::*;
use std::io::{Error, ErrorKind};

pub fn exec(p: &Program, image_path: String) -> Result<(), std::io::Error> {
        let new_path = String::from(image_path.clone());
        let server = LklSetup::new(LklSetupArgs {
            filename: new_path.clone(),
            boot_settings: Some(String::from("mem=128M\0")),
            partition_num: None,
            filesystem_type: None,
            filesystem_options: None,
            on_panic: None,
	    print: None,
        })
        .unwrap();
        for sys in p.syscalls.iter() {
            match exec_syscall(p, sys, server.mount_point.clone()) {
                Ok(ret_val) => {
                    if ret_val < 0 {
                        print_error(&(ret_val as i32));
                    }
                    println!("{} {}", sys.nr, ret_val);
                }
                Err(e) => {
                    eprintln!("{e}");
                }
            }
        }
        for fd in p.active_fds.iter() {
            lkl_sys_close(*fd as i32);
        }
        Ok(())
}

pub fn exec_syscall(prog: &Program, syscall: &Syscall, mount_point: String) -> Result<i64, String> {
    let mut vars = Vec::<VariableType>::new();
    for arg in syscall.args.iter() {
        if arg.is_variable {
            vars.push(
                *prog
                    .variables
                    .get(arg.index.expect("exec_syscall failed to unwrap index"))
                    .expect("exec_syscall failed to find anything at the provided index")
                    .var_type
                    .clone(),
            );
        } else {
            vars.push(VariableType::Long(
                arg.value.expect("exec syscall failed to unwrap value"),
            ));
        }
    }
    match syscall.nr {
        SysNo::Open => {
            let path = var_to_path(&vars[0], &mount_point)?;
            let flags = var_to_u32(&vars[1])?;
            let mode = var_to_u32(&vars[2])?;
            return Ok(lkl_sys_open(to_cstr(&path).unwrap(), flags, mode));
        }
        SysNo::Read => {
            let fd = var_to_i32(&vars[0])?;
            let mut buf = var_to_vec(&vars[1])?;
            let count = var_to_usize(&vars[2])?;
            return Ok(lkl_sys_read(fd, &mut buf[..], count));
        }
        SysNo::Write => {
            let fd = var_to_i32(&vars[0])?;
            let buf = var_to_str(&vars[1])?;
            let count = var_to_usize(&vars[2])?;
            return Ok(lkl_sys_write(fd, buf.as_bytes(), count));
        }
        SysNo::Lseek => {
            let fd = var_to_i32(&vars[0])?;
            let offset = var_to_u32(&vars[1])?;
            let origin = var_to_u32(&vars[2])?;
            return Ok(lkl_sys_lseek(fd, offset, origin));
        }
        SysNo::Getdents => {
            let fd = var_to_i32(&vars[0])?;
            let mut dirent = dirent64::default();
            let count = var_to_usize(&vars[2])?;
            return Ok(lkl_sys_getdents64(fd, &mut dirent, count));
        }
        SysNo::Pread => {
            let fd = var_to_i32(&vars[0])?;
            let mut buf = var_to_vec(&vars[1])?;
            let count = var_to_usize(&vars[2])?;
            let off = var_to_u64(&vars[3])?;
            return Ok(lkl_sys_pread64(fd, &mut buf[..], count, off));
        }
        SysNo::Pwrite => {
            let fd = var_to_i32(&vars[0])?;
            let buf = var_to_vec(&vars[1])?;
            let count = var_to_usize(&vars[2])?;
            let off = var_to_u64(&vars[3])?;
            return Ok(lkl_sys_pwrite64(fd, &buf[..], count, off));
        }
        SysNo::Fstat => {
            let fd = var_to_i32(&vars[0])?;
            let mut stat = stat::default();
            return Ok(lkl_sys_fstat(fd, &mut stat));
        }
        SysNo::Rename => {
            let path = var_to_path(&vars[0], &mount_point)?;
            let newpath = var_to_path(&vars[1], &mount_point)?;
            return Ok(lkl_sys_rename(
                to_cstr(&path).unwrap(),
                to_cstr(&newpath).unwrap(),
            ));
        }
        SysNo::Fsync => {
            let fd = var_to_i32(&vars[0])?;
            return Ok(lkl_sys_fsync(fd));
        }
        SysNo::Fdatasync => Ok(lkl_sys_fdatasync(var_to_i32(&vars[0])?)),
        SysNo::Syncfs => Ok(lkl_sys_syncfs(var_to_i32(&vars[0])?)),
        SysNo::Sendfile => {
            let mut v = Vec::new();
            v.push(var_to_i32(&vars[2])? as u8);
            Ok(lkl_sys_sendfile(
                var_to_i32(&vars[0])?,
                var_to_i32(&vars[1])?,
                &mut v[..],
                var_to_usize(&vars[3])?,
            ))
        }
        SysNo::Access => Ok(lkl_sys_access(
            to_cstr(&var_to_path(&vars[0], &mount_point)?).unwrap(),
            var_to_i32(&vars[1])?,
        )),
        SysNo::Ftruncate => Ok(lkl_sys_ftruncate(
            var_to_i32(&vars[0])?,
            var_to_u64(&vars[1])?,
        )),
        SysNo::Truncate => Ok(lkl_sys_truncate(
            to_cstr(&var_to_path(&vars[0], &mount_point)?).unwrap(),
            var_to_i64(&vars[1])?,
        )),
        SysNo::Mkdir => Ok(lkl_sys_mkdir(
            to_cstr(&var_to_path(&vars[0], &mount_point)?).unwrap(),
            var_to_u32(&vars[1])?,
        )),
        SysNo::Rmdir => Ok(lkl_sys_rmdir(
            to_cstr(&var_to_path(&vars[0], &mount_point)?).unwrap(),
        )),
        SysNo::Link => Ok(lkl_sys_link(
            to_cstr(&var_to_path(&vars[0], &mount_point)?).unwrap(),
            to_cstr(&var_to_path(&vars[1], &mount_point)?).unwrap(),
        )),
        SysNo::Unlink => Ok(lkl_sys_unlink(
            to_cstr(&var_to_path(&vars[0], &mount_point)?).unwrap(),
        )),
        SysNo::Symlink => Ok(lkl_sys_symlink(
            to_cstr(&var_to_path(&vars[0], &mount_point)?).unwrap(),
            to_cstr(&var_to_path(&vars[1], &mount_point)?).unwrap(),
        )),
        SysNo::Setxattr => Ok(lkl_sys_setxattr(
            to_cstr(&var_to_path(&vars[0], &mount_point)?).unwrap(),
            to_cstr(&var_to_str(&vars[1])?).unwrap(),
            &var_to_vec(&vars[2])?[..],
            var_to_usize(&vars[3])?,
            var_to_u32(&vars[4])?,
        )),
        SysNo::Listxattr => Ok(lkl_sys_listxattr(
            to_cstr(&var_to_path(&vars[0], &mount_point)?).unwrap(),
            &mut var_to_vec(&vars[1])?[..],
        )),
        SysNo::Removexattr => Ok(lkl_sys_removexattr(
            to_cstr(&var_to_path(&vars[0], &mount_point)?).unwrap(),
            to_cstr(&var_to_str(&vars[1])?).unwrap(),
        )),
        _ => {
            return Err("not implemented".to_string());
        }
    }
}

fn var_to_str(v: &VariableType) -> Result<String, String> {
    match v {
        VariableType::Str(s) => Ok(s.to_string()),
        _ => Err(format!("failed to convert {:?} to str", v)),
    }
}

//if anyone can make macro for this make PR
fn var_to_u32(v: &VariableType) -> Result<u32, String> {
    match v {
        VariableType::Long(n) => Ok(*n as u32),
        _ => Err(format!("failed to convert {:?} to u32", v)),
    }
}

fn var_to_u64(v: &VariableType) -> Result<u64, String> {
    match v {
        VariableType::Long(n) => Ok(*n as u64),
        _ => Err(format!("failed to convert {:?} to u64", v)),
    }
}

fn var_to_i64(v: &VariableType) -> Result<i64, String> {
    match v {
        VariableType::Long(n) => Ok(*n as i64),
        _ => Err(format!("invalid i64 {:?}", v)),
    }
}

fn var_to_usize(v: &VariableType) -> Result<usize, String> {
    match v {
        VariableType::Long(n) => Ok(*n as usize),
        _ => {
            eprintln!("invalid usize {:?}", v);
            return Err(format!("invalid usize {:?}", v));
        }
    }
}

fn var_to_i32(v: &VariableType) -> Result<i32, String> {
    match v {
        VariableType::Long(n) => Ok(*n as i32),
        _ => Err(format!("invalid i32 {:?}", v)),
    }
}

fn var_to_path(v: &VariableType, mount_point: &str) -> Result<String, String> {
    match v {
        VariableType::Str(s) => {
            let mut mpoint = String::from(mount_point);
            mpoint.push_str(&s);
            mpoint.push_str("\0");
            Ok(mpoint)
        }
        _ => Err(format!("invalid path {:?}", v)),
    }
}

fn var_to_vec(v: &VariableType) -> Result<Vec<u8>, String> {
    match v {
        VariableType::UCharPtr(bytes, size) => match bytes {
            None => Ok(vec![0; *size as usize]),
            Some(buf) => Ok(buf.clone()),
        },
        _ => Err(format!("invalid vector {:?}", v)),
    }
}

fn main() -> Result<(), std::io::Error> {
	let args: Vec<String> = env::args().collect();
    	if args.len() != 3 {
		eprintln!("Usage: {} [deserialized program] [filesystem image]", &args[0]);
		return Err(Error::new(ErrorKind::Other, "invalid arguments"));
	}
	let f = Program::from_path(&args[1]);
	exec(&f, args[2].clone())?;
	Ok(())
}
