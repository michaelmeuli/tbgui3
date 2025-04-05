use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::menu::{items, root, Item};
use cosmic::{
    widget::menu::{ItemHeight, ItemWidth, MenuBar, Tree},
    Element,
};
use std::collections::HashMap;
use cosmic::widget::{self, menu, nav_bar};

use cosmic::widget::menu::action::MenuAction;

use crate::app::{Action, Message};
use crate::fl;

pub fn menu_bar<'a>(key_binds: &HashMap<KeyBind, Action>) -> Element<'a, Message> {
    let menu_bar = menu::bar(vec![menu::Tree::with_children(
        menu::root(fl!("view")),
        menu::items(
            key_binds,
            vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
        ),
    )]);

    vec![menu_bar.into()]
}
