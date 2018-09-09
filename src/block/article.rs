use ::std::fmt;
use ::std::error::Error;

use ::exonum::crypto::{PublicKey};
use ::exonum::storage::{Fork, MapIndex, Snapshot};
use ::exonum::blockchain::{ExecutionError, ExecutionResult, Transaction};
use ::exonum::messages::{Message};

pub const SERVICE_ID: u16 = 1;
const INIT_BID: u64 = 100;

mod indexes {
    pub const ARTICLES: &'static str = "cryptocurrency.articles";
}

encoding_struct! {
    struct Article {
        pub_key: &PublicKey,
        name: &str,
        bid: u64,
        is_open: bool,
    }
}

impl Article {
    pub fn new_bid(self, amount: u64) -> Self {
        Self::new(self.pub_key(), self.name(), amount, self.is_open())
    }

    pub fn open(self) -> Self {
        Self::new(self.pub_key(), self.name(), self.bid(), true)
    }

    pub fn close(self) -> Self {
        Self::new(self.pub_key(), self.name(), self.bid(), false)
    }
}

pub struct Schema<T> {
    view: T,
}

impl<T: AsRef<Snapshot>> Schema<T> {
    pub fn new(view: T) -> Self {
        Self { view }
    }

    pub fn articles(&self) -> MapIndex<&Snapshot, PublicKey, Article> {
        MapIndex::new(indexes::ARTICLES, self.view.as_ref())
    }

    // Utility method to quickly get a separate wallet from the storage
    pub fn article(&self, pub_key: &PublicKey) -> Option<Article> {
        self.articles().get(pub_key)
    }
}

impl<'a> Schema<&'a mut Fork> {
    pub fn articles_mut(&mut self) -> MapIndex<&mut Fork, PublicKey, Article> {
        MapIndex::new(indexes::ARTICLES, &mut self.view)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum TransactionError {
    AlreadyExists,
    Opened,
    Closed,
    InvalidBid,
    UnknownArticle,
}

impl Error for TransactionError {
    fn description(&self) -> &str {
        match self {
            &TransactionError::AlreadyExists => "Article already exists",
            &TransactionError::Opened => "Article is already open",
            &TransactionError::Closed => "Article is not open for bids",
            &TransactionError::InvalidBid => "New bid is less or equal to already existing",
            &TransactionError::UnknownArticle => "Bid for unknown article",
        }
    }
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<TransactionError> for ExecutionError {
    fn from(value: TransactionError) -> ExecutionError {
        ExecutionError::with_description(value as u8, value.description())
    }
}

transactions! {
    pub Transactions {
        const SERVICE_ID = SERVICE_ID;

        struct CreateArticle {
            pub_key: &PublicKey,
            name: &str,
        }

        struct OpenArticle {
            pub_key: &PublicKey,
            name: &str,
        }

        struct CloseArticle {
            pub_key: &PublicKey,
            name: &str,
        }

        struct MakeBid {
            to: &PublicKey,
            amount: u64,
        }
    }
}

impl Transaction for CreateArticle {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(view);

        if schema.article(self.pub_key()).is_none() {
            let article = Article::new(self.pub_key(), self.name(), INIT_BID, false);
            schema.articles_mut().put(self.pub_key(), article);
            Ok(())
        } else {
            Err(TransactionError::AlreadyExists.into())
        }
    }
}

impl Transaction for OpenArticle {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(view);

        let article = match schema.article(self.pub_key()) {
            Some(article) => article,
            None => return Err(TransactionError::UnknownArticle.into()),
        };

        if article.is_open() {
            return Err(TransactionError::Opened.into());
        }

        let article = article.open();
        schema.articles_mut().put(self.pub_key(), article);

        Ok(())
    }
}

impl Transaction for CloseArticle {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(view);

        let article = match schema.article(self.pub_key()) {
            Some(article) => article,
            None => return Err(TransactionError::UnknownArticle.into()),
        };

        if !article.is_open() {
            return Err(TransactionError::Closed.into());
        }

        let article = article.close();
        schema.articles_mut().put(self.pub_key(), article);

        Ok(())
    }
}

impl Transaction for MakeBid {
    fn verify(&self) -> bool {
        self.verify_signature(self.to())
    }

    fn execute(&self, view: &mut Fork) -> ExecutionResult {
        let mut schema = Schema::new(view);

        let article = match schema.article(self.to()) {
            Some(article) => article,
            None => return Err(TransactionError::UnknownArticle.into()),
        };

        if !article.is_open() {
            return Err(TransactionError::Closed.into());
        } else if article.bid() >= self.amount() {
            return Err(TransactionError::InvalidBid.into());
        }

        let article = article.new_bid(self.amount());
        schema.articles_mut().put(self.to(), article);

        Ok(())
    }
}
