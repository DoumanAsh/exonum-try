use ::exonum::api::{ServiceApiState, ServiceApiBuilder, self};
use ::exonum::blockchain::{Transaction, Service, TransactionSet};
use ::exonum::crypto::{Hash, PublicKey};
use ::exonum::node::TransactionSend;
use ::exonum::messages::{RawTransaction};
use ::exonum::storage::{Snapshot};
use ::exonum::encoding;

use super::article;
use super::article::Article;

struct Auction;

#[derive(Serialize, Deserialize)]
pub struct TransactionResponse {
    pub hash: Hash,
}

///Query to retrieve particular article
#[derive(Deserialize)]
pub struct ArticleQuery {
    pub pub_key: PublicKey,
}

impl Auction {
    fn post_transaction(state: &ServiceApiState, query: article::Transactions) -> api::Result<TransactionResponse> {
        let transaction: Box<Transaction> = query.into();
        let hash = transaction.hash();
        state.sender().send(transaction)?;
        Ok(TransactionResponse { hash })
    }

    /// Get article by key
    fn get_article(state: &ServiceApiState, query: ArticleQuery) -> api::Result<Article> {
        let snapshot = state.snapshot();
        let schema = article::Schema::new(snapshot);
        schema.article(&query.pub_key).ok_or_else(|| api::Error::NotFound("Article no found".to_owned()))
    }

    /// Retrieve all articles
    fn get_articles(state: &ServiceApiState, _query: ()) -> api::Result<Vec<Article>> {
        let snapshot = state.snapshot();
        let schema = article::Schema::new(snapshot);
        let idx = schema.articles();
        let result = idx.values().collect();
        Ok(result)
    }

    fn wire(builder: &mut ServiceApiBuilder) {
        // Binds handlers to specific routes.
        builder.public_scope()
               .endpoint("v1/article", Self::get_article)
               .endpoint("v1/articles", Self::get_articles)
               //I wonder if it is actually possible to force
               //particular transaction?
               //Kinda couldn't find it while skimming docs
               .endpoint_mut("v1/article/new", Self::post_transaction)
               .endpoint_mut("v1/article/open", Self::post_transaction)
               .endpoint_mut("v1/article/close", Self::post_transaction)
               .endpoint_mut("v1/article/bid", Self::post_transaction);
    }
}

pub struct AuctionService;

impl Service for AuctionService {
    fn service_name(&self) -> &'static str { "auction" }

    fn service_id(&self) -> u16 { article::SERVICE_ID }

    fn tx_from_raw(&self, raw: RawTransaction) -> Result<Box<Transaction>, encoding::Error> {
        let tx = article::Transactions::tx_from_raw(raw)?;
        Ok(tx.into())
    }

    fn state_hash(&self, _: &Snapshot) -> Vec<Hash> {
        vec![]
    }

    fn wire_api(&self, builder: &mut ServiceApiBuilder) {
        Auction::wire(builder)
    }
}
