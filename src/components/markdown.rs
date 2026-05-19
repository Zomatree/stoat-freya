use std::sync::LazyLock;
use itertools::Itertools;

use freya::prelude::*;
use regex::Regex;

#[derive(PartialEq)]
pub struct Markdown {
    pub content: String,
}

impl Component for Markdown {
    fn render(&self) -> impl IntoElement {
        let parts = parse_markdown(self.content.clone());
        println!("part: {parts:?}");

        let mut elements = Vec::new();
        let mut last = None::<Part>;

        for part in parts {
            if let Some(last) = &last && last != &Part::Newline {
                elements.push(label().text(" ").into_element())
            };

            match part.clone() {
                Part::Text(text) => {elements.push(label().max_lines(1).text(text).into_element())}
                Part::Emoji(id) => {elements.push(label().max_lines(1).text("<EMOJI>").into_element())}
                Part::ChannelMention(id) => {elements.push(label().max_lines(1).text("<CHANNEL>").into_element())}
                Part::Newline => {elements.push(label().width(Size::Fill).into_element())}
            }

            last = Some(part);
        };

        rect()
        .background(Color::GRAY)
            .horizontal()
            .content(Content::Wrap {
                wrap_spacing: Some(2.),
            })
            .children(elements)
            // .children(parts.into_iter().map(|part| match part {
            //     Part::Text(text) => label().max_lines(1).text(text).into_element(),
            //     Part::Emoji(id) => label().max_lines(1).text("<EMOJI>").into_element(),
            //     Part::ChannelMention(id) => label().max_lines(1).text("<CHANNEL>").into_element(),
            //     Part::Newline => label().width(Size::Fill).into_element()
            // }).intersperse(label().text(" ").into_element()))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Part {
    Text(String),
    Emoji(String),
    ChannelMention(String),
    Newline
}

static EMOJI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^:([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26}):$"#).unwrap());
static CHANNEL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^<#([0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26})>$"#).unwrap());

fn parse_markdown(content: String) -> Vec<Part> {
    let lines = content.split_inclusive('\n');

    let mut out = Vec::new();

    for line in lines {
        println!("line: {line:?}");
        let (line, add_newline) = if line.ends_with('\n') {
            (&line[..line.len() - 1], true)
        } else {
            (line, false)
        };

        for word in line.split(' ') {
            println!("word: {word:?}");
            if let Some(captures) = EMOJI.captures(word) {
                out.push(Part::Emoji(captures.get(1).unwrap().as_str().to_string()));
            } else if let Some(captures) = CHANNEL.captures(word) {
                out.push(Part::ChannelMention(
                    captures.get(1).unwrap().as_str().to_string(),
                ));
            } else {
                out.push(Part::Text(word.to_string()));
            }
        };

        if add_newline {
            out.push(Part::Newline);
        }
    }

    out
}

/*
        if let Some(captures) = EMOJI.captures(part) {
            out.push(Part::Emoji(captures.get(1).unwrap().as_str().to_string()));
        } else if let Some(captures) = CHANNEL.captures(part) {
            out.push(Part::ChannelMention(
                captures.get(1).unwrap().as_str().to_string(),
            ));
        } else {
            out.push(Part::Text(part.to_string()));
        }
*/