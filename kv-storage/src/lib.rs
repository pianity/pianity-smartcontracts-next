use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

pub use kv_macro::kv_storage as kv;

#[async_trait(?Send)]
pub trait KvStorage {
    async fn put<T: Serialize>(key: &str, value: &T);
    async fn get<T: DeserializeOwned>(key: &str) -> Option<T>;
    // async fn del(key: &str);
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
    use std::collections::HashMap;

    use crate::{kv, KvStorage};
    use async_trait::async_trait;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json;

    pub struct Kv;

    #[async_trait(?Send)]
    impl KvStorage for Kv {
        async fn put<T: Serialize>(key: &str, value: &T) {
            println!(
                "{}",
                &format!("put: {} = {:?}", key, serde_json::to_string(value).unwrap())
            );
        }

        async fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
            println!("{}", &format!("get: {}", key));

            match key {
                ".the_name" => Some(serde_json::from_str("\"hello\"").unwrap()),
                // ".tokens.bob" => {
                //     Some(serde_json::from_str(r#"{ "name": "bob", "balance": 0 }"#).unwrap())
                // }
                ".tokens.PTY.name" => Some(serde_json::from_str(r#""PTY""#).unwrap()),
                ".tokens.PTY.balances.bob" => Some(serde_json::from_str(r#"123"#).unwrap()),
                ".settings.paused" => Some(serde_json::from_str("false").unwrap()),
                _ => None,
            }

            // serde_json::from_str("\"bonjour\"").unwrap()
        }

        // async fn del(key: &str) {
        //     println!("{}", &format!("del: {}", key));
        // }
    }

    #[kv(impl = "Kv")]
    struct State {
        the_name: Option<String>,
        #[kv(subpath)]
        settings: Settings,
        #[kv(map, subpath)]
        tokens: Token,
        // tokens: u32,
    }

    #[kv(impl = "Kv", subpath)]
    struct Settings {
        paused: bool,
        rate: u32,
    }

    #[kv(impl = "Kv", subpath)]
    struct Token {
        name: String,
        #[kv(map)]
        balances: u32,
    }

    fn capitalize(string: &str) -> String {
        let (first, rest) = string.split_at(1);
        format!("{}{}", first.to_ascii_uppercase(), rest)
    }

    fn test() -> Result<(), &'static str> {
        let mut test = Some(Some(Some("hello")));

        let test2 = test.ok_or("err")?.ok_or("err")?.ok_or("err")?;

        Ok(())

        // let string = "hello_world";
        // let splited = string
        //     .split('_')
        //     .map(capitalize)
        //     .collect::<Vec<_>>()
        //     .join("");
        // Some(1).map(|x| x + 1);
        // // String::default().split_at_mut;
    }

    fn overwrite() {
        println!("hello");
    }

    #[tokio::test]
    async fn test_macro() {
        // State::default().init().await;
        let init_state = State {
            // name: Some("hello".to_string()),
            the_name: None,
            settings: Settings {
                paused: false,
                rate: 0,
            },
            tokens: HashMap::from([
                (
                    String::from("PTY"),
                    Token {
                        name: String::from("PTYname"),
                        balances: HashMap::from([
                            (String::from("bob"), 123),
                            (String::from("alice"), 321),
                        ]),
                    },
                ),
                (
                    String::from("PIA"),
                    Token {
                        name: String::from("PIAname"),
                        balances: HashMap::from([
                            (String::from("alfred"), 456),
                            (String::from("david"), 654),
                        ]),
                    },
                ),
            ]),
        }
        .init()
        .await;

        println!("-------------");

        println!("thename: {:?}", State::the_name().get().await);

        // let bobsBalance = State::tokens("PTY")
        //     .init_default()
        //     .await
        //     .balances("bob")
        //     .peek()
        //     .await;
        // println!("bobsBalance: {:?}", bobsBalance);

        // State::tokens("BOB").set(&Token::default()).await;

        // let token = State::tokens("PTY")
        //     .init_default()
        //     .await
        //     .balances("bob")
        //     .init_default()
        //     .await
        //     .map(|x| x + 1)
        //     .await;

        // .get()
        // .await;

        // println!("{}", token);

        // let name = State::tokens("bob").name().value().await;
        // let name = State::tokens("bob").init();
        // println!("{:?}", name);

        // let name = State::name().value().await;
        // // let bob = State::tokens("PTY").name().value().await;
        // let paused = State::settings().paused().value().await;
        //
        // // let op = Some(1).unwrap_or_default
        //
        // let map = HashMap::from([("bob", 1)]);
        // // map.entry("bob").or_default
        //
        // let balance = State::settings()
        //     .init()
        //     .or_init_default()
        //     // .await()
        //     .balances("bob")
        //     .value()
        //     // .set()
        //     .await;
        //
        // println!("bob {:?}", balance);
        //
        // // let state = State::init_or(State {
        // //     name: String::from("Pianity"),
        // //     settings: Settings::default(),
        // //     tokens: HashMap::from([(
        // //         String::from("bob"),
        // //         Token {
        // //             name: String::from("PTY"),
        // //             balances: HashMap::new(),
        // //         },
        // //     )]),
        // // });
        // // let token = State::tokens("bob"); // <-- Wouldn't be `Token`, but instead a `MaybeToken`
        // // let token = State::tokens("bob").or_else(|| MyError)?;
        // // let token = State::tokens("bob").or_init(|| );
        // // token.name().set_value
        //
        // // println!("{}, {:?}, {}", name, bob, paused);
        //
        // // let paused = State::settings().sub_settings().sub().value().await;
        // // let value = paused.value().await;
        //
        // // State::name().set_value(String::from("hello")).await;
        // // let name = State::name().value().await;
        // //
        // // State::tokens("PTY")
        // //     .set_value(Token {
        // //         name: String::from("hello"),
        // //         balances: 0,
        // //     })
        // //     .await;
        // //
        // // println!("TEST {:?}", name);
        // // // let name =
        // // // println!("TEST {:?}", tokio_test::block_on(state().value()));
        // // // foo();
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
