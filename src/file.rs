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
file.rs - Handling Mokk Files (.mokkf)
File:
    Term for either a Document or a Page
*/
use chrono::DateTime;
use comrak::{markdown_to_html, ComrakOptions};
use liquid::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// Document:
///     User-specified data regarding a Mokk File
pub struct Document {
    /// A File's contextual data, represented as YAML at the head/front of the file
    pub frontmatter: HashMap<String, serde_yaml::Value>,
    /// A File's contents following the frontmatter
    pub content: String,
    /// Data representing the output path of a File
    /// This is defined in a File's frontmatter
    pub permalink: String,
    /// A File's date-time metadata, formatted per RFC 3339 spec.
    /// This is defined in a File's frontmatter
    pub date: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Page:
///     Generated data regarding a Mokk File
pub struct Page {
    /// A Page's Document
    pub document: Document,
    /// Path to the File, not including the File itself
    pub directory: String,
    /// The File's base filename
    pub name: String,
    pub url: String, // Rendered permalink
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
}

/// Returns an expanded permalink value, for when shorthand is used
///
/// # Arguments
///
/// * `permalink` - A string slice that represents the permalink value specified in the File
///
/// # Shorthand
///
/// * `date` → `/{{ page.document.frontmatter.collection }}/{{ page.year }}/{{ page.month }}/{{ page.day }}/{{ page.document.frontmatter.title }}.html`
///
/// * `pretty` → `/{{ page.document.frontmatter.collection }}/{{ page.year }}/{{ page.month }}/{{ page.day }}/{{ page.document.frontmatter.title }}.html`
///
/// * `ordinal` → `/{{ page.document.frontmatter.collection }}/{{ page.year }}/{{ page.y_day }}/{{ page.document.frontmatter.title }}.html`
///
/// * `weekdate` → `/{{ page.document.frontmatter.collection }}/{{ page.year }}/W{{ page.week }}/{{ page.short_day }}/{{ page.document.frontmatter.title }}.html`
///
/// * `none` → `/{{ page.document.frontmatter.collection }}/{{ page.document.frontmatter.title }}.html`
pub fn get_permalink(permalink: &str) -> String {
    match &*permalink {
        "date" => {
            "/{{ page.document.frontmatter.collection }}/{{ page.year }}/{{ page.month }}/{{ page.day }}/{{ page.document.frontmatter.title }}.html".to_owned()
        }
        "pretty" => {
            "/{{ page.document.frontmatter.collection }}/{{ page.year }}/{{ page.month }}/{{ page.day }}/{{ page.document.frontmatter.title }}.html".to_owned()
        }
        "ordinal" => {
            "/{{ page.document.frontmatter.collection }}/{{ page.year }}/{{ page.y_day }}/{{ page.document.frontmatter.title }}.html"
                .to_owned()
        }
        "weekdate" => {
            "/{{ page.document.frontmatter.collection }}/{{ page.year }}/W{{ page.week }}/{{ page.short_day }}/{{ page.document.frontmatter.title }}.html".to_owned()
        }
        "none" => {
            "/{{ page.document.frontmatter.collection }}/{{ page.document.frontmatter.title }}.html".to_owned()
        }
        _ => {
            permalink.to_string()
        }
    }
}

/// Returns a tuple with a File's frontmatter and contents, in that order.
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
            frontmatter.push_str(&format!("{}\n", &line));
        } else {
            contents.push_str(&format!("{}\n", &line));
        }
    }

    if frontmatter.trim().is_empty() {
        frontmatter = "empty: true".to_owned();
    }

    (frontmatter, contents)
}

/// Returns an object with a Page's context.
///
/// # Arguments
///
/// * `page_path` - The `.mokkf` file's path as a `String`
pub fn get_page_object(page_path: String, collections: &HashMap<String, Vec<Page>>) -> Page {
    // Define variables which we'll use to create our Document, which we'll use to generate the Page context
    let split_page = split_frontmatter(fs::read_to_string(&page_path).unwrap()); // See file::split_frontmatter
    let frontmatter: HashMap<String, serde_yaml::Value> =
        serde_yaml::from_str(&split_page.0).unwrap(); // Parse frontmatter as HashMap (collection of key-value pairs)
    let permalink = frontmatter.get("permalink"); // Get the key-value pair of the 'permalink' key from the frontmatter
    let date = frontmatter.get("date"); // Get the key-value pair of the 'date' key from the frontmatter

    let permalink_string: String;
    let date_string: String;

    match permalink {
        Some(_) => {
            permalink_string = permalink.unwrap().as_str().unwrap().to_string();
        }
        None => {
            permalink_string = "".to_owned();
        }
    }

    match date {
        Some(_) => {
            date_string = date.unwrap().as_str().unwrap().to_string();
        }
        None => {
            date_string = "".to_owned();
        }
    }

    // Plug the variables collected above into our Page's Document (see definitions of Page and Document, in this sourcecode file, for clarification as to their roles)
    let document = Document {
        frontmatter: serde_yaml::from_str(&split_page.0).unwrap(),
        content: split_page.1,
        permalink: permalink_string,
        date: date_string,
    };

    let page_path_io = Path::new(&page_path[..]); // Turn the path into a Path object for easy manipulation (to get page.directory and page.name)
    let mut page: Page;
    match &document.date[..] {
        "" => {
            // Define our Page
            page = Page {
                document,
                directory: page_path_io.parent().unwrap().to_str().unwrap().to_owned(),
                name: page_path_io
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                url: "".to_owned(),
                year: "".to_owned(),
                short_year: "".to_owned(),
                month: "".to_owned(),
                i_month: "".to_owned(),
                short_month: "".to_owned(),
                long_month: "".to_owned(),
                day: "".to_owned(),
                i_day: "".to_owned(),
                y_day: "".to_owned(),
                w_year: "".to_owned(),
                week: "".to_owned(),
                w_day: "".to_owned(),
                short_day: "".to_owned(),
                long_day: "".to_owned(),
                hour: "".to_owned(),
                minute: "".to_owned(),
                second: "".to_owned(),
            };
        }
        _ => {
            let datetime = DateTime::parse_from_rfc3339(date.unwrap().as_str().unwrap()); // Turn the date-time into a DateTime object for easy manipulation (to generate temporal Page metadata)
            let global: HashMap<String, serde_yaml::Value> =
                serde_yaml::from_str(&fs::read_to_string("./_global.yml").unwrap()).unwrap(); // TODO: Figure out a way to not have to get copy of Global context in get_page, save on memory
            let locale: chrono::Locale =
                chrono::Locale::try_from(&(global.get("locale").unwrap().as_str().unwrap()[..]))
                    .unwrap(); // Get locale from Global context
                               // Define our Page
            page = Page {
                document,
                directory: page_path_io.parent().unwrap().to_str().unwrap().to_owned(),
                name: page_path_io
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
                url: "".to_owned(),
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
            };
        }
    }

    match &page.document.permalink[..] {
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

/// Returns a Liquid object with a Page's Liquid contexts.
///
/// # Arguments
///
/// * `page` - The `.mokkf` file's context as a Page
pub fn get_contexts(
    page: &Page,
    collections: &HashMap<String, Vec<Page>>,
    snippet_context: Option<&HashMap<&str, serde_yaml::Value>>,
) -> Object {
    let global: HashMap<String, serde_yaml::Value> =
        serde_yaml::from_str(&fs::read_to_string("./_global.yml").unwrap()).unwrap(); // Defined as variable as it required a type annotation

    /*
    Layouts
    */
    let layout_name = page.document.frontmatter.get("layout");
    let layout: HashMap<String, serde_yaml::Value>;
    // Import layout context if Page has a layout
    match layout_name {
        None => {
            layout = HashMap::new();
        }
        Some(_) => {
            layout = serde_yaml::from_str(
                &split_frontmatter(
                    fs::read_to_string(format!(
                        "./layouts/{}.mokkf",
                        layout_name.unwrap().as_str().unwrap().to_string()
                    ))
                    .unwrap(),
                )
                .0,
            )
            .unwrap();
        }
    }

    let contexts;
    match snippet_context {
        Some(_) => {
            contexts = object!({
                "global": global,
                "page": page,
                "layout": layout,
                "collections": collections,
                "snippet": snippet_context.unwrap()
            });
        }
        None => {
            contexts = object!({
                "global": global,
                "page": page,
                "layout": layout,
                "collections": collections,
            });
        }
    }

    contexts
}

/// Returns a String with a &str's File rendered.
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
pub fn render(
    page: &Page,
    text_to_render: &str,
    only_context: bool,
    collections: &HashMap<String, Vec<Page>>,
) -> String {
    match only_context {
        true => {
            let template = liquid::ParserBuilder::with_stdlib()
                .tag(liquid_lib::jekyll::IncludeTag)
                .filter(liquid_lib::jekyll::ArrayToSentenceString)
                .filter(liquid_lib::jekyll::Pop)
                .filter(liquid_lib::jekyll::Push)
                .filter(liquid_lib::jekyll::Shift)
                .filter(liquid_lib::jekyll::Slugify)
                .filter(liquid_lib::jekyll::Unshift)
                .filter(liquid_lib::shopify::Pluralize)
                .filter(liquid_lib::extra::DateInTz)
                .build()
                .unwrap()
                .parse(text_to_render)
                .unwrap();

            render_snippets(
                page,
                &template
                    .render(&get_contexts(page, collections, None))
                    .unwrap(),
                collections,
            )
        }
        false => {
            let mut markdown_options: ComrakOptions = ComrakOptions::default();
            markdown_options.extension.strikethrough = true;
            markdown_options.extension.tagfilter = false;
            markdown_options.render.unsafe_ = true;
            markdown_options.render.escape = false;
            markdown_options.extension.table = true;
            //markdown_options.extension.autolink = true;
            markdown_options.extension.tasklist = true;
            markdown_options.extension.superscript = true;
            markdown_options.extension.header_ids = Some("".to_string());
            markdown_options.extension.footnotes = true;
            markdown_options.extension.description_lists = true;
            markdown_options.parse.smart = true;
            markdown_options.render.github_pre_lang = true;

            let template = liquid::ParserBuilder::with_stdlib()
                .tag(liquid_lib::jekyll::IncludeTag)
                .filter(liquid_lib::jekyll::ArrayToSentenceString)
                .filter(liquid_lib::jekyll::Pop)
                .filter(liquid_lib::jekyll::Push)
                .filter(liquid_lib::jekyll::Shift)
                .filter(liquid_lib::jekyll::Slugify)
                .filter(liquid_lib::jekyll::Unshift)
                .filter(liquid_lib::shopify::Pluralize)
                .filter(liquid_lib::extra::DateInTz)
                .build()
                .unwrap()
                .parse(&markdown_to_html(text_to_render, &markdown_options))
                .unwrap();

            render_snippets(
                page,
                &template
                    .render(&get_contexts(page, collections, None))
                    .unwrap(),
                collections,
            )
        }
    }
}

/// Compiles a Mokk File; renders, makes note of the File (when, or if, the need arises), returns compiled HTML
///
/// # Arguments
///
/// * `page` - The `.mokkf` file's context as a Page
///
/// * `collections` - Collection store of this build
pub fn compile(
    mut page: Page,
    mut collections: HashMap<String, Vec<Page>>,
) -> (String, HashMap<String, Vec<Page>>) {
    let compiled_page;
    let embeddable_page = render(&page, &page.document.content, false, &collections);
    let layout_name = &page.document.frontmatter.get("layout");
    let collection_name = &page.document.frontmatter.get("collection");

    // If Page has a layout, render with layout(s)
    // Otherwise, render with Document's contents
    match layout_name {
        None => {
            compiled_page = embeddable_page.to_string();
        }
        Some(_) => {
            let layout_object = get_page_object(
                format!(
                    "./layouts/{}.mokkf",
                    layout_name.unwrap().as_str().unwrap().to_string()
                ),
                &collections,
            );
            compiled_page = render(
                &page,
                &render_layouts(&page, layout_object, &collections),
                false,
                &collections,
            );
        }
    }

    // When within a collection, append embeddable_page to list of collection's entries
    match collection_name {
        None => {}
        Some(_) => {
            page.document.content = embeddable_page;
            match collections.contains_key(&collection_name.unwrap().as_str().unwrap().to_string())
            {
                true => {
                    let mut current_collection_entries = collections
                        .get_key_value(collection_name.unwrap().as_str().unwrap())
                        .unwrap()
                        .1
                        .to_vec();
                    current_collection_entries.push(page);
                }
                false => {
                    collections.insert(
                        collection_name.unwrap().as_str().unwrap().to_string(),
                        vec![page],
                    );
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
pub fn render_layouts(
    sub: &Page,
    layout: Page,
    collections: &HashMap<String, Vec<Page>>,
) -> String {
    // Take layout's text, render it with sub's context
    let rendered: String;

    let super_layout = layout.document.frontmatter.get("layout");
    match super_layout {
        Some(_) => {
            let super_layout_object = get_page_object(
                format!(
                    "./layouts/{}.mokkf",
                    super_layout.unwrap().as_str().unwrap().to_string()
                ),
                collections,
            );
            rendered = render_layouts(&layout, super_layout_object, collections);
        }
        None => {
            rendered = render(&sub, &layout.document.content, true, collections);
        }
    }

    rendered
}

/// Render all snippets throughout a '.mokkf' file together
///
/// # Arguments
///
/// * `page` - The `.mokkf` file's context as a Page
///
/// * `text_to_parse` - The text to be parsed
///
/// * `collections` - Collection store of this build
pub fn render_snippets(
    page: &Page,
    text_to_parse: &str,
    collections: &HashMap<String, Vec<Page>>,
) -> String {
    let mut snippet_calls: Vec<String> = vec![];
    let mut brace_count = 0;
    let mut parsing_str: String = "".to_owned();
    let mut parsed_str = text_to_parse.to_owned();

    for character in text_to_parse.chars() {
        match character {
            '{' => {
                if brace_count == 0 {
                    brace_count += 1;
                    parsing_str.push(character);
                    continue;
                }
            }
            '}' => {
                if brace_count == 1 {
                    brace_count = 0;
                    parsing_str.push(character);
                    if parsing_str.contains("{! snippet ") {
                        snippet_calls.push(parsing_str);
                    }
                    parsing_str = String::new();
                    continue;
                }
            }
            _ => {
                if brace_count == 1 {
                    parsing_str.push(character);
                    continue;
                }
            }
        }
        for snippet_call in &snippet_calls {
            let call_portions = get_snippet_call_portions(snippet_call.to_owned());
            let snippet_path = format!("./snippets/{}", call_portions[2]);

            let keys = get_snippet_keys(&call_portions);
            let values = get_snippet_values(&call_portions, &keys);
            let mut snippet_context: HashMap<&str, serde_yaml::Value> = HashMap::new();

            for i in 0..keys.len() {
                snippet_context.insert(&keys[i], serde_yaml::from_str(&values[i]).unwrap());
            }

            parsed_str = text_to_parse.replace(
                snippet_call,
                &render_snippet(page, snippet_path, &snippet_context, collections),
            );
        }
    }

    parsed_str
}

/// Render an individual snippet call
///
/// # Arguments
///
/// * `page` - The `.mokkf` file's context as a Page
///
/// * `snippet_path` - The path to the snippet being called
///
/// * `snippet_context` - The context passed within the snippet call
///
/// * `collections` - Collection store of this build
pub fn render_snippet(
    page: &Page,
    snippet_path: String,
    snippet_context: &HashMap<&str, serde_yaml::Value>,
    collections: &HashMap<String, Vec<Page>>,
) -> String {
    let template = liquid::ParserBuilder::with_stdlib()
        .tag(liquid_lib::jekyll::IncludeTag)
        .filter(liquid_lib::jekyll::ArrayToSentenceString)
        .filter(liquid_lib::jekyll::Pop)
        .filter(liquid_lib::jekyll::Push)
        .filter(liquid_lib::jekyll::Shift)
        .filter(liquid_lib::jekyll::Slugify)
        .filter(liquid_lib::jekyll::Unshift)
        .filter(liquid_lib::shopify::Pluralize)
        .filter(liquid_lib::extra::DateInTz)
        .build()
        .unwrap()
        .parse(&fs::read_to_string(snippet_path).unwrap())
        .unwrap();

    template
        .render(&get_contexts(page, collections, Some(snippet_context)))
        .unwrap()
}

/// Get the portions of a snippet call; seperate the call by spaces
///
/// # Arguments
///
/// * `snippet_call` - The snippet call to be cut up
pub fn get_snippet_call_portions(snippet_call: String) -> Vec<String> {
    let mut call_portions: Vec<String> = vec![];
    let mut current_argument: String = "".to_owned();

    for character in snippet_call.chars() {
        match character {
            ' ' => {
                call_portions.push(current_argument);
                current_argument = String::new();
                continue;
            }
            _ => {
                current_argument.push(character);
                continue;
            }
        }
    }

    call_portions
}

/// Get the keys of a snippet call's arguments, should they exist
///
/// # Arguments
///
/// * `call_portions` - A snippet call, seperated into multiple portions by spaces
pub fn get_snippet_keys(call_portions: &[String]) -> Vec<String> {
    let mut keys: Vec<String> = vec![];
    let mut current_key: String = "".to_owned();

    for call_argument in call_portions.iter().skip(3) {
        // Skip three places, so as to just look at the actual argument portions
        for character in call_argument.chars() {
            match character {
                '=' => {
                    keys.push(current_key);
                    current_key = String::new();
                    break;
                }
                _ => {
                    current_key.push(character);
                    continue;
                }
            }
        }
    }

    keys
}

/// Get the values of a snippet call's arguments, should they exist
///
/// # Arguments
///
/// * `call_portions` - A snippet call, seperated into multiple portions by spaces
///
/// * `keys` - The keys of a snippet call's arguments
pub fn get_snippet_values(call_portions: &[String], keys: &[String]) -> Vec<String> {
    let mut values: Vec<String> = vec![];
    let mut current_value: String = "".to_owned();
    let mut portions_by_space: Vec<usize> = vec![]; // Indices of portions of the argument separated by spaces

    for i in 0..keys.len() {
        // Skip if this portion of the arguments has been processed as a part of a quoted value
        if portions_by_space.contains(&(i + 3)) {
            continue;
        }

        current_value = format!("{}{}", current_value, call_portions[i + 3]); // Append this portion of the arguments to the current_value
        current_value = current_value.replace(&format!("{}=", &keys[i]), ""); // Get value by removing key

        let start_of_current_value = current_value.chars().next().unwrap();

        // If value is in quotes, get all pieces of argument it's in, regardless of space-character separators
        if start_of_current_value == '"' {
            for (j, _) in call_portions.iter().enumerate().skip(i + 4) {
                // 'i + 4' comes from 'i + 3' and 'i + 1'; the '+ 3' offset handles the initial portions of the call, allowing us to reach the call arguments
                if call_portions[j].contains('=') {
                    portions_by_space.push(j);
                    break;
                } else {
                    current_value = format!("{} {}", current_value, call_portions[j]);
                    continue;
                }
            }
        }

        let end_of_current_value = current_value
            .chars()
            .nth(current_value.chars().count() - 1)
            .unwrap(); // Define here, as above can modify current_value
                       // Remove quotes around current_value
        if start_of_current_value == '"' && end_of_current_value == '"' {
            current_value.remove(0);
            current_value.remove(current_value.len() - 1);
        }

        values.push(current_value);
        current_value = String::new();
    }

    values
}
