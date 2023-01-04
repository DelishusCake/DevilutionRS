use diablo::mpq::Archive;

fn main() {
    let mpq = Archive::open("data/DIABDAT.MPQ").expect("Failed to open MPQ file");

    println!("{:?}", mpq.file_size("(listfile)"));
    println!("{:?}", mpq.file_size("Levels\\L1Data\\L1.amp"));
    println!("{:?}", mpq.file_size("music\\dintro.wav"));
}
