use indicatif::ProgressBar;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::bitmap::BitMap;
use crate::encoding;

const HEADER_LEN: u64 = 12;

pub struct Index {
	document_count: u32,
	ngram_count: u32,
	source: BufReader<File>,
}

#[derive(Debug)]
pub enum IndexError {
	BinaryFile,
	InvalidHeader,
	UnsupportedNGramLength(u8),
	Other(Box<dyn std::error::Error>),
}

impl Display for IndexError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			IndexError::BinaryFile => write!(
				f,
				"index error: Given file was binary or used an unrecognized encoding"
			),
			IndexError::InvalidHeader => write!(f, "index error: Invalid header"),
			IndexError::UnsupportedNGramLength(len) => {
				write!(f, "index error: Invalid n-gram length {len}")
			}
			IndexError::Other(e) => write!(f, "index error: {e}"),
		}
	}
}

impl Error for IndexError {}

impl From<std::io::Error> for IndexError {
	fn from(value: std::io::Error) -> Self {
		IndexError::Other(value.into())
	}
}

impl From<std::string::FromUtf8Error> for IndexError {
	fn from(value: std::string::FromUtf8Error) -> Self {
		IndexError::Other(value.into())
	}
}

impl Index {
	pub fn bitmask_len(&self) -> u64 {
		(self.document_count as f64 / 8.0).ceil() as u64
	}

	pub fn create<P: AsRef<Path>>(path: P) -> Result<Self, IndexError> {
		// Create a list of files to index
		let spinner = ProgressBar::new_spinner().with_message("Collecting files...");
		let mut files = Vec::new();
		index_dir(Path::new("."), &mut files, &spinner).map_err(IndexError::Other)?;
		spinner.finish_with_message("Collecting files... Done.");

		// Index all files into documents
		let progress = ProgressBar::new(files.len() as u64 * 2);
		progress.println("Creating index...");

		let mut documents = Vec::with_capacity(files.len());
		for file in files {
			progress.inc(1);
			let trigrams = match index_file(&file) {
				Ok(v) => v,
				Err(e) => {
					progress.println(format!("Failed to index {}: {}", file.to_string_lossy(), e));
					continue;
				}
			};

			if trigrams.len() == 0 {
				continue;
			}

			let name = file.to_string_lossy().to_string();
			documents.push((name, trigrams));
		}

		// Order documents by filename
		documents.sort_by(|a, b| a.0.cmp(&b.0));

		// Put all documents into a search index
		let mut index = HashMap::new();
		for (i, trigrams) in documents
			.iter()
			.map(|v| v.1.iter().map(|v| v.0))
			.enumerate()
		{
			for t in trigrams {
				if !index.contains_key(&t) {
					index.insert(t, BitMap::new(documents.len()));
				}

				index.get_mut(&t).unwrap().set(i, true);
			}

			progress.inc(1);
		}

		// Order index by trigram
		let mut index = index.into_iter().collect::<Vec<([u8; 3], BitMap)>>();
		index.sort_by(|a, b| a.0.cmp(&b.0));

		progress.finish();

		let file = File::options()
			.create(true)
			.write(true)
			.truncate(true)
			.open(&path)?;

		write_index(file, documents, index).map_err(IndexError::Other)?;
		Self::load(path)
	}

	pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, IndexError> {
		let file = File::open(path)?;
		let mut reader = BufReader::new(file);
		let mut header = [0; 12];
		reader.read_exact(&mut header)?;
		if !header.starts_with(&[0x4b, 0x43, 0x53]) {
			return Err(IndexError::InvalidHeader);
		}

		if header[3] != 3 {
			return Err(IndexError::UnsupportedNGramLength(header[3]));
		}

		let mut document_count = [0; 4];
		document_count.copy_from_slice(&header[4..8]);
		let document_count = u32::from_be_bytes(document_count);

		let mut ngram_count = [0; 4];
		ngram_count.copy_from_slice(&header[8..12]);
		let ngram_count = u32::from_be_bytes(ngram_count);

		Ok(Self {
			document_count,
			ngram_count,
			source: reader,
		})
	}

	pub fn update(&mut self) -> Result<(), IndexError> {
		todo!()
	}

	pub fn find_document(
		&mut self,
		document: u32,
	) -> Result<Option<(String, Vec<([u8; 3], u32)>)>, IndexError> {
		let seek_start = HEADER_LEN + (self.bitmask_len() + 3) * self.ngram_count as u64;
		self.source.seek(SeekFrom::Start(seek_start))?;
		let mut buf = Vec::with_capacity(1024);
		for _ in 0..document {
			if self.source.read_until(0x1e, &mut buf)? == 0 {
				return Ok(None);
			}

			buf.clear();
		}

		let len = self.source.read_until(0x1f, &mut buf)?;
		if len == 0 {
			return Ok(None);
		}

		buf.pop(); // Remove 0x1f
		let document = String::from_utf8(buf)?;

		let mut buf = Vec::with_capacity(1024);
		self.source.read_until(0x1e, &mut buf)?;
		buf.pop();

		let mut trigrams = Vec::new();
		let mut trigram = [0; 3];
		let mut count = [0; 5];
		for i in (0..buf.len()).step_by(8) {
			trigram.copy_from_slice(&buf[i..i + 3]);
			count.copy_from_slice(&buf[i + 3..i + 8]);

			trigrams.push((trigram, encoding::from_ascii_compat(count)));
		}

		Ok(Some((document, trigrams)))
	}

	pub fn find_trigram(&mut self, trigram: [u8; 3]) -> Result<Option<BitMap>, IndexError> {
		let skip = self.bitmask_len() + 3;
		let seek_start = HEADER_LEN;

		// Binary search for the right trigram
		let mut rec_start = 0;
		let mut rec_end = self.ngram_count;
		let mut rec = self.ngram_count / 2 + 1;
		let mut buf = [0; 3];
		let mut bitmap_buf = vec![0; self.bitmask_len() as usize];
		while rec > rec_start && rec < rec_end {
			self.source
				.seek(SeekFrom::Start(rec as u64 * skip + seek_start))?;

			self.source.read_exact(&mut buf)?;
			match trigram.cmp(&buf) {
				std::cmp::Ordering::Less => {
					rec_end = rec;
					rec = rec_start + (rec_end - rec_start) / 2;
				}
				std::cmp::Ordering::Equal => {
					self.source.read_exact(&mut bitmap_buf)?;

					return Ok(Some(bitmap_buf.into()));
				}
				std::cmp::Ordering::Greater => {
					rec_start = rec;
					rec = rec_start + (rec_end - rec_start) / 2 + 1;
				}
			}
		}

		Ok(None)
	}
}

fn index_dir(
	path: &Path,
	files: &mut Vec<PathBuf>,
	spinner: &ProgressBar,
) -> Result<(), Box<dyn Error>> {
	spinner.tick();
	let dir = path.read_dir()?;
	for entry in dir {
		let entry = entry?;
		let file_type = entry.file_type()?;
		if file_type.is_dir() {
			index_dir(&entry.path(), files, spinner)?;
		} else {
			files.push(entry.path());
		}
	}

	Ok(())
}

fn index_file(path: &Path) -> Result<Vec<([u8; 3], u32)>, IndexError> {
	let file = File::open(path)?;
	let mut reader = BufReader::new(file);
	let mut buf = [0; 3];
	let mut trigrams = HashMap::new();
	'read: while let Ok(()) = reader.read_exact(&mut buf) {
		reader.seek_relative(-2)?;

		if !encoding::is_utf8(&buf) || !encoding::is_printable(&buf) {
			return Err(IndexError::BinaryFile);
		}

		if let Ok(s) = std::str::from_utf8(&buf) {
			let mut lower = buf;
			for (i, c) in s.char_indices() {
				if !c.is_alphanumeric() {
					continue 'read;
				}

				if c.is_ascii() {
					lower[i] = buf[i].to_ascii_lowercase();
				}
			}

			let add = !trigrams.contains_key(&lower);
			if add {
				trigrams.insert(lower, 1);
			} else {
				*trigrams.get_mut(&lower).unwrap() += 1;
			}
		}
	}

	let trigrams = trigrams.into_iter().collect();
	Ok(trigrams)
}

fn write_index<T: Write>(
	mut out: T,
	documents: Vec<(String, Vec<([u8; 3], u32)>)>,
	index: Vec<([u8; 3], BitMap)>,
) -> Result<(), Box<dyn Error>> {
	// Write header
	assert!(documents.len() <= u32::MAX as usize);
	let document_count = (documents.len() as u32).to_be_bytes();

	assert!(index.len() <= u32::MAX as usize);
	let ngram_count = (index.len() as u32).to_be_bytes();

	// Write header
	let header: [u8; HEADER_LEN as usize] = [
		// KCS
		0x4b,
		0x43,
		0x53,
		// ngram size
		0x03,
		// document count
		document_count[0],
		document_count[1],
		document_count[2],
		document_count[3],
		// ngram count
		ngram_count[0],
		ngram_count[1],
		ngram_count[2],
		ngram_count[3],
	];

	out.write_all(&header)?;

	// Write index
	let progress = ProgressBar::new((index.len() + documents.len()) as u64);
	progress.println("Writing index...");

	for (trigram, bitmap) in index {
		out.write_all(&trigram)?;
		out.write_all(&bitmap.as_bytes())?;
		progress.inc(1);
	}

	progress.finish();

	// Write documents
	for (document, trigrams) in documents {
		out.write_all(document.as_bytes())?;
		out.write_all(&[0x1f])?;
		for (trigram, count) in trigrams {
			out.write_all(&trigram)?;
			out.write_all(&encoding::to_ascii_compat(count))?;
		}

		out.write_all(&[0x1e])?;
		progress.inc(1);
	}

	progress.finish();

	Ok(())
}
