use crate::{
    app::icons,
    app::{Action, Message},
    fl,
};
use cosmic::{
    widget::menu::{items, key_bind::KeyBind, root, Item, ItemHeight, ItemWidth, MenuBar, Tree},
    Element,
};
use std::collections::HashMap;
use std::vec;

pub fn menu_bar<'a>(key_binds: &HashMap<KeyBind, Action>) -> Element<'a, Message> {
    MenuBar::new(vec![
        Tree::with_children(
            root(fl!("file")),
            items(
                key_binds,
                vec![
                    Item::Button(
                        fl!("new-window"),
                        Some(icons::get_handle("tabs-stack-symbolic", 14)),
                        Action::WindowNew,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("quit"),
                        Some(icons::get_handle("cross-small-square-filled-symbolic", 14)),
                        Action::WindowClose,
                    ),
                ],
            ),
        ),
        Tree::with_children(
            root(fl!("view")),
            items(
                key_binds,
                vec![
                    Item::Button(
                        fl!("menu-settings"),
                        Some(icons::get_handle("settings-symbolic", 14)),
                        Action::Settings,
                    ),
                    Item::Divider,
                    Item::Button(
                        fl!("menu-about"),
                        Some(icons::get_handle("info-outline-symbolic", 14)),
                        Action::About,
                    ),
                ],
            ),
        ),
    ])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(260))
    .spacing(4.0)
    .into()
}

//     let menu_bar = menu::bar(vec![menu::Tree::with_children(
//         menu::root(fl!("view")),
//         menu::items(
//             key_binds,
//             vec![menu::Item::Button(fl!("about"), None, Action::About)],
//         ),
//     )]);

//     menu_bar.into()
// }
