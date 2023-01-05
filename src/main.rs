use diablo::mpq::Archive;

fn main() {
    let mpq = Archive::open("data/DIABDAT.MPQ").expect("Failed to open MPQ file");

    println!("{:?}", mpq.has_file("(listfile)"));
    println!("{:?}", mpq.has_file("Levels\\L1Data\\L1.amp"));
    println!("{:?}", mpq.has_file("music\\dintro.wav"));
}
