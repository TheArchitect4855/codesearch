use std::{fs, path::Path};

pub fn rank_file<P: AsRef<Path> + std::fmt::Debug>(
	path: P,
	search_terms: &[String],
	trigrams: &[[u8; 3]],
	previews: &mut Vec<String>,
) -> std::io::Result<usize> {
	let contents = fs::read_to_string(&path)?;
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
			preview_buf.push(get_preview(&contents, (start, start + len)).to_string());
		}
	}

	// Check for individual terms
	search_terms.iter().for_each(|term| {
		if let Some(i) = contents.find(term) {
			rank += term.len() * 10;
			preview_buf.push(get_preview(&contents, (i, i + term.len())).to_string());
		}
	});

	// Check for individual trigrams
	trigrams
		.iter()
		.map(|tri| std::str::from_utf8(tri).unwrap())
		.for_each(|tri| {
			if let Some(i) = contents.find(tri) {
				rank += 1;
				preview_buf.push(get_preview(&contents, (i, i + tri.len())).to_string());
			}
		});

	preview_buf.into_iter().for_each(|prev| {
		if !previews.contains(&prev) {
			previews.push(prev);
		}
	});

	Ok(rank)
}

fn get_preview(source: &str, target: (usize, usize)) -> &str {
	if target.1 - target.0 >= 50 {
		return &source[target.0..target.0 + 50].trim();
	}

	let mut start = target.0;
	while start > 0 {
		if source[start - 1..].starts_with('\n') {
			break;
		}

		start -= 1;
	}

	let mut end = target.1;
	if end - start >= 50 {
		let start = end - 25;
		let end = start + 50;
		return &source[start..end].trim();
	}

	while end < source.len() - 1 {
		if source[end + 1..].starts_with('\n') {
			break;
		} else if end - start == 50 {
			break;
		}

		end += 1;
	}

	source[start..end].trim()
}
