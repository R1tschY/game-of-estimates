use crate::Asset;
use crate::AssetCatalog;
use bytes::Bytes;
use rust_embed::{Embed, EmbeddedFile};
use std::borrow::Cow;
use std::marker::PhantomData;

pub struct EmbedCatalog<T>(PhantomData<fn(T)>);

impl<T> Default for EmbedCatalog<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Clone for EmbedCatalog<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

pub struct EmbedAsset(EmbeddedFile);

impl<T: Embed> AssetCatalog for EmbedCatalog<T> {
    type Asset = EmbedAsset;

    fn get(&self, path: &str) -> Option<Self::Asset> {
        T::get(path).map(EmbedAsset)
    }

    fn iter(&self) -> impl Iterator<Item = Cow<'static, str>> + 'static {
        T::iter()
    }
}

impl Asset for EmbedAsset {
    fn data(self) -> Bytes {
        match self.0.data {
            Cow::Borrowed(bytes) => Bytes::from(Box::<[u8]>::from(bytes)),
            Cow::Owned(bytes) => Bytes::from(bytes),
        }
    }

    fn len(&self) -> usize {
        self.0.data.len()
    }

    fn last_modified(&self) -> Option<u64> {
        self.0.metadata.last_modified()
    }

    fn sha256(&self) -> [u8; 32] {
        self.0.metadata.sha256_hash()
    }

    fn mimetype(&self) -> &str {
        self.0.metadata.mimetype()
    }
}
