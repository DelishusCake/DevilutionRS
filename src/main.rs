use diablo::mpq::Archive;

fn main() {
    let mpq = Archive::open("data/DIABDAT.MPQ").expect("Failed to open MPQ file");
    
    let lvl1 = mpq.get_file("levels\\l1data\\l1.min")
        .expect("Failed to get file handle");

    let mut data = vec![0x0u8; lvl1.size()];
    lvl1.read(&mpq, &mut data).expect("Failed to read file contents");
    println!("Read {:?} bytes from file", data.len());
}
