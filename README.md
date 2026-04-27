# Blog Live View Counter

A live, WebSocket-driven view counter built with Rust on Cloudflare Workers and Durable Objects.

## Features

- **Live Updates**: View counts update in real-time via WebSockets as users join and leave.
- **Persistent Storage**: All-time view counts are persisted to SQLite (D1) using Durable Objects' local storage.
- **Efficient**: Uses WebSocket hibernation to minimize costs and resource usage.

## Architecture

For a detailed breakdown of how this was built, see the blog post:
[A live view counter on Cloudflare Durable Objects](https://kgdev.me/posts/blog-view-counter)

## Tech Stack

- **Language**: Rust
- **Platform**: Cloudflare Workers
- **State/WebSockets**: Cloudflare Durable Objects
- **Storage**: SQLite (Durable Objects local storage)
- **Frontend**: Astro (Integration example)
