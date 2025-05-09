# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "aiohttp",
#     "uvloop",
# ]
# ///
from enum import Enum
import asyncio
import aiohttp
from typing import List
import uvloop

CONTEXTO_API_URL: str = "https://api.contexto.me/machado"


class Lang(str, Enum):
    EN = "en"
    PT = "pt-br"
    ES = "es"

    @property
    def url(self):
        return f"{CONTEXTO_API_URL}/{self}"


class Scraper:
    """A class for building a reasonable vocab as a search space for Contexto"""

    def __init__(self, lang: Lang, past_n_games: int = 100):
        self.urls = [f"{lang.url}/top/{i + 1}" for i in range(past_n_games)]

    @staticmethod
    async def fetch_words(session: aiohttp.ClientSession, url: str) -> List[str]:
        async with session.get(url) as response:
            if response.status == 200:
                json = await response.json()
                return json.get("words", [])
            return []

    async def build_corpus(self) -> List[str]:
        """return unique words from past n games"""
        async with aiohttp.ClientSession() as http:
            futures = [self.fetch_words(http, url) for url in self.urls]
            words = await asyncio.gather(*futures)

        words = sum(words, [])
        return list(set(words))


if __name__ == "__main__":
    import os
    import pathlib
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("--lang", "-l", help="language: 'en', 'pt-br', 'es'", type=Lang, default=Lang.EN)
    parser.add_argument("--out-file", "-o", help="output file for word dump", type=pathlib.Path)
    parser.add_argument(
        "--n-past-games",
        "-n",
        help="how many past games to include in dump",
        type=int,
        default=100,
    )
    args = parser.parse_args()

    scraper = Scraper(args.lang, args.n_past_games)
    corpus = uvloop.run(scraper.build_corpus())

    os.makedirs(args.out_file.parent, exist_ok=True)
    with open(args.out_file, "w") as f:
        f.writelines((c + "\n" for c in corpus))
