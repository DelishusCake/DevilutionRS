use diablo::mpq::Mpq;

fn main() {
    let _mpq = Mpq::open("data/DIABDAT.MPQ")
        .expect("Failed to open MPQ file");
}
