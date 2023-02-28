use chrono::Utc;
use mongodb::{
    bson::{doc, from_bson, to_document, Bson, Document},
    options::ReplaceOptions,
    Collection,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AgentIdentity {
    symbol: String,
    token: String,
    created_timestamp: String,
}

impl AgentIdentity {
    pub fn new(symbol: String, token: String) -> Self {
        Self {
            symbol,
            token,
            created_timestamp: Utc::now().to_string(),
        }
    }

    pub fn get_symbol(&self) -> String {
        self.symbol.to_string()
    }

    pub fn get_token(&self) -> String {
        self.token.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct IdentityManager {
    agent_identity_table: Collection<Document>,
}

impl IdentityManager {
    pub fn new(agent_identity_table: Collection<Document>) -> Self {
        Self {
            agent_identity_table,
        }
    }

    pub async fn get_agent(&self, symbol: String) -> Option<AgentIdentity> {
        let standardized_symbol = symbol.to_uppercase();
        let found = self
            .agent_identity_table
            .find_one(Some(doc! {"symbol": standardized_symbol.to_string()}), None)
            .await
            .unwrap();

        match found {
            Some(ref document) => {
                let identity: AgentIdentity =
                    from_bson(Bson::Document(document.to_owned())).unwrap();
                return Some(identity);
            }
            None => return None,
        }
    }

    pub async fn save_agent(&self, symbol: String, token: String) {
        let standardized_symbol = symbol.to_uppercase();
        self.agent_identity_table
            .replace_one(
                doc! {"symbol": standardized_symbol.to_string()},
                to_document(&AgentIdentity::new(
                    standardized_symbol.to_string(),
                    token.to_string(),
                ))
                .unwrap(),
                Some(ReplaceOptions::builder().upsert(true).build()),
            )
            .await
            .unwrap();
    }
}
