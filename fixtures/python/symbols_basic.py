"""Fixture for basic symbol extraction."""


def train(epochs):
    return epochs


class Model:
    def fit(self, data):
        return data

    async def predict(self):
        return None


def outer():
    def inner():
        return 1

    return inner


@decorator
def served():
    return True


class Outer:
    class Inner:
        def method(self):
            return self
