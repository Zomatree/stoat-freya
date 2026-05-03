use std::{cell::Ref, rc::Rc};

use freya::{prelude::*, radio::Readable};
use stoat_models::v0;

pub fn map_readable<T, U>(readable: Readable<T>, f: impl Fn(&T) -> &U + 'static) -> Readable<U> {
    let f = Rc::new(f);

    Readable::new(
        Box::new({
            let readable = readable.clone();
            let f = f.clone();

            move || {
                let f = f.clone();

                let ReadableRef::Ref(r) = readable.read() else {
                    panic!("Unsupported")
                };

                ReadableRef::Ref(r.map(move |r| Ref::map(r, |v| f(v))))
            }
        }),
        Box::new({
            let readable = readable.clone();
            let f = f.clone();

            move || {
                let f = f.clone();

                let ReadableRef::Ref(r) = readable.peek() else {
                    panic!("Unsupported")
                };

                ReadableRef::Ref(r.map(move |r| Ref::map(r, |v| f(v))))
            }
        }),
    )
}

pub struct OptionalReadable<T: 'static> {
    pub(crate) read_fn: Rc<dyn Fn() -> Option<ReadableRef<T>>>,
    pub(crate) peek_fn: Rc<dyn Fn() -> Option<ReadableRef<T>>>,
}

impl<T: 'static> OptionalReadable<T> {
    pub fn new(
        read_fn: Box<dyn Fn() -> Option<ReadableRef<T>>>,
        peek_fn: Box<dyn Fn() -> Option<ReadableRef<T>>>,
    ) -> Self {
        Self {
            read_fn: Rc::from(read_fn),
            peek_fn: Rc::from(peek_fn),
        }
    }

    pub fn read(&self) -> Option<ReadableRef<T>> {
        (self.read_fn)()
    }

    pub fn peek(&self) -> Option<ReadableRef<T>> {
        (self.peek_fn)()
    }
}

pub fn map_optional_readable<T, U>(
    readable: Readable<T>,
    f: impl Fn(&T) -> Option<&U> + 'static,
) -> OptionalReadable<U> {
    let f = Rc::new(f);

    OptionalReadable::new(
        Box::new({
            let readable = readable.clone();
            let f = f.clone();

            move || {
                let f = f.clone();

                let ReadableRef::Ref(r) = readable.read() else {
                    panic!("Unsupported")
                };

                r.try_map(|r| Ref::filter_map(r, |v| f(v)).ok())
                    .map(ReadableRef::Ref)
            }
        }),
        Box::new({
            let readable = readable.clone();
            let f = f.clone();

            move || {
                let f = f.clone();

                let ReadableRef::Ref(r) = readable.peek() else {
                    panic!("Unsupported")
                };

                r.try_map(|r| Ref::filter_map(r, |v| f(v)).ok())
                    .map(ReadableRef::Ref)
            }
        }),
    )
}

pub fn parse_hex(hex: &str) -> Option<Color> {
    let hex = hex.strip_prefix('#')?;

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
}

pub fn member_display_color(member: &v0::Member, server: &v0::Server) -> Option<Fill> {
    let mut roles = member
        .roles
        .iter()
        .filter_map(|id| server.roles.get(id))
        .collect::<Vec<_>>();

    roles.sort_by(|a, b| a.rank.cmp(&b.rank));

    let color = roles
        .into_iter()
        .filter_map(|role| role.colour.as_ref())
        .next()?;

    if let Some(color) = parse_hex(color) {
        return Some(Fill::Color(color))
    };

    None
}

// pub fn map_optional_readable<T, U>(
//     readable: Readable<T>,
//     f: impl Fn(&T) -> Option<&U> + 'static,
// ) -> Readable<Option<U>> {
//     let f = Rc::new(f);

//     debug_assert!(Layout::new::<&Option<U>>() == Layout::new::<&U>());

//     Readable::new(
//         Box::new({
//             let readable = readable.clone();
//             let f = f.clone();

//             move || {
//                 let f = f.clone();

//                 let ReadableRef::Ref(r) = readable.read() else {
//                     panic!("Unsupported")
//                 };

//                 ReadableRef::Ref(r.map(|r| match Ref::filter_map(r, |v| f(v)) {
//                     Ok(r) => unsafe { transmute(r) },
//                     Err(r) => Ref::map(r, |_| &None),
//                 }))
//             }
//         }),
//         Box::new({
//             let readable = readable.clone();
//             let f = f.clone();

//             move || {
//                 let f = f.clone();

//                 let ReadableRef::Ref(r) = readable.peek() else {
//                     panic!("Unsupported")
//                 };

//                 ReadableRef::Ref(r.map(|r| match Ref::filter_map(r, |v| f(v)) {
//                     Ok(r) => unsafe { transmute::<Ref<'_, U>, Ref<'_, Option<U>>>(r) },
//                     Err(r) => Ref::map(r, |_| &None),
//                 }))
//             }
//         }),
//     )
// }

// pub struct MapSlice<Value, SliceValue, MapValue, Channel>
// where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static,
//     MapValue: 'static,
// {
//     slice: RadioSlice<Value, SliceValue, Channel>,
//     f: Rc<dyn Fn(&SliceValue) -> &MapValue + 'static>,
// }

// impl<Value, SliceValue, MapValue, Channel> MapSlice<Value, SliceValue, MapValue, Channel>
// where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static,
//     MapValue: 'static,
// {
//     pub fn new(
//         slice: RadioSlice<Value, SliceValue, Channel>,
//         f: Rc<dyn Fn(&SliceValue) -> &MapValue + 'static>,
//     ) -> Self {
//         Self { slice, f }
//     }
// }

// impl<Value, SliceValue, MapValue, Channel> Clone for MapSlice<Value, SliceValue, MapValue, Channel>
// where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static,
//     MapValue: 'static,
// {
//     fn clone(&self) -> Self {
//         Self {
//             slice: self.slice.clone(),
//             f: self.f.clone(),
//         }
//     }
// }

// impl<Value, SliceValue, MapValue, Channel> PartialEq
//     for MapSlice<Value, SliceValue, MapValue, Channel>
// where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static,
//     MapValue: 'static,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.slice == other.slice
//     }
// }

// impl<Value, SliceValue, MapValue, Channel> IntoReadable<MapValue>
//     for MapSlice<Value, SliceValue, MapValue, Channel>
// where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static,
//     MapValue: 'static,
// {
//     fn into_readable(self) -> Readable<MapValue> {
//         Readable::new(
//             Box::new({
//                 let readable = self.slice.clone();
//                 let f = self.f.clone();

//                 move || {
//                     let f = f.clone();
//                     let readable = readable.clone();

//                     ReadableRef::Ref(
//                         readable
//                             .read_unchecked()
//                             .map(move |r| Ref::map(r, |v| f(v))),
//                     )
//                 }
//             }),
//             Box::new({
//                 let readable = self.slice.clone();
//                 let f = self.f.clone();

//                 move || {
//                     let f = f.clone();
//                     let readable = readable.clone();

//                     ReadableRef::Ref(
//                         readable
//                             .peek_unchecked()
//                             .map(move |r| Ref::map(r, |v| f(v))),
//                     )
//                 }
//             }),
//         )
//     }
// }

// pub struct OptionalSlice<Value, SliceValue, Channel> where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static
// {
//     channel: Channel,
//     station: RadioStation<Value, Channel>,
//     selector: Rc<dyn Fn(&Value) -> Option<&SliceValue> + 'static>
// }

// impl<Value, SliceValue, Channel> OptionalSlice<Value, SliceValue, Channel> where
//     Channel: RadioChannel<Value>,
//     Value: 'static,
//     SliceValue: 'static
// {
//     pub fn new(
//         channel: Channel,
//         station: RadioStation<Value, Channel>,
//         selector: impl Fn(&Value) -> Option<&SliceValue> + 'static,
//     ) -> Self {
//         Self {
//             channel,
//             station,
//             selector: Rc::new(selector),
//         }
//     }

//     pub fn peek_unchecked(&self) -> Option<ReadRef<'static, SliceValue>> {
//         let inner = self.station.peek_unchecked();

//         inner.try_map(|v| {
//             let o = Ref::filter_map(v, |v| {
//                 (self.selector)(v)
//             });

//             o.ok()
//         })
//     }
// }
