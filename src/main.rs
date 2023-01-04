use diablo::mpq::Archive;

fn main() {
    let mpq = Archive::open("data/DIABDAT.MPQ")
        .expect("Failed to open MPQ file");

    println!("File index {:?}", mpq.get(""));
}
