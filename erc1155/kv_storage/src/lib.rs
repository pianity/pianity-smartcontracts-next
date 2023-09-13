use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

pub use kv_macro::kv_storage as kv_storage_macro;

#[async_trait(?Send)]
pub trait KvStorage {
    async fn put<T: Serialize>(key: &str, value: &T);
    async fn get<T: DeserializeOwned>(key: &str) -> T;
    async fn del(key: &str);
}

// #[async_trait(?Send)]
// pub trait StorageItem<T: KvStorage + Sized> {
//     type Value<'b>: for<'a> Deserialize<'a> + Serialize + 'b;
//
//     async fn set_value<'b>(&self, value: Self::Value<'b>) {
//         T::put::<Self::Value<'b>>(&self.key(), &value).await;
//     }
//
//     async fn value<'b>(&self) -> Self::Value<'b> {
//         T::get(&self.key()).await
//     }
//
//     async fn update<'b, F: FnOnce(&mut Self::Value<'b>)>(&self, update_fn: F) {
//         let mut value = self.value().await;
//         update_fn(&mut value);
//         self.set_value(value).await;
//     }
//
//     fn key(&self) -> &'static str;
// }

#[cfg(test)]
mod tests {
    use super::{kv_storage_macro, KvStorage};
    use async_trait::async_trait;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json;

    struct Kv;

    #[async_trait(?Send)]
    impl KvStorage for Kv {
        async fn put<T: Serialize>(key: &str, value: &T) {
            println!(
                "{}",
                &format!("put: {} = {:?}", key, serde_json::to_string(value).unwrap())
            );
        }

        async fn get<T: DeserializeOwned>(key: &str) -> T {
            println!("{}", &format!("get: {}", key));

            match key {
                ".name" => serde_json::from_str("\"hello\"").unwrap(),
                ".tokens.bob" => {
                    serde_json::from_str(r#"{ "name": "bob", "balance": 0 }"#).unwrap()
                }
                ".settings.paused" => serde_json::from_str("false").unwrap(),
                _ => serde_json::from_str("{}").unwrap(),
            }

            // serde_json::from_str("\"bonjour\"").unwrap()
        }

        async fn del(key: &str) {
            println!("{}", &format!("del: {}", key));
        }
    }

    // fn test<T: KvStorage>() {
    //     T::put("foo", JsValue::from("bar"));
    // }

    #[derive(Serialize, Deserialize, Debug)]
    enum Bob {
        Foo,
        Bar,
    }

    #[kv_storage_macro(kv = "Kv")]
    struct State {
        name: String,
        #[kv_storage_macro(subpath)]
        settings: Settings,
        #[kv_storage_macro(map)]
        tokens: Token,
    }

    #[kv_storage_macro(kv = "Kv", subpath)]
    struct Settings {
        paused: bool,
        rate: u32,
        #[kv_storage_macro(subpath)]
        sub_settings: SubSettings,
    }

    #[kv_storage_macro(kv = "Kv", subpath)]
    struct SubSettings {
        sub: bool,
    }

    // #[kv_storage_macro(kv = "Kv", subpath)]
    #[derive(Serialize, Deserialize, Debug)]
    struct Token {
        name: String,
        balance: u32,
    }

    #[tokio::test]
    async fn test_macro() {
        let name = State::name().value().await;
        let bob = State::tokens("bob").value().await;
        let paused = State::settings().paused().value().await;

        println!("{}, {:?}, {}", name, bob, paused);

        // let paused = State::settings().sub_settings().sub().value().await;
        // let value = paused.value().await;

        // State::name().set_value(String::from("hello")).await;
        // let name = State::name().value().await;
        //
        // State::tokens("PTY")
        //     .set_value(Token {
        //         name: String::from("hello"),
        //         balances: 0,
        //     })
        //     .await;
        //
        // println!("TEST {:?}", name);
        // // let name =
        // // println!("TEST {:?}", tokio_test::block_on(state().value()));
        // // foo();
    }
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
