🦀 pastemd
=============

`pastemd` is a lightweight and pluggable library for creating pastebin services.

An example/template can be found at <https://github.com/hkauso/sealable>.

***

Pastemd uses a [Dorsal](https://github.com/stellularorg/dorsal)-based backend which supports multiple database types. Please see the [original source](https://github.com/caffeineee/pasties) for more current development.

Pastemd only handles the database and API routes (with Axum). It is encouraged you build your own frontend around the API or use the source to learn how to write your own pastebin API.
