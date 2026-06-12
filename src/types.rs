use std::collections::HashMap;

pub(crate) type TemplateHandler = fn(&str) -> String;
pub(crate) type DispatchTable = HashMap<&'static str, TemplateHandler>;

pub(crate) type EmptyHandler = fn() -> String;
pub(crate) type EmptyDispatchTable = HashMap<&'static str, EmptyHandler>;

#[derive(Clone, Copy)]
pub(crate) enum PersonRole {
    Author,
    Editor,
}
