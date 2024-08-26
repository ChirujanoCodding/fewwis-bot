use std::borrow::Cow;

use crate::http::definitions::{Definition, Word};

pub(crate) fn parse_request(request: String) -> Word {
    // SAFETY: Reference to [tl::parse_owned].
    let dom = unsafe { tl::parse_owned(request, tl::ParserOptions::default()).unwrap() };
    let parser = dom.get_ref();
    let (word, variant) = extract_header(parser);
    let definitions = parser
        .get_elements_by_class_name("j")
        .map(|handle| handle.get(parser.parser()).unwrap())
        .collect::<Vec<_>>();
    let definitions = definitions
        .iter()
        .map(|node| {
            let subnodes = node.children().unwrap().all(parser.parser());
            let def_type = extract_abbrs(subnodes, parser);
            let short = def_type.iter().map(|(s, _)| s.clone()).collect::<Vec<_>>();
            let long = def_type.iter().map(|(_, l)| l.clone()).collect::<Vec<_>>();
            let mut definition = node
                .inner_text(parser.parser())
                .split_ascii_whitespace()
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ");
            for (s, l) in short.iter().zip(long.clone()) {
                definition = definition.replace(s, "");
                definition = definition.replace(&l, "");
            }
            Definition::new(
                long.join(", "),
                to_latin_chars(definition.into()).trim().to_string(),
            )
        })
        .collect::<Vec<_>>();
    Word::new(word, variant, definitions)
}

fn extract_abbrs(nodes: &[tl::Node<'_>], dom: &tl::VDom<'_>) -> Vec<(String, String)> {
    nodes
        .iter()
        .filter(|n| {
            if let Some(n) = n.as_tag() {
                n.name() == "abbr"
            } else {
                false
            }
        })
        .map(|n| {
            let tag = n.as_tag().unwrap();
            (
                tag.inner_text(dom.parser()).to_string(),
                to_latin_chars(
                    tag.attributes()
                        .get("title")
                        .unwrap()
                        .unwrap()
                        .as_utf8_str(),
                ),
            )
        })
        .collect::<Vec<_>>()
}

fn extract_header(parser: &tl::VDom<'_>) -> (String, Option<String>) {
    let word = to_latin_chars(
        parser
            .query_selector("header")
            .and_then(|mut iter| iter.next())
            .unwrap()
            .get(parser.parser())
            .unwrap()
            .as_tag()
            .unwrap()
            .attributes()
            .get("title")
            .unwrap()
            .unwrap()
            .as_utf8_str(),
    );
    let words = word
        .replace("Definición de", "")
        .split(',')
        .map(|d| d.trim().to_string())
        .collect::<Vec<String>>();

    let (word, variant) = {
        if words.len() > 1 {
            (words[0].clone(), Some(words[1].clone()))
        } else {
            (words[0].clone(), None)
        }
    };
    (word, variant)
}

fn to_latin_chars(word: Cow<'_, str>) -> String {
    word.replace("&#xE1;", "á")
        .replace("&#xE9;", "é")
        .replace("&#xED;", "í")
        .replace("&#xF3;", "ó")
        .replace("&#xFA;", "ú")
        .replace("&#xF1;", "ñ")
        .replace("&#x2016;", "||")
}
