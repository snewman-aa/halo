use derive_more::{AsRef, Deref, Display, From, Into};
use freedesktop_icons::lookup;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Deref, From, Into, AsRef)]
pub struct IconName(String);

crate::impl_string_newtype!(IconName);

pub fn find_icon_path(icon_name: &IconName) -> Option<PathBuf> {
    if icon_name.is_empty() {
        return None;
    }

    let path = Path::new(icon_name.as_ref());
    if path.is_absolute() && path.exists() {
        return Some(path.to_path_buf());
    }

    lookup(icon_name.as_ref())
        .with_size(512)
        .with_scale(1)
        .find()
}
