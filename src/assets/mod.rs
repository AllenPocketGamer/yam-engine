//! TODO: 先做一个非常简单的同步的只加载纹理的加载器;
//!   虽然, 异步加载, 资源缓存/卸载, 分级加载, 资源依赖等等都非常重要, 但优先级并不高;
//!   所以, 现在的思路就是简单的`同步的`加载`纹理资源`, 返回一个Arc, 并做一个粗糙的缓存, 十分简单!

pub mod types;

use types::{Asset, Texture};

use std::{any::Any, collections::HashMap, sync::Arc};

/// A type alias for `Arc<T>`, which is a solution temporarily.
///
/// Bounds on generic parameters are not enforced in type aliases,
///   Just remind you that `T` should implement the trait `Asset`.
#[allow(type_alias_bounds)]
type Handle<T: Asset> = Arc<T>;

pub struct AssetsLoader {
    // Key: file path to the asset; Value: as you see.
    storages: HashMap<String, Handle<dyn Any + Sync + Send>>,
}

impl AssetsLoader {
    pub(crate) fn new() -> Self {
        Self {
            storages: Default::default(),
        }
    }

    /// `&mut self` is a solution temporarily, will be changed lately.
    pub fn load<A: Asset>(&mut self, path: &str) -> Handle<A> {
        if let Some((_, asset)) = self.storages.get_key_value(path) {
            if let Ok(concrete) = Arc::downcast::<A>(Arc::clone(asset)) {
                concrete
            } else {
                panic!("ERR: temporary");
            }
        } else {
            let asset: Handle<A> = Handle::new(A::load_from(path));

            self.storages.insert(path.to_owned(), Handle::<A>::clone(&asset));

            asset
        }
    }
}
