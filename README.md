# exonum-try

Just trying out [exonum](https://github.com/exonum/exonum)

## API

Auction API is under `/api/services/auction/v1`

Available endpoints

- `/article` - Get article by public key
- `/articles` - Find all articles.
- `/article/new` - Transaction to create article in auction.
- `/article/open` - Transaction to open article in auction, should be created.
- `/article/close` - Transaction to close article in auction, should be created and open.
- `/article/bid` - Transaction to place new bid, it should be greater than the current one.
