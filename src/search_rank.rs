use std::{fs, path::Path};

pub fn rank_file<P: AsRef<Path> + std::fmt::Debug>(
	path: P,
	search_terms: &[String],
	trigrams: &[[u8; 3]],
	previews: &mut Vec<(usize, String)>,
) -> std::io::Result<usize> {
	let contents = fs::read_to_string(&path)?.to_lowercase();
	let mut rank = 0;
	let mut preview_buf = Vec::new();

	// Check if the file contains our exact phrase
	let mut terms = search_terms.iter();
	if let Some(start) = contents.find(terms.next().expect("search_terms cannot be empty")) {
		let mut search_str = contents[start..].trim();
		if terms.all(|term| {
			if search_str.starts_with(term) {
				search_str = search_str[term.len()..].trim();
				true
			} else {
				false
			}
		}) {
			let len = search_terms.iter().fold(0, |v, term| v + term.len());
			rank += len * 100;
			preview_buf.push(get_preview(&contents, &contents[start..start + len]));
		}
	}

	// Check for individual terms
	search_terms.iter().for_each(|term| {
		if contents.contains(term) {
			rank += term.len() * 10;
			preview_buf.push(get_preview(&contents, term));
		}
	});

	// Check for individual trigrams
	trigrams
		.iter()
		.map(|tri| std::str::from_utf8(tri).unwrap())
		.for_each(|tri| {
			if contents.contains(tri) {
				rank += 1;
				preview_buf.push(get_preview(&contents, tri));
			}
		});

	preview_buf.sort_by(|a, b| a.0.cmp(&b.0));
	preview_buf.into_iter().for_each(|prev| {
		if !previews.contains(&prev) {
			previews.push(prev);
		}
	});

	Ok(rank)
}

fn get_preview(source: &str, search: &str) -> (usize, String) {
	for (i, line) in source.lines().enumerate() {
		if line.contains(search) {
			let trimmed = line.trim();
			return (i + 1, trimmed[..50.min(trimmed.len())].to_string());
		}
	}

	unreachable!()
}
