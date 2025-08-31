# `v0.4.0`
- *Added:* New method `ArticleId::formatted_len` returns the length of the formatted string, but much more efficiently than allocating the string itself.
- *Removed:* `Validated<S>` no longer implements `PartialEq<ArticleId>`.
