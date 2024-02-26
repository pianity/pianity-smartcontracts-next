use warp_scarcity::action::Scarcity;

use crate::state::State;

pub async fn is_op(address: &str) -> bool {
    State::settings()
        .operators()
        .get()
        .await
        .contains(&address.into())
}

pub async fn is_super_op(address: &str) -> bool {
    State::settings()
        .super_operators()
        .get()
        .await
        .contains(&address.into())
}

// TODO: This code is mostly duplicated from the Shuffle contract. It should be refactored to be
// shared instead.
pub struct NftId {
    pub edition: u32,
    pub scarcity: Scarcity,
    pub base_id: String,
}

impl ToString for NftId {
    fn to_string(&self) -> String {
        format!(
            "{}-{}-{}",
            self.edition,
            self.scarcity.to_string(),
            self.base_id
        )
    }
}

pub enum NftIdParseError {
    Scarcity,
    Edition,
    Shape,
}

impl TryFrom<&str> for NftId {
    type Error = NftIdParseError;

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        let splited = {
            let mut splited = id.splitn(3, '-');

            (
                splited.next().ok_or(NftIdParseError::Shape)?,
                splited.next().ok_or(NftIdParseError::Shape)?,
                splited.next().ok_or(NftIdParseError::Shape)?,
            )
        };

        let scarcity = Scarcity::try_from(splited.1).map_err(|_| NftIdParseError::Scarcity)?;
        let edition = splited
            .0
            .parse::<u32>()
            .map_err(|_| NftIdParseError::Edition)?;

        if !matches!(scarcity, Scarcity::Limited(_)) {
            if edition > Option::<u32>::from(&scarcity).unwrap() {
                return Err(NftIdParseError::Edition);
            }
        }

        Ok(Self {
            base_id: splited.2.to_string(),
            scarcity,
            edition,
        })
    }
}

pub struct ShuffleId {
    pub base_id: String,
}

impl ToString for ShuffleId {
    fn to_string(&self) -> String {
        format!("SHUFFLE-{}", self.base_id.clone())
    }
}

impl TryFrom<&str> for ShuffleId {
    type Error = ();

    fn try_from(id: &str) -> Result<Self, Self::Error> {
        let shuffle_split = id.splitn(2, '-').collect::<Vec<&str>>();

        if shuffle_split.len() == 2 && shuffle_split[0] == "SHUFFLE" {
            Ok(Self {
                base_id: shuffle_split[1].to_string(),
            })
        } else {
            Err(())
        }
    }
}

pub enum TokenId {
    Nft(NftId),
    Shuffle(ShuffleId),
    Token(String),
}

impl From<&str> for TokenId {
    fn from(token_id: &str) -> Self {
        if let Ok(shuffle) = ShuffleId::try_from(token_id) {
            Self::Shuffle(shuffle)
        } else if let Ok(nft) = NftId::try_from(token_id) {
            Self::Nft(nft)
        } else {
            Self::Token(token_id.to_string())
        }
    }
}
