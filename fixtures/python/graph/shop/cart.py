"""Shopping cart."""
import os
from .. import helper
from .store.shelf import move, locate


class Cart:
    def __init__(self):
        self.items = []

    def add(self, item):
        self.items.append(item)
        return locate(item)

    def checkout(self):
        helper.assist()
        total = self.total()
        return total

    def total(self):
        return len(self.items)
