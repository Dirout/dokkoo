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
lib.rs - Handling Mokk files (`.mokkf`)

Mokk is a custom file format that is used by Dokkoo to generate static websites.

A Mokk file represents a document or page written in accordance to [the Mokk specification](https://dirout.github.io/mokk).
*/
#![warn(clippy::disallowed_types)]

use ahash::AHashMap;
use chrono::{DateTime, Utc};
use comrak::plugins::syntect::SyntectAdapter;
use comrak::{markdown_to_html_with_plugins, ComrakPlugins, ListStyleType};
use derive_more::{Constructor, Div, Error, From, Into, Mul, Rem, Shl, Shr};
use html_minifier::HTMLMinifier;
use liquid::*;
use miette::{miette, IntoDiagnostic, WrapErr};
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::ffi::OsString;
use std::fmt;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use sys_locale::get_locale;

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
/// A Mokk file's date-time metadata
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
	/// A Mokk file's date-time metadata, formatted per the RFC 3339 standard
	pub rfc_3339: String,
	/// A Mokk file's date-time metadata, formatted per the RFC 2822 standard
	pub rfc_2822: String,
}

/// Handle conversion of a Date object into a string of characters
impl fmt::Display for Date {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.rfc_3339)
	}
}

impl Date {
	/// Convert a `serde_yaml::Value` object into a `Date` object
	///
	/// # Arguments
	///
	/// * `value` - The `serde_yaml::Value` object to convert
	///
	/// * `locale` - A `chrono::Locale` object
	pub fn value_to_date(value: Option<&serde_yaml::Value>, locale: chrono::Locale) -> Date {
		match value {
			Some(d) => {
				let datetime = DateTime::parse_from_rfc3339(
					d.as_str()
						.ok_or(miette!(
							"Unable to read `date` value ({:?}) as a string.",
							d
						))
						.unwrap(),
				)
				.into_diagnostic()
				.wrap_err(format!(
					"Unable to parse `date` value ({:?}) as an RFC 3339 date-time.",
					d
				))
				.unwrap(); // Turn the date-time into a DateTime object for easy manipulation (to generate temporal metadata)

				Date::chrono_to_date(datetime.into(), locale)
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
		}
	}

	/// Convert a `chrono::DateTime` object into a `Date` object
	///
	/// # Arguments
	///
	/// * `datetime` - A `chrono::DateTime<chrono::Utc>` object
	///
	/// * `locale` - A `chrono::Locale` object
	pub fn chrono_to_date(datetime: chrono::DateTime<Utc>, locale: chrono::Locale) -> Date {
		Date {
			year: format!("{}", datetime.format_localized("%Y", locale)),
			short_year: format!("{}", datetime.format_localized("%y", locale)),
			month: format!("{}", datetime.format_localized("%m", locale)),
			i_month: format!("{}", datetime.format_localized("%-m", locale)),
			short_month: format!("{}", datetime.format_localized("%b", locale)),
			long_month: format!("{}", datetime.format_localized("%B", locale)),
			day: format!("{}", datetime.format_localized("%d", locale)),
			i_day: format!("{}", datetime.format_localized("%-d", locale)),
			y_day: format!("{}", datetime.format_localized("%j", locale)),
			w_year: format!("{}", datetime.format_localized("%G", locale)),
			week: format!("{}", datetime.format_localized("%U", locale)),
			w_day: format!("{}", datetime.format_localized("%u", locale)),
			short_day: format!("{}", datetime.format_localized("%a", locale)),
			long_day: format!("{}", datetime.format_localized("%A", locale)),
			hour: format!("{}", datetime.format_localized("%H", locale)),
			minute: format!("{}", datetime.format_localized("%M", locale)),
			second: format!("{}", datetime.format_localized("%S", locale)),
			rfc_3339: datetime.to_rfc3339(),
			rfc_2822: datetime.to_rfc2822(),
		}
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
/// Generated data regarding a Mokk file
pub struct Page {
	/// A Mokk file's contextual data, represented as YAML at the head/front of the file
	pub data: AHashMap<String, serde_yaml::Value>,
	/// A Mokk file's contents following the frontmatter
	pub content: String,
	/// Data representing the output path of a Mokk file.
	/// This is defined in a Mokk file's frontmatter
	pub permalink: String,
	/// A Mokk file's date-time metadata, formatted per the RFC 3339 standard.
	/// This is defined in a Mokk file's frontmatter
	pub date: Date,
	/// Path to the Mokk file, not including the Mokk file itself
	pub directory: String,
	/// The Mokk file's base filename
	pub name: String,
	/// The output path of a file; a processed `permalink` value
	pub url: String,
	/// Whether a Mokk file's contents are intended to be processed as Markdown or not
	pub markdown: bool,
	/// Whether a Mokk file's contents are intended to be processed as LaTeX Math or not
	pub math: bool,
	/// Whether a Mokk file is intended to be minified
	pub minify: bool,
}

/// Handle conversion of a Page object into a string of characters
impl fmt::Display for Page {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{self:#?}")
	}
}

#[derive(
	PartialEq,
	Clone,
	Debug,
	Serialize,
	Deserialize,
	From,
	Into,
	Mul,
	Div,
	Rem,
	Shr,
	Shl,
	Constructor,
)]
/// Build configuration data held in memory during the build process, from the global file
pub struct Global {
	/// The global locale, used to format dates
	pub locale: String,
	/// The `Date` object representing the date & time of the build
	pub date: Date,
	/// Whether the build's outputs are intended to be minified
	pub minify: bool,
}

/// The initial state of a `Global` object
impl Default for Global {
	fn default() -> Self {
		Self {
			locale: default_locale_string(),
			date: Date::default(),
			minify: false,
		}
	}
}

/// Gets a string representing the system locale, if available. Otherwise, defaults to 'en_US'
pub fn default_locale_string() -> String {
	get_locale().unwrap_or("en_US".to_owned())
}

/// Gets the system locale, if available. Otherwise, defaults to `en_US`
pub fn default_locale() -> chrono::Locale {
	chrono::Locale::try_from(default_locale_string().as_str()).unwrap_or(chrono::Locale::en_US)
}

/// Gets a `chrono::Locale` object from a string
pub fn locale_string_to_locale(locale: String) -> chrono::Locale {
	chrono::Locale::try_from(locale.as_str()).unwrap_or(default_locale())
}

/// Data held in memory during the build process
pub struct Build {
	/// A collection of pages, grouped by their collection name
	pub collections: AHashMap<String, Vec<Page>>,
	/// The global context, defined in the Mokk's global file
	pub global_context: (AHashMap<String, serde_yaml::Value>, Global),
	/// The Liquid parser
	pub liquid_parser: liquid::Parser,
}

/// The initial state of a `Build` object
impl Default for Build {
	fn default() -> Self {
		Self {
			collections: AHashMap::new(),
			global_context: get_global_context(),
			liquid_parser: create_liquid_parser(),
		}
	}
}

impl Build {
	/// Returns an object with a `Page`'s context
	///
	/// # Arguments
	///
	/// * `page_path` - The `.mokkf` file's path as a `String`
	pub fn get_page_object(&self, page_path: String) -> Page {
		// Define variables which we'll use to create our Document, which we'll use to generate the Page context
		let split_page = split_frontmatter(
			fs::read_to_string(&page_path)
				.into_diagnostic()
				.wrap_err(format!("Failed to read the file at '{}'.", &page_path))
				.unwrap(),
		); // See file::split_frontmatter
		let frontmatter: AHashMap<String, serde_yaml::Value> = serde_yaml::from_str(&split_page.0)
			.into_diagnostic()
			.wrap_err(format!(
				"Failed to parse frontmatter from '{}'.",
				&page_path
			))
			.unwrap(); // Parse frontmatter as AHashMap (collection of key-value pairs)

		let permalink_string: String = match frontmatter.get("permalink") {
			Some(p) => p
				.as_str()
				.ok_or(miette!(
					"Unable to read `permalink` value ({:?}) as string in frontmatter of file '{}'.",
					p,
					&page_path
				))
				.unwrap()
				.to_string(),
			None => String::new(),
		};

		let markdown_bool: bool = match frontmatter.get("markdown") {
			Some(m) => m
				.as_bool()
				.ok_or(miette!(
					"Unable to read `markdown` value ({:?}) as string in frontmatter of file '{}'.",
					m,
					&page_path
				))
				.unwrap(),
			None => true,
		};

		let math_bool: bool = match frontmatter.get("math") {
			Some(m) => m
				.as_bool()
				.ok_or(miette!(
					"Unable to read `math` value ({:?}) as string in frontmatter of file '{}'.",
					m,
					&page_path
				))
				.unwrap(),
			None => true,
		};

		let locale_value = match frontmatter.get("locale") {
			Some(pl) => pl
				.as_str()
				.ok_or(miette!(
					"Unable to read `locale` value ({:?}) from page frontmatter.",
					pl
				))
				.unwrap()
				.to_owned(),
			None => self.global_context.1.locale.clone(),
		};

		let minify_value = match frontmatter.get("minify") {
			Some(m) => m
				.as_bool()
				.ok_or(miette!(
					"Unable to read `minify` value ({:?}) from page frontmatter.",
					m
				))
				.unwrap()
				.to_owned(),
			None => self.global_context.1.minify,
		};

		let locale: chrono::Locale = locale_string_to_locale(locale_value); // Get locale from Global context

		let date_object = Date::value_to_date(frontmatter.get("date"), locale);

		let page_path_io = Path::new(&page_path[..]); // Turn the path into a Path object for easy manipulation (to get page.directory and page.name)

		// Define our Page
		let mut page = Page {
			minify: minify_value,
			data: serde_yaml::from_str(&split_page.0)
				.into_diagnostic()
				.wrap_err(format!(
					"Unable to parse page frontmatter ({}) while rendering '{}'.",
					&split_page.0, &page_path
				))
				.unwrap(),
			content: split_page.1,
			permalink: permalink_string.clone(),
			date: date_object,
			directory: page_path_io
				.parent()
				.unwrap_or(Path::new(""))
				.to_str()
				.ok_or(miette!(
					"Unable to represent parent directory of page as a string while rendering '{}'.",
					&page_path
				))
				.unwrap()
				.to_owned(),
			name: page_path_io
				.file_stem()
				.unwrap_or(&OsString::new())
				.to_str()
				.ok_or(miette!(
					"Unable to represent file stem of page as a string while rendering '{}'.",
					&page_path
				))
				.unwrap()
				.to_owned(),
			url: String::new(),
			markdown: markdown_bool,
			math: math_bool,
		};

		match &page.permalink[..] {
			// Don't render the URL if the permalink is empty
			"" => {}
			_ => {
				// Render the URL once the Page metadata has been generated
				page.url = self.render(&page, &get_permalink(&permalink_string), false, false);
			}
		}

		page
	}

	/// Returns a Liquid object with a `Page`'s Liquid contexts
	///
	/// # Arguments
	///
	/// * `page` - The `.mokkf` file's context as a `Page`
	pub fn get_contexts(&self, page: &Page) -> Object {
		/*
		Layouts
		*/
		let layout_name = page.data.get("layout");

		// Import layout context if Page has a layout
		let layout: AHashMap<String, serde_yaml::Value> = match layout_name {
			None => AHashMap::new(),
			Some(l) => serde_yaml::from_str(
				&split_frontmatter(
					fs::read_to_string(format!(
						"./layouts/{}.mokkf",
						l
							.as_str()
							.ok_or(miette!("Unable to represent layout name ({:?}) as a string while rendering '{:#?}'.", l, page))
							.unwrap()
					))
					.into_diagnostic()
					.wrap_err(format!("Unable to read layout file ({:?}) mentioned in frontmatter of file '{}'.", l, page.name))
					.unwrap(),
				)
				.0,
			)
			.into_diagnostic()
			.wrap_err(format!("Unable to parse frontmatter of layout file ({:?}) mentioned in frontmatter of file '{}'.", l, page.name))
			.unwrap(),
		};

		let contexts = object!({
			"global": self.global_context.0,
			"page": page,
			"layout": layout,
			"collections": self.collections,
		});

		contexts
	}

	/// Returns a `String` with a `&str`'s Mokk file rendered
	///
	/// # Arguments
	///
	/// * `page` - A `.mokkf` file's context as a `Page`
	///
	/// * `text_to_render` - The text to be rendered
	///
	/// * `markdown` - Whether or not to render Markdown
	///
	/// * `math` - Whether or not to render LaTeX Math
	pub fn render(&self, page: &Page, text_to_render: &str, markdown: bool, math: bool) -> String {
		let template = self
			.liquid_parser
			.parse(text_to_render)
			.into_diagnostic()
			.wrap_err(format!(
				"Unable to parse text to render ('{text_to_render}') for {page:#?}."
			))
			.unwrap();

		let mut rendered = template
			.render(&self.get_contexts(page))
			.into_diagnostic()
			.wrap_err(format!(
				"Unable to render text ('{text_to_render}') for {page:#?}."
			))
			.unwrap();

		rendered = match markdown {
			true => render_markdown(rendered, math),
			false => rendered,
		};

		rendered = match math {
			true => latex2mathml::replace(&rendered)
				.into_diagnostic()
				.wrap_err(format!(
					"Unable to render math in document ('{rendered}') for {page:#?}."
				))
				.unwrap(),
			false => rendered,
		};

		match &page.minify {
			true => {
				let mut html_minifier = HTMLMinifier::new();
				html_minifier
					.digest(&rendered)
					.into_diagnostic()
					.wrap_err(format!("Unable to minify HTML for {page:#?}."))
					.unwrap();
				String::from_utf8_lossy(html_minifier.get_html()).to_string()
			}
			false => rendered,
		}
	}

	/// Compiles a Mokk file; renders, makes note of the Mokk file (when, or if, the need arises)
	///
	/// # Arguments
	///
	/// * `page` - The `.mokkf` file's context as a `Page`
	pub fn compile(&mut self, mut page: Page) -> String {
		let layout_name = &page.data.get("layout");
		let collection_name = &page.data.get("collection");

		// If Page has a layout, render with layout(s)
		// Otherwise, render with Page's contents
		page.content = self.render(&page, &page.content, page.markdown, page.math);
		let compiled_page = match layout_name {
			None => page.content.to_owned(),
			Some(l) => {
				let layout_object = self.get_page_object(
					format!("./layouts/{}.mokkf", l.as_str().ok_or(miette!("Unable to represent layout name ({:?}) as a string while rendering '{:#?}'.", l, page)).unwrap()),
				);
				let layouts = self.render_layouts(&page, layout_object); // Embed page in layout
				self.render(&page, &layouts, false, false)
				// Final render, to capture whatever layouts & snippets introduce
			}
		};

		// When within a collection, append embeddable page to list of collection's entries
		match collection_name {
			None => {}
			Some(c) => {
				let collection_name_str = c
					.as_str()
					.ok_or(miette!(
						"Unable to represent collection name ({:?}) as a string while rendering '{:#?}'.",
						c,
						page
					))
					.unwrap();
				match self
					.collections
					.contains_key(&collection_name_str.to_string())
				{
					true => {
						(*self
							.collections
							.get_mut(collection_name_str)
							.ok_or(miette!(
								"Unable to get collection ({}) while rendering '{:#?}'.",
								collection_name_str,
								page
							))
							.unwrap())
						.push(page);
					}
					false => {
						self.collections
							.insert(collection_name_str.to_owned(), vec![page]);
					}
				}
			}
		}

		compiled_page
	}

	/// Render the layout(s) of a post recursively (should a layout have a layout of its own)
	///
	/// # Arguments
	///
	/// * `page` - The `.mokkf` file's context as a `Page`
	///
	/// * `layout` - The Mokk file's layout's context as a `Page`
	pub fn render_layouts(&self, sub: &Page, layout: Page) -> String {
		// Take layout's text, render it with sub's context

		let merged_sub_page = Page {
			data: sub
				.clone()
				.data
				.into_iter()
				.chain(layout.clone().data)
				.collect(),
			content: layout.clone().content,
			date: sub.clone().date,
			name: sub.clone().name,
			directory: sub.clone().directory,
			permalink: sub.clone().permalink,
			url: sub.clone().url,
			minify: sub.clone().minify,
			markdown: layout.markdown,
			math: layout.math,
		};

		let super_layout = layout.data.get("layout");
		let rendered: String = match super_layout {
			Some(l) => {
				let super_layout_object = self.get_page_object(
					format!(
						"./layouts/{}.mokkf",
						l
							.as_str()
							.ok_or(miette!("Unable to represent layout name ({:?}) as a string while rendering '{:#?}'.", l, merged_sub_page))
							.unwrap()
					),
				);
				self.render_layouts(&merged_sub_page, super_layout_object)
			}
			None => self.render(sub, &layout.content, layout.markdown, layout.math),
		};

		rendered
	}
}

/// Returns an expanded permalink value, for when shorthand is used
///
/// # Arguments
///
/// * `permalink` - A string slice that represents the permalink value specified in the Mokk file
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

/// Returns a tuple with a Mokk file's frontmatter and contents, in that order
///
/// # Arguments
///
/// * `page_text` - The `.mokkf` file's data as a `String`
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
			writeln!(frontmatter, "{}", &line)
				.into_diagnostic()
				.wrap_err(format!("Failed to write a line of frontmatter to memory ({}). Managed to write the following:\n{}", &line, &frontmatter))
				.unwrap();
		} else {
			//contents.push_str(&format!("{}\n", &line));
			writeln!(contents, "{}", &line)
				.into_diagnostic()
				.wrap_err(format!("Failed to write a line of content to memory ({}). Managed to write the following:\n{}", &line, &contents))
				.unwrap();
		}
	}

	if frontmatter.trim().is_empty() {
		frontmatter = "empty: true".to_owned();
	}

	(frontmatter, contents)
}

/// Creates a Liquid parser
pub fn create_liquid_parser() -> liquid::Parser {
	let mut partial = liquid::partials::InMemorySource::new();
	let snippets = glob::glob("./snippets/**/*");
	if let Ok(s) = snippets {
		for snippet in s {
			let unwrapped_snippet = snippet
				.into_diagnostic()
				.wrap_err("Unable to interpret path to snippet file.")
				.unwrap();
			if unwrapped_snippet.is_file() {
				let relative_path = RelativePath::from_path(&unwrapped_snippet)
					.into_diagnostic()
					.wrap_err(format!(
						"Unable to interpret path to snippet file ('{}') as a relative path.",
						unwrapped_snippet.display()
					))
					.unwrap();
				let snippet_name = relative_path.strip_prefix("snippets").unwrap().to_string();
				let path = &unwrapped_snippet.as_path();
				partial.add(
					snippet_name,
					&fs::read_to_string(path)
						.into_diagnostic()
						.wrap_err(format!("Unable to read snippet file '{}'.", path.display()))
						.unwrap(),
				);
			}
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
		.into_diagnostic()
		.wrap_err("Unable to build a Liquid parser.")
		.unwrap()
}

/// Render Markdown as HTML
///
/// # Arguments
///
/// * `text_to_render` - The Markdown text to render into HTML
///
/// * `math` - Whether or not Markdown is being rendered with LaTeX Math
pub fn render_markdown(text_to_render: String, math: bool) -> String {
	let mut options = comrak::Options::default();

	options.extension.strikethrough = true;
	options.extension.tagfilter = false;
	options.extension.table = true;
	options.extension.autolink = false;
	options.extension.tasklist = true;
	options.extension.superscript = !math;
	options.extension.header_ids = Some(String::from("h-"));
	options.extension.footnotes = true;
	options.extension.description_lists = true;
	options.extension.front_matter_delimiter = None;
	options.extension.shortcodes = true;

	options.parse.smart = true;
	options.parse.default_info_string = None;
	options.parse.relaxed_tasklist_matching = true;
	options.parse.relaxed_autolinks = true;

	options.render.hardbreaks = true;
	options.render.github_pre_lang = true;
	options.render.full_info_string = true;
	options.render.width = 80;
	options.render.unsafe_ = true;
	options.render.escape = false;
	options.render.list_style = ListStyleType::Dash;
	options.render.sourcepos = false;

	let mut plugins = ComrakPlugins::default();
	let syntax_highlighting_adapter = SyntectAdapter::new("InspiredGitHub");
	plugins.render.codefence_syntax_highlighter = Some(&syntax_highlighting_adapter);

	markdown_to_html_with_plugins(&text_to_render, &options, &plugins)
}

/// Get the global context
pub fn get_global_context() -> (AHashMap<String, serde_yaml::Value>, Global) {
	let global_context: AHashMap<String, serde_yaml::Value> = match fs::read_to_string(
		"./_global.yml",
	) {
		Ok(g) => {
			serde_yaml::from_str(&g)
				.into_diagnostic()
				.wrap_err(format!("Unable to parse global file ({g})."))
				.unwrap() // Defined as variable as it required a type annotation
		}
		Err(e) => {
			serde_yaml::from_str("empty: true")
				.into_diagnostic()
				.wrap_err(format!("Unable to initialise a blank global file. If you're seeing this message, something is very wrong. The global file cannot be read and a blank, default global file failed to initialise. An error occurred when attempting to read the global file: {e}"))
				.unwrap() // Defined as variable as it required a type annotation
		}
	};

	let locale_value = match global_context.get("locale") {
		Some(l) => l
			.as_str()
			.ok_or(miette!(
				"Unable to read `locale` value ({:?}) from global file.",
				l
			))
			.unwrap()
			.to_owned(),
		None => get_locale().unwrap_or("en_US".to_owned()),
	};

	// let locale: chrono::Locale = chrono::Locale::try_from(locale_value.as_str()).unwrap(); // Get locale from Global context

	let minify_value = match global_context.get("minify") {
		Some(m) => m
			.as_bool()
			.ok_or(miette!(
				"Unable to read `minify` value ({:?}) from global file.",
				m
			))
			.unwrap(),
		None => false,
	};

	let global = Global {
		locale: locale_value.clone(),
		date: Date::chrono_to_date(Utc::now(), locale_string_to_locale(locale_value)),
		minify: minify_value,
	};

	let mut global_map: AHashMap<String, serde_yaml::Value> = serde_yaml::from_value(
		serde_yaml::to_value(global.clone())
			.into_diagnostic()
			.wrap_err(miette!(
				"Unable to represent global file data ({:#?}) as a collection of values.",
				global
			))
			.unwrap(),
	)
	.into_diagnostic()
	.wrap_err(miette!(
		"Unable to represent global file data as a collection of values."
	))
	.unwrap();
	global_map.extend(global_context);

	(global_map, global)
}
