# slice-rbtree
A `#[no_std]` [Red-Black tree][2], fully packed in a single slice of bytes
Originally developed for storing data in [Solana][0] [Accounts][1], this crate allows you to
access tree nodes without deserializing the whole tree. It is useful when you have a huge
tree in raw memory, but want to interact only with a few values at a time.

There are two core type in this crate: `RBTree` and `RBForest`

## `RBTree`
As name suggests, it is a [Red-Black tree][2], contained in the slice of bytes.
The API is similar to [BTreeMap][3] with a few exceptions, such as [Entry API][4], but it will be added in the future releases.
```rust
use slice_rbtree::{tree_size, RBTree};
// RBTree requires input slice to have a proper size
let size = tree_size(50, 50, 10);
let mut buffer = vec![0; size];
let mut movie_reviews: RBTree<String, String, 50, 50> = RBTree::init_slice(&mut buffer).unwrap();

// review some movies.
movie_reviews.insert("Office Space".to_string(),       "Deals with real issues in the workplace.".to_string());
movie_reviews.insert("Pulp Fiction".to_string(),       "Masterpiece.".to_string());
movie_reviews.insert("The Godfather".to_string(),      "Very enjoyable.".to_string());
movie_reviews.insert("The Blues Brothers".to_string(), "Eye lyked it a lot.".to_string());

// check for a specific one.
if !movie_reviews.contains_key("Les Misérables") {
    println!("We've got {} reviews, but Les Misérables ain't one.",
             movie_reviews.len());
}

// oops, this review has a lot of spelling mistakes, let's delete it.
movie_reviews.remove("The Blues Brothers");

// look up the values associated with some keys.
let to_find = ["Up!".to_string(), "Office Space".to_string()];
for movie in &to_find {
    match movie_reviews.get(movie) {
       Some(review) => println!("{movie}: {review}"),
       None => println!("{movie} is unreviewed.")
    }
}

// iterate over everything.
for (movie, review) in movie_reviews.pairs() {
    println!("{movie}: \"{review}\"");
}
```
[0]: https://docs.solana.com/
[1]: https://docs.rs/solana-sdk/latest/solana_sdk/account/struct.Account.html
[2]: https://en.wikipedia.org/wiki/Red%E2%80%93black_tree
[3]: https://doc.rust-lang.org/stable/std/collections/btree_map/struct.BTreeMap.html
[4]: https://doc.rust-lang.org/stable/std/collections/struct.BTreeMap.html#method.entry
