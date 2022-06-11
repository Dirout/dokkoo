/*
	This file is part of Dokkoo.

	Dokkoo is free software: you can redistribute it and/or modify
	it under the terms of the GNU Affero General Public License as published by
	the Free Software Foundation, either version 3 of the License, or
	(at your option) any later version.

	Dokkoo is distributed in the hope that it will be useful,
	but WITHOUT ANY WARRANTY; without even the implied warranty of
	MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
	GNU Affero General Public License for more details.

	You should have received a copy of the GNU Affero General Public License
	along with Dokkoo.  If not, see <https://www.gnu.org/licenses/>.
*/
/*
lib.rs - Handling Mokk Files (.mokkf)
File:
	Term for a document or page written in accordance to the Mokk specification
*/
use anyhow::Context;
use chrono::DateTime;
use derive_more::{Constructor, Div, Error, From, Into, Mul, Rem, Shl, Shr};
use liquid::*;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::{collections::HashMap, fmt};

#[derive(
	Eq,
	PartialEq,
	PartialOrd,
	Clone,
	Default,
	Debug,
	Serialize,
	Deserialize,
	From,
	Into,
	Error,
	Mul,
	Div,
	Rem,
	Shr,
	Shl,
	Constructor,
)]
/// A File's date-time metadata
pub struct Date {
	/// Year with four digits
	pub year: String,
	/// Year without the century (00..99)
	pub short_year: String,
	/// Month (01..12)
	pub month: String,
	/// Month without leading zeros
	pub i_month: String,
	/// Three-letter month abbreviation, e.g. “Jan”
	pub short_month: String,
	/// Full month name, e.g. “January”
	pub long_month: String,
	/// Day of the month (01..31)
	pub day: String,
	/// Day of the month without leading zeros
	pub i_day: String,
	/// Ordinal day of the year, with leading zeros. (001..366)
	pub y_day: String,
	/// Week year which may differ from the month year for up to three days at the start of January and end of December
	pub w_year: String,
	/// Week number of the current year, starting with the first week having a majority of its days in January (01..53)
	pub week: String,
	/// Day of the week, starting with Monday (1..7)
	pub w_day: String,
	/// Three-letter weekday abbreviation, e.g. “Sun”
	pub short_day: String,
	/// Weekday name, e.g. “Sunday”
	pub long_day: String,
	/// Hour of the day, 24-hour clock, zero-padded (00..23)
	pub hour: String,
	/// Minute of the hour (00..59)
	pub minute: String,
	/// Second of the minute (00..59)
	pub second: String,
	/// A File's date-time metadata, formatted per the RFC 3339 standard
	pub rfc_3339: String,
	/// A File's date-time metadata, formatted per the RFC 2822 standard
	pub rfc_2822: String,
}

/// Handle conversion of a Date object into a string of characters
impl fmt::Display for Date {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.rfc_3339)
	}
}

#[derive(
	Eq,
	PartialEq,
	Clone,
	Default,
	Debug,
	Serialize,
	Deserialize,
	From,
	Into,
	Error,
	Mul,
	Div,
	Rem,
	Shr,
	Shl,
	Constructor,
)]
/// Generated data regarding a Mokk File
pub struct Page {
	/// A File's contextual data, represented as YAML at the head/front of the file
	pub data: HashMap<String, serde_yaml::Value>,
	/// A File's contents following the frontmatter
	pub content: String,
	/// Data representing the output path of a File.
	/// This is defined in a File's frontmatter
	pub permalink: String,
	/// A File's date-time metadata, formatted per the RFC 3339 standard.
	/// This is defined in a File's frontmatter
	pub date: Date,
	/// Path to the File, not including the File itself
	pub directory: String,
	/// The File's base filename
	pub name: String,
	/// The output path of a file; a processed `permalink` value
	pub url: String,
	/// Whether a File is intended to be marked-up in Markdown (intended for when a different markup language [HTML, XML, et cetera], or none at all, is more appropriate)
	pub markdown: bool,
}

/// Handle conversion of a Page object into a string of characters
impl fmt::Display for Page {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:#?}", self)
	}
}

/// Returns an expanded permalink value, for when shorthand is used
///
/// # Arguments
///
/// * `permalink` - A string slice that represents the permalink value specified in the File
///
/// # Shorthand
///
/// * `date` → `/{{ page.data.collection }}/{{ page.date.year }}/{{ page.date.month }}/{{ page.date.day }}/{{ page.data.title }}.html`
///
/// * `pretty` → `/{{ page.data.collection }}/{{ page.date.year }}/{{ page.date.month }}/{{ page.date.day }}/{{ page.data.title }}/index.html`
///
/// * `ordinal` → `/{{ page.data.collection }}/{{ page.date.year }}/{{ page.date.y_day }}/{{ page.data.title }}.html`
///
/// * `weekdate` → `/{{ page.data.collection }}/{{ page.date.year }}/W{{ page.date.week }}/{{ page.date.short_day }}/{{ page.data.title }}.html`
///
/// * `none` → `/{{ page.data.collection }}/{{ page.data.title }}.html`
#[inline(always)]
pub fn get_permalink(permalink: &str) -> String {
	match permalink {
        "date" => {
            "/{{ page.data.collection }}/{{ page.date.year }}/{{ page.date.month }}/{{ page.date.day }}/{{ page.data.title }}.html".to_owned()
        }
        "pretty" => {
            "/{{ page.data.collection }}/{{ page.date.year }}/{{ page.date.month }}/{{ page.date.day }}/{{ page.data.title }}/index.html".to_owned()
        }
        "ordinal" => {
            "/{{ page.data.collection }}/{{ page.date.year }}/{{ page.date.y_day }}/{{ page.data.title }}.html"
                .to_owned()
        }
        "weekdate" => {
            "/{{ page.data.collection }}/{{ page.date.year }}/W{{ page.date.week }}/{{ page.date.short_day }}/{{ page.data.title }}.html".to_owned()
        }
        "none" => {
            "/{{ page.data.collection }}/{{ page.data.title }}.html".to_owned()
        }
        _ => {
            permalink.to_string()
        }
    }
}

/// Returns a tuple with a File's frontmatter and contents, in that order
///
/// # Arguments
///
/// * `page_text` - The `.mokkf` file's data as a `String`
#[inline(always)]
pub fn split_frontmatter(page_text: String) -> (String, String) {
	let mut begin = false;
	let mut end = false;
	let mut frontmatter = String::new();
	let mut contents = String::new();

	for line in page_text.lines() {
		if !begin && line == "---" {
			begin = true;
		} else if begin && line == "---" && !end {
			end = true;
		} else if begin && !end {
			//frontmatter.push_str(&format!("{}\n", &line));
			writeln!(frontmatter, "{}", &line).unwrap();
		} else {
			//contents.push_str(&format!("{}\n", &line));
			writeln!(contents, "{}", &line).unwrap();
		}
	}

	if frontmatter.trim().is_empty() {
		frontmatter = "empty: true".to_owned();
	}

	(frontmatter, contents)
}

/// Returns an object with a Page's context
///
/// # Arguments
///
/// * `page_path` - The `.mokkf` file's path as a `String`
///
/// * `conditions` - Prints conditions information
#[inline(always)]
pub fn get_page_object(page_path: String, collections: &HashMap<String, Vec<Page>>) -> Page {
	// Define variables which we'll use to create our Document, which we'll use to generate the Page context
	let split_page = split_frontmatter(fs::read_to_string(&page_path).unwrap()); // See file::split_frontmatter
	let frontmatter: HashMap<String, serde_yaml::Value> =
		serde_yaml::from_str(&split_page.0).unwrap(); // Parse frontmatter as HashMap (collection of key-value pairs)
	let permalink = frontmatter.get("permalink"); // Get the key-value pair of the 'permalink' key from the frontmatter
	let date = frontmatter.get("date"); // Get the key-value pair of the 'date' key from the frontmatter
	let markdown = frontmatter.get("markdown"); // Get the key-value pair of the 'markdown' key from the frontmatter

	let permalink_string: String = match permalink {
		Some(_) => permalink.unwrap().as_str().unwrap().to_string(),
		None => String::new(),
	};

	let markdown_bool: bool = match markdown {
		Some(_) => markdown.unwrap().as_bool().unwrap(),
		None => true,
	};

	let date_object = match date {
		Some(_) => {
			let datetime = DateTime::parse_from_rfc3339(date.unwrap().as_str().unwrap()); // Turn the date-time into a DateTime object for easy manipulation (to generate temporal Page metadata)
			let global_file = fs::read_to_string("./_global.yml");

			let global: HashMap<String, serde_yaml::Value> = match global_file {
				Ok(_) => {
					serde_yaml::from_str(&global_file.unwrap()).unwrap()
					// Defined as variable as it required a type annotation
				}
				Err(_) => {
					serde_yaml::from_str("locale: \"en_US\"").unwrap()
					// Defined as variable as it required a type annotation
				}
			};

			let locale_key = global.get("locale");

			let locale_value = match locale_key {
				Some(_) => locale_key.unwrap().as_str().unwrap(),
				None => "en_US",
			};

			let locale: chrono::Locale = chrono::Locale::try_from(locale_value).unwrap(); // Get locale from Global context

			Date {
				year: format!("{}", datetime.unwrap().format_localized("%Y", locale)),
				short_year: format!("{}", datetime.unwrap().format_localized("%y", locale)),
				month: format!("{}", datetime.unwrap().format_localized("%m", locale)),
				i_month: format!("{}", datetime.unwrap().format_localized("%-m", locale)),
				short_month: format!("{}", datetime.unwrap().format_localized("%b", locale)),
				long_month: format!("{}", datetime.unwrap().format_localized("%B", locale)),
				day: format!("{}", datetime.unwrap().format_localized("%d", locale)),
				i_day: format!("{}", datetime.unwrap().format_localized("%-d", locale)),
				y_day: format!("{}", datetime.unwrap().format_localized("%j", locale)),
				w_year: format!("{}", datetime.unwrap().format_localized("%G", locale)),
				week: format!("{}", datetime.unwrap().format_localized("%U", locale)),
				w_day: format!("{}", datetime.unwrap().format_localized("%u", locale)),
				short_day: format!("{}", datetime.unwrap().format_localized("%a", locale)),
				long_day: format!("{}", datetime.unwrap().format_localized("%A", locale)),
				hour: format!("{}", datetime.unwrap().format_localized("%H", locale)),
				minute: format!("{}", datetime.unwrap().format_localized("%M", locale)),
				second: format!("{}", datetime.unwrap().format_localized("%S", locale)),
				rfc_3339: datetime.unwrap().to_rfc3339(),
				rfc_2822: datetime.unwrap().to_rfc2822(),
			}
		}
		None => Date {
			year: String::new(),
			short_year: String::new(),
			month: String::new(),
			i_month: String::new(),
			short_month: String::new(),
			long_month: String::new(),
			day: String::new(),
			i_day: String::new(),
			y_day: String::new(),
			w_year: String::new(),
			week: String::new(),
			w_day: String::new(),
			short_day: String::new(),
			long_day: String::new(),
			hour: String::new(),
			minute: String::new(),
			second: String::new(),
			rfc_3339: String::new(),
			rfc_2822: String::new(),
		},
	};

	let page_path_io = Path::new(&page_path[..]); // Turn the path into a Path object for easy manipulation (to get page.directory and page.name)

	// Define our Page
	let mut page = Page {
		data: serde_yaml::from_str(&split_page.0).unwrap(),
		content: split_page.1,
		permalink: permalink_string,
		date: date_object,
		directory: page_path_io.parent().unwrap().to_str().unwrap().to_owned(),
		name: page_path_io
			.file_stem()
			.unwrap()
			.to_str()
			.unwrap()
			.to_owned(),
		url: String::new(),
		markdown: markdown_bool,
	};

	match &page.permalink[..] {
		"" => {}
		_ => {
			// Render the URL once the Page metadata has been generated
			page.url = render(
				&page,
				&get_permalink(permalink.unwrap().as_str().unwrap()),
				true,
				collections,
			);
		}
	}

	page
}

/// Returns a Liquid object with a Page's Liquid contexts
///
/// # Arguments
///
/// * `page` - The `.mokkf` file's context as a Page
///
/// * `conditions` - Prints conditions information
#[inline(always)]
pub fn get_contexts(page: &Page, collections: &HashMap<String, Vec<Page>>) -> Object {
	let global_file = fs::read_to_string("./_global.yml");
	let global: HashMap<String, serde_yaml::Value> = match global_file {
		Ok(_) => {
			serde_yaml::from_str(&global_file.unwrap()).unwrap() // Defined as variable as it required a type annotation
		}
		Err(_) => {
			serde_yaml::from_str("locale: \"en_US\"").unwrap() // Defined as variable as it required a type annotation
		}
	};

	/*
	Layouts
	*/
	let layout_name = page.data.get("layout");

	// Import layout context if Page has a layout
	let layout: HashMap<String, serde_yaml::Value> = match layout_name {
		None => HashMap::new(),
		Some(_) => serde_yaml::from_str(
			&split_frontmatter(
				fs::read_to_string(format!(
					"./layouts/{}.mokkf",
					layout_name.unwrap().as_str().unwrap()
				))
				.unwrap(),
			)
			.0,
		)
		.unwrap(),
	};

	let contexts = object!({
		"global": global,
		"page": page,
		"layout": layout,
		"collections": collections,
	});

	contexts
}

/// Returns a String with a &str's File rendered
///
/// # Arguments
///
/// * `page` - A `.mokkf` file's context as a Page
///
/// * `text_to_render` - The text to be rendered
///
/// * `only_context` - Whether or not to only render the contexts of a File
///
/// * `collections` - Collection store of this build
#[inline(always)]
pub fn render(
	page: &Page,
	text_to_render: &str,
	only_context: bool,
	collections: &HashMap<String, Vec<Page>>,
) -> String {
	match only_context {
		true => {
			let template = create_liquid_parser()
				.parse(text_to_render)
				.with_context(|| {
					format!(
						"Could not parse the Mokk file at {}/{}.mokkf",
						page.directory, page.name
					)
				})
				.unwrap();
			template
				.render(&get_contexts(page, collections))
				.with_context(|| {
					format!(
						"Could not render the Mokk file at {}/{}.mokkf",
						page.directory, page.name
					)
				})
				.unwrap()
		}
		false => {
			let template = create_liquid_parser()
				.parse(text_to_render)
				.with_context(|| {
					format!(
						"Could not parse the Mokk file at {}/{}.mokkf",
						page.directory, page.name
					)
				})
				.unwrap();
			let liquid_render = template
				.render(&get_contexts(page, collections))
				.with_context(|| {
					format!(
						"Could not render the Mokk file at {}/{}.mokkf",
						page.directory, page.name
					)
				})
				.unwrap();
			render_markdown(liquid_render)
		}
	}
}

/// Compiles a Mokk File; renders, makes note of the File (when, or if, the need arises)
///
/// # Arguments
///
/// * `page` - The `.mokkf` file's context as a Page
///
/// * `collections` - Collection store of this build
#[inline(always)]
pub fn compile(
	mut page: Page,
	mut collections: HashMap<String, Vec<Page>>,
) -> (String, HashMap<String, Vec<Page>>) {
	let layout_name = &page.data.get("layout");
	let collection_name = &page.data.get("collection");

	// If Page has a layout, render with layout(s)
	// Otherwise, render with Page's contents
	page.content = render(&page, &page.content, !page.markdown, &collections);
	let compiled_page = match layout_name {
		None => page.content.to_owned(),
		Some(_) => {
			let layout_object = get_page_object(
				format!("./layouts/{}.mokkf", layout_name.unwrap().as_str().unwrap()),
				&collections,
			);
			let layouts = render_layouts(&page, layout_object, &collections); // Embed page in layout
			render(&page, &layouts, true, &collections)
			// Final render, to capture whatever layouts & snippets introduce
		}
	};

	// When within a collection, append embeddable page to list of collection's entries
	match collection_name {
		None => {}
		Some(_) => {
			let collection_name_str = collection_name.unwrap().as_str().unwrap();
			match collections.contains_key(&collection_name_str.to_string()) {
				true => {
					(*collections.get_mut(collection_name_str).unwrap()).push(page);
				}
				false => {
					collections.insert(collection_name_str.to_owned(), vec![page]);
				}
			}
		}
	}

	(compiled_page, collections)
}

/// Render the layout(s) of a post recursively (should a layout have a layout of its own)
///
/// # Arguments
///
/// * `page` - The `.mokkf` file's context as a Page
///
/// * `layout` - The File's layout's context as a Page
///
/// * `collections` - Collection store of this build
#[inline(always)]
pub fn render_layouts(
	sub: &Page,
	layout: Page,
	collections: &HashMap<String, Vec<Page>>,
) -> String {
	// Take layout's text, render it with sub's context

	let super_layout = layout.data.get("layout");
	let rendered: String = match super_layout {
		Some(_) => {
			let super_layout_object = get_page_object(
				format!(
					"./layouts/{}.mokkf",
					super_layout.unwrap().as_str().unwrap()
				),
				collections,
			);
			render_layouts(&layout, super_layout_object, collections)
		}
		None => render(sub, &layout.content, !layout.markdown, collections),
	};

	rendered
}

/// Creates a Liquid parser
pub fn create_liquid_parser() -> liquid::Parser {
	let mut partial = liquid::partials::InMemorySource::new();
	let snippets = fs::read_dir("./snippets");
	if let Ok(s) = snippets {
		for snippet in s {
			let unwrapped_snippet = snippet.unwrap();
			let file_name = &unwrapped_snippet.file_name().into_string().unwrap();
			let path = &unwrapped_snippet.path();
			partial.add(file_name, &fs::read_to_string(path).unwrap());
		}
	}
	let partial_compiler = liquid::partials::EagerCompiler::new(partial);
	liquid::ParserBuilder::with_stdlib()
		.tag(liquid_lib::jekyll::IncludeTag)
		.filter(liquid_lib::jekyll::ArrayToSentenceString)
		.filter(liquid_lib::jekyll::Pop)
		.filter(liquid_lib::jekyll::Push)
		.filter(liquid_lib::jekyll::Shift)
		.filter(liquid_lib::jekyll::Slugify)
		.filter(liquid_lib::jekyll::Unshift)
		.filter(liquid_lib::shopify::Pluralize)
		.filter(liquid_lib::extra::DateInTz)
		.partials(partial_compiler)
		.build()
		.unwrap()
}

/// Render Markdown as HTML
///
/// # Arguments
///
/// * `text_to_render` - The Markdown text to render into HTML
#[inline(always)]
pub fn render_markdown(text_to_render: String) -> String {
	let mut markdown_options = Options::empty();
	markdown_options.insert(Options::ENABLE_TABLES);
	markdown_options.insert(Options::ENABLE_FOOTNOTES);
	markdown_options.insert(Options::ENABLE_STRIKETHROUGH);
	markdown_options.insert(Options::ENABLE_TASKLISTS);
	markdown_options.insert(Options::ENABLE_SMART_PUNCTUATION);
	let markdown_parser = Parser::new_ext(&text_to_render, markdown_options);
	let mut rendered_markdown = String::new();
	html::push_html(&mut rendered_markdown, markdown_parser);

	rendered_markdown
}
