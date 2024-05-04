# Chat server examples

Full source code for the examples from the article [Beginner's Guide to Concurrent Programming: Coding a Multithreaded Chat Server using Tokio](https://github.com/pretzelhammer/rust-blog/blob/master/posts/chat-server.md).

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
