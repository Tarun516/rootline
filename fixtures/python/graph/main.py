"""Entry point."""
import shop.store.shelf as shelf
from shop.cart import Cart
from .ghost import missing
from pathlib import Path


def run():
    cart = Cart()
    shelf.Shelf()
    unknown_thing()
    return Path.cwd()
