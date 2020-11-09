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
use std::fs;
use std::path::Path;
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
/// Document:
///     User-specified data regarding a Mokk File
pub struct Document {
    /// A File's contextual data, represented as YAML at the head/front of the file
    frontmatter: HashMap<String, String>,
    /// A File's contents following the frontmatter
    content: String,
    /// Data representing the output path of a File
    /// This is defined in a File's frontmatter
    permalink: String,
    /// A File's date-time metadata, formatted per RFC 3339 spec.
    /// This is defined in a File's frontmatter
    date: String,
}

#[derive(Debug, Deserialize, Serialize)]
/// Page:
///     Generated data regarding a Mokk File
pub struct Page {
    document: Document,
    dir: String,
    name: String,
    url: String, // Rendered permalink
    year: String,
    short_year: String,
    month: String,
    i_month: String,
    short_month: String,
    long_month: String,
    day: String,
    i_day: String,
    y_day: String,
    w_year: String,
    week: String,
    w_day: String,
    short_day: String,
    long_day: String,
    hour: String,
    minute: String,
    second: String,
}

/// Returns an expanded permalink value, for when shorthand is used
///
/// # Arguments
///
/// * `permalink` - A string slice that represents the permalink value specified in the File
///
/// # Shorthand
///
/// * `date` → `/{{ page.collection }}/{{ page.year }}/{{ page.month }}/{{ page.day }}/{{ page.title }}.html`
///
/// * `pretty` → `/{{ page.collection }}/{{ page.year }}/{{ page.month }}/{{ page.day }}/{{ page.title }}.html`
///
/// * `ordinal` → `/{{ page.collection }}/{{ page.year }}/{{ page.y_day }}/{{ page.title }}.html`
///
/// * `weekdate` → `/{{ page.collection }}/{{ page.year }}/W{{ page.week }}/{{ page.short_day }}/{{ page.title }}.html`
///
/// * `none` → `/{{ page.collection }}/{{ page.title }}.html`
pub fn get_permalink(permalink: &str) -> String {
    match &*permalink {
        "date" => {
            "/{{ page.collection }}/{{ page.year }}/{{ page.month }}/{{ page.day }}/{{ page.title }}.html".to_owned()
        }
        "pretty" => {
            "/{{ page.collection }}/{{ page.year }}/{{ page.month }}/{{ page.day }}/{{ page.title }}.html".to_owned()
        }
        "ordinal" => {
            "/{{ page.collection }}/{{ page.year }}/{{ page.y_day }}/{{ page.title }}.html"
                .to_owned()
        }
        "weekdate" => {
            "/{{ page.collection }}/{{ page.year }}/W{{ page.week }}/{{ page.short_day }}/{{ page.title }}.html".to_owned()
        }
        "none" => {
            "/{{ page.collection }}/{{ page.title }}.html".to_owned()
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
            frontmatter.push_str(&line);
        } else {
            contents.push_str(&line);
        }
    }

    (frontmatter, contents)
}

/// Returns an object with a Page's context.
///
/// # Arguments
///
/// * `page_path` - The `.mokkf` file's path as a `String`
pub fn get_page_object(page_path: String) -> Page {
    // Define variables which we'll use to create our Document, which we'll use to generate the Page context
    let split_page = split_frontmatter(fs::read_to_string(&page_path).unwrap()); // See file::split_frontmatter
    let frontmatter: HashMap<String, String> = serde_yaml::from_str(&split_page.0).unwrap(); // Parse frontmatter as HashMap (collection of key-value pairs)
    let permalink = frontmatter.get_key_value("permalink"); // Get the key-value pair of the 'permalink' key from the frontmatter
    let date = frontmatter.get_key_value("date"); // Get the key-value pair of the 'date' key from the frontmatter

    // Plug the variables collected above into our Page's Document (see definitions of Page and Document, in this sourcecode file, for clarification as to their roles)
    let document = Document {
        frontmatter: serde_yaml::from_str(&split_page.0).unwrap(),
        content: split_page.1,
        permalink: permalink.unwrap().1.to_string(),
        date: date.unwrap().1.to_string(),
    };

    let page_path_io = Path::new(&page_path[..]); // Turn the path into a Path object for easy manipulation (to get page.dir and page.name)
    let datetime = DateTime::parse_from_rfc3339(date.unwrap().1); // Turn the date-time into a DateTime object for easy manipulation (to generate temporal Page metadata)
    let global: HashMap<String, String> =
        serde_yaml::from_str(&fs::read_to_string("./_global.yml").unwrap()).unwrap(); // TODO: Figure out a way to not have to get copy of Global context in get_page, save on memory
    let locale: chrono::Locale = chrono::Locale::try_from(&(global.get_key_value("locale").unwrap().1)[..]).unwrap(); // Get locale from Global context

    // Define our Page
    let mut page = Page {
        document,
        dir: page_path_io.parent().unwrap().to_str().unwrap().to_owned(),
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

    // Render the URL once the Page metadata has been generated
    page.url = render(&page, &get_permalink(permalink.unwrap().1));

    // Render Page content, set page.document.content as rendered version
    page.document.content = render(&page, &page.document.content);

    page
}

/// Returns a Liquid object with a Page's Liquid contexts.
///
/// # Arguments
///
/// * `page` - The `.mokkf` file's context as a Page
pub fn get_contexts(page: &Page) -> Object {
    let global: HashMap<String, String> =
        serde_yaml::from_str(&fs::read_to_string("./_global.yml").unwrap()).unwrap(); // Defined as variable as it required a type annotation

    /*
    Collections
    */
    let collection_name = page.document.frontmatter.get_key_value("collection");
    let collection: HashMap<String, String>;
    // Import collection context if Page is in a collection
    match collection_name {
        None => {
            collection = HashMap::new();
        }
        Some(_) => {
            collection = serde_yaml::from_str(
                &fs::read_to_string(format!("./_{}/_collection.yml", collection_name.unwrap().1))
                    .unwrap(),
            )
            .unwrap();
        }
    }

    /*
    Layouts
    */
    let layout_name = page.document.frontmatter.get_key_value("layout");
    let layout: HashMap<String, String>;
    // Import layout context if Page has a layout
    match layout_name {
        None => {
            layout = HashMap::new();
        }
        Some(_) => {
            layout = serde_yaml::from_str(
                &split_frontmatter(
                    fs::read_to_string(format!("./layouts/{}.mokkf", layout_name.unwrap().1))
                        .unwrap(),
                )
                .0,
            )
            .unwrap();
        }
    }

    let contexts = object!({
        "global": global,
        "page": page,
        "collection": collection,
        "layout": layout
    });

    contexts
}

/// Returns a String with a &str's File rendered.
///
/// # Arguments
///
/// * `page` - A `.mokkf` file's context as a Page
///
/// * `text_to_render` - The text to be rendered
pub fn render(page: &Page, text_to_render: &str) -> String {
    let mut markdown_options: ComrakOptions = ComrakOptions::default();
    markdown_options.extension.strikethrough = true;
    markdown_options.extension.tagfilter = true;
    markdown_options.render.unsafe_ = true;
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
        .build()
        .unwrap()
        .parse(&markdown_to_html(text_to_render, &markdown_options))
        .unwrap();

    template.render(&get_contexts(page)).unwrap()
}

/// Compiles a Mokk File; renders, makes note of the File (if needed), returns compiled HTML
///
/// # Arguments
///
/// * `page` - The `.mokkf` file's context as a Page
pub fn compile(page: &Page) -> String {
    let compiled_page;
    let layout_name = page.document.frontmatter.get_key_value("layout");

    // If Page has a layout, render with layout
    // Otherwise, render with Document's contents
    match layout_name {
        None => {
            compiled_page = page.document.content.to_owned();
        }
        Some(_) => {
            compiled_page = render(
                &page,
                &compile(&get_page_object(format!(
                    "./layouts/{}.mokkf",
                    layout_name.unwrap().1
                ))),
            );
        }
    }

    // If within a collection, append page.document.content to list of collection's entries

    compiled_page
}
