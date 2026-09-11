+++
title = "SQL doesn't have to be complicated"
description = "Ditch ORM, improve performance, half LOC; That's how it works"
date = 2026-09-11
template = "blog-entry.html"
+++

From my experience writing user interfaces in Flutter, I made great experiences with [Drift](https://drift.simonbinder.eu/) after spending a few years writing SQL queries manually. I could just automatically generate correct migrations from my data structures, got build-in type safety and could easily `watch` queries to display continuously updating results in my UI. All while bloat was at an acceptable level. Not enough benefits to migrate existing apps, but I wouldn't build a new app without this.

For a yet unreleased project I'm writing a server in rust, that needs to access a database (like most servers do). I learned from my experience in Dart and researched ORMs. Diesel and SeaORM seem to be the most widely used, so I looked into them, found SeaORM syntax feeling familiar, and went with it. I was somewhat _appalled_ by the heap of dependencies this introduced, but went with it since many of those dependencies looked like something I might introduce for future features. Building the server I noticed a few things that were different from accessing a DB for UI: It rarely made sense to watch values, getting and setting were much more important. Also the kind of queries I wanted to write where different: No more "read these rows from that table", and much more conditions, reading selected columns and even some "recursive" (bounded depth, lookup in the same table) queries.

I found myself fighting the SeaORMs DSL and doing the kind of unsafe things I wanted my ORM to prevent. And as the project got more and more feature complete, I still didn't need all the dependencies SeaORM introduced. So I decided I needed to change things. There were still 3 options:
- Use Diesel, in hindsight the primitives looked much more like what I actually wanted and it had less dependencies
- Use [sqlite](https://crates.io/crates/sqlite) directly and do everything manually
- Use [sqlx](https://crates.io/crates/sqlx)

sqlx is an interesting option. SeaORM is based on it, and it provides some macros for checking prepared statement syntax and doing migrations. It is low-level enough not to introduce half of crates.io. And most importantly: Switching to it, I would make switching to raw sqlite fairly easy.

The migration went very smoothly: I could copy the CREATE TABLE statements from the existing database, and even clean them up a little - I now had better control over foreign keys and constraints. Most SeaORM queries translated to simple SQL queries of similar length. There was even room for some improvements like combining multiple queries into one or using `ON CONFLICT ... DO UPDATE` instead of reading the data-base before inserting.

Because I now use a lower-level connection I took the chance to read parts of the [official sqlite documentation](https://sqlite.org/docs.html). Learned how types are resolved, and that I had to periodically run `PRAGMA optimize;`. I removed the table-row structs and now read data into task specific structs.

That whole migration had a `+916 -1925` diff, with over 1000 deletions in `Cargo.lock`. Build times went down, because of the reduction in dependencies and having fewer complex macros. And the app is now subjectively more snappy.

Overall I'd consider this a success. Let's see how evolving the code works out.
