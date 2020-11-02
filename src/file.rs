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
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

    let page_path_IO = Path::new(&page_path[..]); // Turn the path into a Path object for easy manipulation (to get page.dir and page.name)
    let datetime = DateTime::parse_from_rfc3339(date.unwrap().1); // Turn the date-time into a DateTime object for easy manipulation (to generate temporal Page metadata)

    let page = Page {
        document,
        dir: page_path_IO.parent().unwrap().to_str().unwrap().to_owned(),
        name: page_path_IO
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned(),
        url: "TODO:IMPLEMENT".to_owned(),
        year: format!("{}", datetime.unwrap().format("%Y")),
        short_year: format!("{}", datetime.unwrap().format("%y")),
        month: format!("{}", datetime.unwrap().format("%m")),
        i_month: format!("{}", datetime.unwrap().format("%-m")),
        short_month: format!("{}", datetime.unwrap().format("%b")),
        long_month: format!("{}", datetime.unwrap().format("%B")),
        day: format!("{}", datetime.unwrap().format("%d")),
        i_day: format!("{}", datetime.unwrap().format("%-d")),
        y_day: format!("{}", datetime.unwrap().format("%j")),
        w_year: format!("{}", datetime.unwrap().format("%G")),
        week: format!("{}", datetime.unwrap().format("%U")),
        w_day: format!("{}", datetime.unwrap().format("%u")),
        short_day: format!("{}", datetime.unwrap().format("%a")),
        long_day: format!("{}", datetime.unwrap().format("%A")),
        hour: format!("{}", datetime.unwrap().format("%H")),
        minute: format!("{}", datetime.unwrap().format("%M")),
        second: format!("{}", datetime.unwrap().format("%S")),
    };

    // Render page content, set page.content as rendered version

    page
}
