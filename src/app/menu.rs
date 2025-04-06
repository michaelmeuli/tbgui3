use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::{menu};
use cosmic::Element;
use std::collections::HashMap;

use crate::app::{Action, Message};
use crate::fl;

pub fn menu_bar<'a>(key_binds: &HashMap<KeyBind, Action>) -> Element<'a, Message> {
    let menu_bar = menu::bar(vec![menu::Tree::with_children(
        menu::root(fl!("view")),
        menu::items(
            key_binds,
            vec![menu::Item::Button(fl!("about"), None, Action::About)],
        ),
    )]);

    menu_bar.into()
}
