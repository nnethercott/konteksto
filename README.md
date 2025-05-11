# Konteksto
Solving [Contexto](https://contexto.me/en/) using greedy [hill climbing](https://en.wikipedia.org/wiki/Hill_climbing) with momentum.

# Running the code
You can run Konteksto in either standalone mode as a CLI or a web app. In standalone the solver iterates automatically towards a solution, while in the web app users manually drive the solver state through word submissions. In both cases we need a running Qdrant instance to handle vector search.

## CLI
An executable to configure and run the hill climbing algorithm for a given (language, game_id) pair.


## web
A wrapper around Contexto built using axum, sqlx, maud, and htmx providing word suggestions.

# Elements of the solution
* Inspecting contexto's page source we find the file `/static/js/gameApi.js` which lists the API endpoints for contexto;
  * base url `https://api.contexto.me/machado`
  * `${baseUrl}/${language}/game/${gameId}/${word}`: scores guesses for a given game
  * `${baseUrl}/${language}/top/${gameId}`: lists top words for a given game

* A database of ~10k words is built by scraping over 500 games using the /top/${gameId} endpoint. This allows us to have a well-defined search space
  * previous attempts to build a dataset using [nltk](https://www.nltk.org/howto/corpus.html) or [publicly available lists](https://github.com/dwyl/english-words) were limited due to size (O(10^5) words) and non-overlap with contexto's own internal list.
* [qdrant](https://github.com/qdrant/qdrant) and [fastembed](https://github.com/qdrant/fastembed) are used to generate and index embeddings per language. This results in a distinct Qdrant collection for English, Portuguese, and Spanish games.
  * we use [sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2](https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2) to encode non-english words, and [BAAI/bge-small-en-v1.5](https://huggingface.co/BAAI/bge-small-en-v1.5) to encode English words.
* `Konteksto-engine` leverages the Rust client for qdrant and some linalg packages to implement the hill climbing algorithm.

## Greedy hill climbing
An outline of the algorithm is provided below.
![](assets/algo.png)
