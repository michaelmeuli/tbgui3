use cosmic::widget::icon;
use include_dir::{include_dir, Dir};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IconCacheKey {
    name: &'static str,
    size: u16,
}

pub struct IconCache {
    cache: HashMap<IconCacheKey, icon::Handle>,
}

impl IconCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();
        macro_rules! bundle {
            ($name:expr, $size:expr) => {
                let data: &'static [u8] =
                    include_bytes!(concat!("../../res/icons/bundled/", $name, ".svg"));
                cache.insert(
                    IconCacheKey {
                        name: $name,
                        size: $size,
                    },
                    icon::from_svg_bytes(data).symbolic(true),
                );
            };
        }

        bundle!("calendar", 16);
        bundle!("clock", 16);
        bundle!("delete", 16);
        bundle!("detail", 16);
        bundle!("download", 16);
        bundle!("meds", 16);
        bundle!("play", 16);
        bundle!("reload", 16);
        bundle!("scanner", 16);

        Self { cache }
    }

    pub fn get(&mut self, name: &'static str, size: u16) -> icon::Icon {
        let handle = self
            .cache
            .entry(IconCacheKey { name, size })
            .or_insert_with(|| icon::from_name(name).size(size).handle())
            .clone();
        icon::icon(handle).size(size)
    }
}

static ICON_CACHE: OnceLock<Mutex<IconCache>> = OnceLock::new();

pub fn icon_cache_get(name: &'static str, size: u16) -> icon::Icon {
    let mut icon_cache = ICON_CACHE
        .get_or_init(|| Mutex::new(IconCache::new()))
        .lock()
        .unwrap();
    icon_cache.get(name, size)
}
