# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "fastembed",
# ]
# ///
from fastembed import TextEmbedding

DEFAULT_MODEL_ID = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"


class EmbeddingDump:
    """A class for lazy embedding docs and dumping to disk. We postpone decisions concerning
    vector properties until vector db collection creation"""

    def __init__(self, file: str, model_id: str = DEFAULT_MODEL_ID):
        self.model = TextEmbedding(model_id)
        self.file = file

    def entries(self):
        with open(self.file, "r") as f:
            docs = f.read().splitlines()

        # lazy embedding
        embeddings = self.model.embed(docs)

        for d, e in zip(docs, embeddings):
            yield {"word": d, "embedding": e.tolist()}


if __name__ == "__main__":
    import os
    import json
    import argparse
    import pathlib

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--in-file",
        "-i",
        help="file handle for newline seperated docs to embed",
        type=pathlib.Path,
    )
    parser.add_argument(
        "--out-file",
        "-o",
        help="path to file where to store the embedding dump",
        type=pathlib.Path,
    )
    parser.add_argument(
        "--model-id",
        "-m",
        help="id of model to embed docs with",
        default=DEFAULT_MODEL_ID,
    )
    args = parser.parse_args()

    embedder = EmbeddingDump(file=args.in_file, model_id=args.model_id)

    os.makedirs(args.out_file.parent, exist_ok=True)
    with open(args.out_file, "w") as f:
        # lazy dump embeddings to disk
        for entry in embedder.entries():
            f.write(json.dumps(entry) + "\n")
