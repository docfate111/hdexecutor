use hdrepresentation::Program;
use hdexecutor::exec;
use std::env;
use std::io::{Error, ErrorKind};

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!(
            "Usage: {} [deserialized program] [filesystem image] [filesystem type(i.e. ext4, btrfs)]",
            &args[0]
        );
        return Err(Error::new(ErrorKind::Other, "invalid arguments"));
    }
    let f = Program::from_path(&args[1]);
    exec(&f, args[2].clone(), args[3].clone())?;
    Ok(())
}
