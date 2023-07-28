use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub use kv_macro::kv_storage as kv_storage_macro;

#[async_trait(?Send)]
pub trait KvStorage {
    async fn put<T: Serialize>(key: &str, value: &T);
    async fn get<T: DeserializeOwned>(key: &str) -> T;
    async fn del(key: &str);
}

#[async_trait(?Send)]
pub trait StorageItem<T: KvStorage + Sized> {
    type Value<'b>: for<'a> Deserialize<'a> + Serialize + 'b;

    async fn set_value<'b>(&self, value: Self::Value<'b>) {
        T::put::<Self::Value<'b>>(&self.key(), &value).await;
    }

    async fn value<'b>(&self) -> Self::Value<'b> {
        T::get(&self.key()).await
    }

    async fn update<'b, F: FnOnce(&mut Self::Value<'b>)>(&self, update_fn: F) {
        let mut value = self.value().await;
        update_fn(&mut value);
        self.set_value(value).await;
    }

    fn key(&self) -> &'static str;
}

// StorageItem version using static methods
//
// #[async_trait(?Send)]
// pub trait StorageItem<T: KvStorage + Sized> {
//     type Value<'b>: for<'a> Deserialize<'a> + Serialize + 'b;
//     const PATH: &'static str;
//
//     async fn set_value<'b>(value: Self::Value<'b>) {
//         T::put(Self::PATH, JsValue::from_serde(&value).unwrap()).await;
//     }
//
//     async fn value<'b>() -> Self::Value<'b> {
//         T::get(Self::PATH).await.into_serde().unwrap()
//     }
//
//     async fn update<'b, F: FnOnce(&mut Self::Value<'b>)>(update_fn: F) {
//         let mut value = Self::value().await;
//         update_fn(&mut value);
//         Self::set_value(value).await;
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[kv_storage_macro]
//     struct Storage {
//         name: String,
//     }
//
//     #[test]
//     fn it_works() {}
// }
