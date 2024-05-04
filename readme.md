# Chat server

This is the companion code repository for the article [Beginner's Guide to Concurrent Programming: Coding a Multithreaded Chat Server using Tokio](https://github.com/pretzelhammer/rust-blog/blob/master/posts/chat-server.md).

You can find the full source code for any example from the article in the [examples directory](https://github.com/pretzelhammer/chat-server/tree/main/examples).

This project uses [just](https://github.com/casey/just) to run commands.

To run an example
```
just example {number}
```

To connect to any running example
```
just telnet
```

To connect to any running example >=09 with a nicer TUI client
```
just chat
```

To see a diff between any two examples run
```
just diff {number} {number}
```

Run the final Production Ready<sup>TM</sup> server code with
```
just server
```

And as before you can connect to it with a TUI client by running
```
just chat
```

And if gets lonely run chat bots with
```
just bots
```

To get a list and description of all commands run
```
just list
```

## License

This code is dual-licensed under [Apache License Version 2.0](./license-apache) or [MIT License](./license-mit), at your option.
