use crate::index::Index;
use bitmap::BitMap;
use console::style;
use search_rank::rank_file;
use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process;
use std::{env, fs};

mod bitmap;
mod encoding;
mod index;
mod search_rank;

fn main() {
	let mut args = env::args();
	let name = args.next();
	let search_term = args.collect::<Vec<String>>();
	if search_term.len() == 0 {
		show_help(name.as_deref());
	}

	let save_path = match get_save_path() {
		Ok(v) => v,
		Err(e) => {
			eprintln!("Failed to get save location: {e}");
			process::exit(1);
		}
	};

	let mut index = match Index::load(&save_path)
		.and_then(|mut i| {
			i.update()?;
			Ok(i)
		})
		.or_else(|e| {
			eprintln!("Failed to read index: {e}");
			Index::create(&save_path)
		}) {
		Ok(i) => i,
		Err(e) => {
			eprintln!("Index creation failed: {e}");
			process::exit(1);
		}
	};

	let results = match search(&mut index, search_term) {
		Ok(v) => v,
		Err(e) => {
			eprintln!("Search failed: {e}");
			process::exit(1);
		}
	};

	results[..usize::min(5, results.len())]
		.into_iter()
		.for_each(|(file, rank, previews)| {
			println!("{} ({})", style(file.to_string_lossy()).bold(), rank);
			previews
				.into_iter()
				.for_each(|(line, prev)| println!("{}\t{prev}", style(line).bold()));
		});
}

fn get_save_path() -> Result<PathBuf, String> {
	let mut path = home::home_dir().ok_or(String::from("Could not get home dir"))?;
	path.push(".thearchitect");
	path.push("codesearch");
	if !path.exists() {
		fs::create_dir_all(&path).map_err(|e| e.to_string())?;
	}

	let cwd = env::current_dir().map_err(|e| e.to_string())?;
	let cwd = encoding::os_str_to_bytes(cwd.as_os_str());
	let hash = hmac_sha256::Hash::hash(&cwd);
	let file_name = encoding::to_hex(&hash);
	path.push(file_name);

	Ok(path)
}

fn get_trigrams(bytes: &[u8], buf: &mut Vec<[u8; 3]>) {
	if bytes.len() < 3 {
		return;
	}

	let mut tri_buf = [0; 3];
	'outer: for i in 0..=bytes.len() - 3 {
		tri_buf.copy_from_slice(&bytes[i..i + 3]);
		for b in tri_buf.iter_mut() {
			if !b.is_ascii_alphanumeric() {
				continue 'outer;
			}

			if b.is_ascii() {
				*b = b.to_ascii_lowercase();
			}
		}

		buf.push(tri_buf);
	}
}

fn search(
	index: &mut Index,
	terms: Vec<String>,
) -> Result<Vec<(OsString, usize, Vec<(usize, String)>)>, Box<dyn Error>> {
	let mut trigrams = Vec::new();
	terms
		.iter()
		.for_each(|t| get_trigrams(t.as_bytes(), &mut trigrams));

	let mut any = BitMap::new(index.bitmap_len() as usize);
	for t in &trigrams {
		if let Some(v) = index.find_trigram(*t)? {
			any |= &v;
		}
	}

	let mut documents = Vec::new();
	for (doc, bit) in any.into_iter().enumerate() {
		if !bit {
			continue;
		}

		let doc = index
			.find_document(doc as u32)?
			.expect("find_trigram returned invalid document index");

		let mut preview_buf = Vec::new();
		let rank = rank_file(&doc, &terms, &trigrams, &mut preview_buf)?;
		documents.push((doc, rank, preview_buf));
	}

	documents.sort_by(|a, b| b.1.cmp(&a.1));
	Ok(documents)
}

fn show_help(name: Option<&str>) {
	println!("Usage: {} [search term]", name.unwrap_or("codesearch"));
	process::exit(1);
}
