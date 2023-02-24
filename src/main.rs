use crate::index::Index;
use bitmap::BitMap;
use console::style;
use search_rank::rank_file;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::process;

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

	let mut index = match Index::load("index.dat")
		.and_then(|mut i| {
			i.update()?;
			Ok(i)
		})
		.or_else(|e| {
			eprintln!("Failed to read index: {e}");
			Index::create("index.dat")
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

	let mut any = BitMap::new(index.bitmask_len() as usize);
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
			.expect("find_trigram return invalid document index");

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
