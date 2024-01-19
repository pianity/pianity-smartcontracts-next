use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub use kv_macro::kv_storage as kv;

#[async_trait(?Send)]
pub trait KvStorage {
    async fn put<T: Serialize>(key: &str, value: &T);
    async fn del(key: &str);
    async fn get<T: DeserializeOwned>(key: &str) -> Option<T>;
    async fn keys(
        gte: Option<&str>,
        lt: Option<&str>,
        reverse: Option<bool>,
        limit: Option<u32>,
    ) -> Vec<String>;
    async fn map<T: DeserializeOwned>(
        gte: Option<&str>,
        lt: Option<&str>,
        reverse: Option<bool>,
        limit: Option<u32>,
    ) -> Vec<(String, T)>;
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{OnceCell, RefCell},
        collections::HashMap,
    };

    use crate::{kv, KvStorage};
    use async_trait::async_trait;
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json;

    thread_local! {
        static STORE: RefCell<HashMap<String, String>> = RefCell::default();
    }

    pub struct Kv;

    #[async_trait(?Send)]
    impl KvStorage for Kv {
        async fn put<T: Serialize>(key: &str, value: &T) {
            println!("put: {} = {:?}", key, serde_json::to_string(value).unwrap());

            STORE.with(|store| {
                store
                    .borrow_mut()
                    .insert(String::from(key), serde_json::to_string(value).unwrap());
            });
        }

        async fn del(key: &str) {
            println!("del: {}", key);

            STORE.with(|store| {
                store
                    .borrow_mut()
                    .remove(key)
                    .expect(&format!("couldn't delete: {}", key));
            });
        }

        async fn get<T: DeserializeOwned>(key: &str) -> Option<T> {
            println!("get: {}", key);

            STORE.with(|store| {
                store
                    .borrow()
                    .get(key)
                    .map(|value| serde_json::from_str(value).unwrap())
            })
        }

        async fn keys(
            gte: Option<&str>,
            lt: Option<&str>,
            reverse: Option<bool>,
            limit: Option<u32>,
        ) -> Vec<String> {
            println!("keys: {:?}, {:?}, {:?}, {:?}", gte, lt, reverse, limit);

            let keys = STORE.with(|store| {
                store
                    .borrow()
                    .iter()
                    .filter(|(key, _)| {
                        let (gte, lt) = (String::from(gte.unwrap()), String::from(lt.unwrap()));

                        **key >= gte && **key < lt
                    })
                    .map(|(key, _)| key.clone())
                    .collect()
            });

            println!("keys: {:?}", keys);
            keys
        }

        async fn map<T: DeserializeOwned>(
            gte: Option<&str>,
            lt: Option<&str>,
            reverse: Option<bool>,
            limit: Option<u32>,
        ) -> Vec<(String, T)> {
            println!("map: {:?}, {:?}, {:?}, {:?}", gte, lt, reverse, limit);

            let map: Vec<(String, T)> = STORE.with(|store| {
                store
                    .borrow()
                    .iter()
                    .filter(|(key, _)| {
                        let (gte, lt) = (String::from(gte.unwrap()), String::from(lt.unwrap()));

                        **key >= gte && **key < lt
                    })
                    .map(|(key, value)| (key.clone(), serde_json::from_str(value).unwrap()))
                    .collect()
            });

            println!(
                "map: {:?}",
                map.iter()
                    .map(|(key, _)| key.clone())
                    .collect::<Vec<String>>()
            );
            map
        }
    }

    #[kv(impl = "Kv", subpath)]
    struct Friend {
        #[kv(map)]
        relations: String,
    }

    #[kv(impl = "Kv", subpath)]
    struct Person {
        name: String,
        age: u32,
        #[kv(map, subpath)]
        friends: Friend,
    }

    #[kv(impl = "Kv")]
    struct State {
        #[kv(map, subpath)]
        people: Person,
        #[kv(map)]
        colors: String,

        the_name: Option<String>,
        #[kv(subpath)]
        settings: Settings,
        #[kv(map, subpath)]
        tokens: Token,
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
        tx_id: Option<String>,
    }

    fn capitalize(string: &str) -> String {
        let (first, rest) = string.split_at(1);
        format!("{}{}", first.to_ascii_uppercase(), rest)
    }

    fn test() -> Result<(), &'static str> {
        let mut test = Some(Some(Some("hello")));

        let test2 = test.ok_or("err")?.ok_or("err")?.ok_or("err")?;

        Ok(())
    }

    fn overwrite() {
        println!("hello");
    }

    #[tokio::test]
    async fn test_macro() {
        State {
            people: HashMap::from([
                (
                    "noom".to_string(),
                    Person {
                        name: "noom".to_string(),
                        age: 123,
                        friends: HashMap::from([
                            (
                                "bob".to_string(),
                                Friend {
                                    relations: HashMap::from([
                                        ("bob".to_string(), "noom".to_string()),
                                        ("alice".to_string(), "noom".to_string()),
                                        ("alfred".to_string(), "noom".to_string()),
                                        ("david".to_string(), "noom".to_string()),
                                    ]),
                                },
                            ),
                            (
                                "alice".to_string(),
                                Friend {
                                    relations: HashMap::from([
                                        ("bob".to_string(), "noom".to_string()),
                                        ("alice".to_string(), "noom".to_string()),
                                        ("alfred".to_string(), "noom".to_string()),
                                        ("david".to_string(), "noom".to_string()),
                                    ]),
                                },
                            ),
                            (
                                "alfred".to_string(),
                                Friend {
                                    relations: HashMap::from([
                                        ("bob".to_string(), "noom".to_string()),
                                        ("alice".to_string(), "noom".to_string()),
                                        ("alfred".to_string(), "noom".to_string()),
                                        ("david".to_string(), "noom".to_string()),
                                    ]),
                                },
                            ),
                        ]),
                    },
                ),
                (
                    "bob".to_string(),
                    Person {
                        name: "bob".to_string(),
                        age: 123,
                        friends: HashMap::new(),
                    },
                ),
            ]),
            colors: HashMap::from([
                ("red".to_string(), "ff0000".to_string()),
                ("green".to_string(), "00ff00".to_string()),
                ("blue".to_string(), "0000ff".to_string()),
            ]),

            the_name: None,
            settings: Settings {
                paused: false,
                rate: 0,
            },
            tokens: HashMap::from([
                (
                    String::from("PTY"),
                    Token {
                        tx_id: None,
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
                        tx_id: None,
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

        let pty = State::tokens("PTY")
            .ok_or("err")
            .await
            .unwrap()
            .tx_id()
            .get()
            .await;

        println!("{:?}", pty);

        // let tokens = State::list_tokens().await;
        //
        // println!("tokens length: {}", tokens.len());
        //
        // println!("colors: {:?}", State::list_colors().await);

        // State::people("noom").ok_or("err")?;

        // State::delete_tokens("PTY").await;
        // State::delete_people("noom").await;
        // State::delete_colors("red").await;
        // State::delete_colors("green").await;

        // println!(
        //     "thename: {:?}",
        //     State::tokens("PTY")
        //         .init_default()
        //         .await
        //         .list_balances()
        //         .await
        // );
        //
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
