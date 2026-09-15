"""Storage shelf."""
import os


def locate(item):
    return item


class Base:
    def move(self, item):
        return os.path.join("shelf", item)


class Shelf(Base):
    def place(self, item):
        return self.move(item)


class Broken(Shelf, Missing):
    pass


class Posix(os.PathLike):
    pass
