mod archive;
mod compression;
mod crypto;
mod header;

pub use archive::*;

#[cfg(test)]
mod tests {
	use super::*;

	const ARCHIVE_PATH: &str = "../DATA/DIABDAT.MPQ";

	#[test]
	fn test_explode_cel() {
		open_file("Data\\Square.CEL").expect("Failed to open file");
	}

	#[test]
	fn test_explode_lvl() {
		open_file("Levels\\TownData\\Town.TIL").expect("Failed to open file");
	}

	#[test]
	fn test_explode_dun() {
		open_file("Levels\\TownData\\Sector1s.DUN").expect("Failed to open file");
	}
	
	fn open_file(filename: &str) -> std::io::Result<Vec<u8>> {
		let archive = Archive::open(ARCHIVE_PATH)?;
		let file = archive.get_file(filename)?;
		let mut bytes = vec![0x0u8; file.size()];
		file.read(&mut bytes)?;
		Ok(bytes)
	}
}
