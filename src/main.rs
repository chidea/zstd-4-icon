use zstd_4_icon::prelude::*;

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let len = args.len();
    if len == 3 {
        compress(&args[1], &args[2])?;
    } else if len == 4 {
        let mut stdout = std::io::stdout();
        decompress(&mut stdout, &args[1], &args[2], &args[3])?;
    } else {        
        println!("argument to compress : <archive_file> <dictionary_file>");
        println!("argument to decompress : <archive_file> <dictionary_file> <decompress_path>");
        panic!("argument format mismatch");
    }

    Ok(())
}
