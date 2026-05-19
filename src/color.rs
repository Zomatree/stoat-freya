use std::{f32::consts::PI, sync::LazyLock};

use chumsky::{
    prelude::*,
    text::{digits, ident, int, keyword, whitespace},
};
use freya::prelude::*;
// use regex::Regex;

// static ANGLE: LazyLock<Regex> =
//     LazyLock::new(|| Regex::new(r#"(\S+) *(deg|grad|rad|turn)"#).unwrap());

// // angle of 0. is down

fn float_parser<'a>() -> impl Parser<'a, &'a str, f32> {
    let digits = digits(10).to_slice();

    let frac = just('.').then(digits);

    just('-')
        .or_not()
        .then(int(10))
        .then(frac.or_not())
        .to_slice()
        .map(|s: &str| s.parse().unwrap())
        .boxed()
}

fn percentage<'a>() -> impl Parser<'a, &'a str, u8> {
    int::<&str, _>(10)
        .then_ignore(just("%"))
        .filter_map(|int| int.parse::<u8>().ok())
        .boxed()
}

// fn uncased_keyword(keyword: &str) -> impl Parser<'a

fn named_color_parser<'a>() -> impl Parser<'a, &'a str, Color> {
    let uncased = |keyword: &'static str, color| {
        ident::<&str, _>()
            .filter(move |v| &v.to_lowercase() == keyword)
            .to(color)
            .boxed()
    };

    choice((
        uncased("red", Color::RED),
        uncased("green", Color::GREEN),
        uncased("blue", Color::BLUE),
    ))
}

// pub fn parse_angle(input: &str) -> Option<f32> {
//     let angle = if let Some(captures) = ANGLE.captures(input) {
//         let num = captures.get(1).unwrap().as_str().parse::<f32>().ok()?;

//         match captures.get(2).unwrap().as_str() {
//             "deg" => num,
//             "grad" => num.atan() * (180. / PI),
//             "rad" => num * (180. / PI),
//             "turn" => num * 360.,
//             _ => unreachable!(),
//         }
//     } else {
//         let direction = input.strip_prefix("to ")?.trim();

//         match direction {
//             "bottom" => 0.,
//             "bottom left" => 45.,
//             "left" => 90.,
//             "top left" => 135.,
//             "top" => 180.,
//             "top right" => 225.,
//             "right" => 270.,
//             "bottom right" => 315.,
//             _ => return None
//         }
//     };

//     Some(angle)
// }

fn direction_parser<'a>() -> impl Parser<'a, &'a str, f32> {
    let primary = choice((keyword("top"), keyword("bottom")))
        .to_slice()
        .padded();
    let secondary = choice((keyword("left"), keyword("right")))
        .to_slice()
        .padded();

    just("to")
        .ignore_then(
            secondary
                .clone()
                .map(|v| (Some(v), None))
                .or(primary.map(Some).then(secondary.map(Some))),
        )
        .map(|(a, b)| match (a, b) {
            (Some("bottom"), None) => 0.,
            (Some("bottom"), Some("left")) => 45.,
            (Some("left"), None) => 90.,
            (Some("top"), Some("left")) => 135.,
            (Some("top"), None) => 180.,
            (Some("top"), Some("right")) => 225.,
            (Some("right"), None) => 270.,
            (Some("bottom"), Some("right")) => 315.,
            _ => unreachable!(),
        })
}

fn base_angle_parser<'a>() -> impl Parser<'a, &'a str, f32> {
    let unit = choice((
        keyword("deg"),
        keyword("grad"),
        keyword("rad"),
        keyword("turn"),
    ))
    .to_slice();

    float_parser()
        .then(unit.padded())
        .map(|(amount, unit)| match unit {
            "deg" => amount,
            "grad" => amount.atan() * (180. / PI),
            "rad" => amount * (180. / PI),
            "turn" => amount * 360.,
            _ => unreachable!(),
        } + 180.)
}

fn angle_parser<'a>() -> impl Parser<'a, &'a str, f32> {
    direction_parser().or(base_angle_parser())
}

// pub fn parse_hex(input: &str) -> Option<Color> {
//     let hex = input.strip_prefix('#')?;

//     let (r, g, b, a) = match hex.len() {
//         3 => (
//             u8::from_str_radix(&hex[0..1], 16).ok()?,
//             u8::from_str_radix(&hex[1..2], 16).ok()?,
//             u8::from_str_radix(&hex[2..3], 16).ok()?,
//             0xFF,
//         ),
//         4 => (
//             u8::from_str_radix(&hex[0..1], 16).ok()?,
//             u8::from_str_radix(&hex[1..2], 16).ok()?,
//             u8::from_str_radix(&hex[2..3], 16).ok()?,
//             u8::from_str_radix(&hex[3..4], 16).ok()?,
//         ),
//         6 => (
//             u8::from_str_radix(&hex[0..2], 16).ok()?,
//             u8::from_str_radix(&hex[2..4], 16).ok()?,
//             u8::from_str_radix(&hex[4..6], 16).ok()?,
//             0xFF,
//         ),
//         8 => (
//             u8::from_str_radix(&hex[0..2], 16).ok()?,
//             u8::from_str_radix(&hex[2..4], 16).ok()?,
//             u8::from_str_radix(&hex[4..6], 16).ok()?,
//             u8::from_str_radix(&hex[6..8], 16).ok()?,
//         ),
//         _ => return None,
//     };

//     Some(Color::from_argb(a, r, g, b))
// }

fn hex_parser<'a>() -> impl Parser<'a, &'a str, Color> {
    just('#')
        .ignore_then(digits(16).to_slice())
        .filter_map(|hex: &str| {
            let (r, g, b, a) = match hex.len() {
                3 => (
                    u8::from_str_radix(&hex[0..1], 16).ok()?,
                    u8::from_str_radix(&hex[1..2], 16).ok()?,
                    u8::from_str_radix(&hex[2..3], 16).ok()?,
                    0xFF,
                ),
                4 => (
                    u8::from_str_radix(&hex[0..1], 16).ok()?,
                    u8::from_str_radix(&hex[1..2], 16).ok()?,
                    u8::from_str_radix(&hex[2..3], 16).ok()?,
                    u8::from_str_radix(&hex[3..4], 16).ok()?,
                ),
                6 => (
                    u8::from_str_radix(&hex[0..2], 16).ok()?,
                    u8::from_str_radix(&hex[2..4], 16).ok()?,
                    u8::from_str_radix(&hex[4..6], 16).ok()?,
                    0xFF,
                ),
                8 => (
                    u8::from_str_radix(&hex[0..2], 16).ok()?,
                    u8::from_str_radix(&hex[2..4], 16).ok()?,
                    u8::from_str_radix(&hex[4..6], 16).ok()?,
                    u8::from_str_radix(&hex[6..8], 16).ok()?,
                ),
                _ => return None,
            };

            Some(Color::from_argb(a, r, g, b))
        })
        .boxed()
}

fn rgb_parser<'a>() -> impl Parser<'a, &'a str, Color> {
    let number = int::<&str, _>(10)
        .filter_map(|int| int.parse::<u8>().ok())
        .boxed();

    let percentage = percentage()
        .map(|f| (255. * (f as f32 / 100.)) as u8)
        .boxed();

    let value = percentage.or(number).boxed();

    let modern = value
        .clone()
        .separated_by(whitespace())
        .exactly(3)
        .collect_exactly::<[_; 3]>()
        .boxed()
        .then(just('/').padded().ignore_then(value.clone()).or_not());

    let legacy = value
        .clone()
        .padded()
        .separated_by(just(','))
        .exactly(3)
        .collect_exactly::<[_; 3]>()
        .boxed()
        .then(just(',').padded().ignore_then(value.clone()).or_not());

    let inner = modern
        .or(legacy)
        .map(|([r, g, b], a)| Color::from_argb(a.unwrap_or(255), r, g, b));

    choice((keyword("rgba"), keyword("rgb")))
        .ignore_then(inner.padded().delimited_by(just('('), just(')')))
}

// pub fn parse_stop(input: &str) -> Option<(Color, Option<f32>)> {
//     let (color, len) = if input.starts_with('#') {
//         let color = parse_hex(input)?;

//         let len = input.split_whitespace().next().unwrap().len();

//         (color, len)
//     } else {
//         todo!()
//     };

//     let stop_length = input[len..].trim().split_whitespace().next().unwrap();

//     let offset = if stop_length.len() != 0 && let Some(percentage) = stop_length.strip_suffix('%') {
//         percentage.parse().ok()
//     } else {
//         None
//     };

//     Some((color, offset))
// }

fn inner_color_parser<'a>() -> impl Parser<'a, &'a str, Color> {
    choice((named_color_parser(), hex_parser(), rgb_parser()))
}

fn stop_parser<'a>() -> impl Parser<'a, &'a str, (Color, Option<f32>)> {
    inner_color_parser().padded().then(
        percentage()
            .then_ignore(percentage().or_not())
            .map(|percentage| percentage as f32)
            .or_not(),
    )
}

// pub fn parse_linear_gradient(input: &str) -> Option<LinearGradient> {
//     let inner = input.strip_prefix("linear-gradient(")?.strip_suffix(')')?;

//     let mut parts = inner.split(',');

//     let mut gradient = LinearGradient::new();

//     let first = parts.next()?;

//     if let Some(angle) = parse_angle(first) {
//         gradient = gradient.angle(angle);
//     } else {
//         todo!()
//     }

//     Some(gradient)
// }

pub fn linear_gradient_parser<'a>() -> impl Parser<'a, &'a str, LinearGradient> {
    let angle = angle_parser().padded().then_ignore(just(','));
    let stops = stop_parser()
        .padded()
        .separated_by(just(','))
        .allow_trailing()
        .collect::<Vec<_>>();

    just("linear-gradient")
        .ignore_then(
            // just('(').ignore_then(stops.padded()).then_ignore(just(')'))
            // stops
            angle
                .or_not()
                .then(stops)
                .delimited_by(just('('), just(')'))
        )
        .map(|(angle, stops)| {
        // .map(|stops| { let angle = None;
            let mut gradient = LinearGradient::new().angle(angle.unwrap_or(0.));

            if stops.iter().all(|(_, p)| p.is_some()) {
                for (color, percentage) in stops {
                    gradient = gradient.stop((color, percentage.unwrap()))
                }
            } else {
                let incr = 100. / (stops.len() - 1) as f32;

                for (i, (color, _)) in stops.into_iter().enumerate() {
                    gradient = gradient.stop((color, incr * i as f32))
                }
            }

            gradient
        })
        .boxed()
}

// pub fn parse_radial_gradient(input: &str) -> Option<LinearGradient> {
//     None
// }

// pub fn parse_conic_gradient(input: &str) -> Option<LinearGradient> {
//     None
// }

// pub fn parse_fill(input: &str) -> Option<Fill> {
//     let input = input.trim();

//     if let Some(color) = parse_hex(input) {
//         Some(Fill::Color(color))
//     } else if let Some(gradient) = parse_linear_gradient(input) {
//         Some(Fill::LinearGradient(Box::new(gradient)))
//     } else if let Some(gradient) = parse_radial_gradient(input) {
//         Some(Fill::LinearGradient(Box::new(gradient)))
//     } else if let Some(gradient) = parse_conic_gradient(input) {
//         Some(Fill::LinearGradient(Box::new(gradient)))
//     } else {
//         None
//     }
// }

fn fill_parser<'a>() -> impl Parser<'a, &'a str, Fill> {
    choice((
        inner_color_parser().map(Fill::Color),
        linear_gradient_parser().map(|gr| Fill::LinearGradient(Box::new(gr))),
    ))
}

pub fn parse_fill(input: &str) -> Option<Fill> {
    fill_parser().parse(input).into_result().ok()
}
