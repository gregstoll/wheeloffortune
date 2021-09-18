# wheeloffortune
Solver for Wheel of Fortune game

See [a live demo here](https://gregstoll.com/wheeloffortune/)!

The word list is taken from [Google Books Ngrams](https://storage.googleapis.com/books/ngrams/books/datasetsv3.html), specifically the 1-grams from the 20200217 release. The word list that the app uses is in [`data/processed/word_frequency.txt`](https://github.com/gregstoll/wheeloffortune/blob/main/data/processed/word_frequency.txt). If you want to generate it:
- Create an empty directory under `data/raw`
- Run the `data/downloadRawCorpus.py` script, which will download and unzip the ngram files into the `data/raw` directory. Note that these files total around 26 GB in size.
- Run the [`process_corpus`](https://github.com/gregstoll/wheeloffortune/blob/main/process_corpus/src/main.rs) script in release mode with `cargo run --release`. This will generate the word frequency file.
  - Note that [`process_corpus.slow.py`](https://github.com/gregstoll/wheeloffortune/blob/main/process_corpus/process_corpus.slow.py) does the same thing, but slower than the release Rust version.

The [`search_corpus`](https://github.com/gregstoll/wheeloffortune/blob/main/search_corpus/src/main.rs) script searches through the word frequency file for the specified pattern.

"Wheel of Fortune®" is a registered trademark of Califon Productions, Inc.
