# starscript

A rust client for the Space Traders API V2

## Info

[PGAdmin](https://admin.starscript.factris.dev/)
[Repo](https://github.com/brct-james/starscript)

## Setup

You must have a postgres server running locally. The compose.yaml can be used to create one in a container by renaming `example_postgres_secrets.env` to `postgres_secrets.env` and setting your own password, then running `docker compose up -d`. You can then run the server like `DATABASE_URL="postgresql://starscript:<db password here>@localhost:5432/starscript" cargo run` inserting the password you set where directed.

## Reference

https://tokio.rs/tokio/tutorial/channels
https://docs.rs/tokio/1.17.0/tokio/sync/broadcast/index.html
https://docs.rs/tokio/1.17.0/tokio/sync/watch/index.html
https://docs.rs/tokio/1.17.0/tokio/sync/watch/struct.Receiver.html
https://docs.rs/flume/latest/flume/index.html
https://docs.rs/flume/latest/flume/struct.Receiver.html
https://docs.rs/flume/0.9.2/flume/enum.TryRecvError.html
https://docs.rs/flume/latest/flume/fn.bounded.html
https://docs.rs/flume/latest/flume/struct.Sender.html#method.try_send
https://tokio.rs/tokio/tutorial/channels
