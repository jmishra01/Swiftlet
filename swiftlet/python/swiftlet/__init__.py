from typing import Callable, Dict
from abc import ABC

from ._swiftlet import *
import queue

# ---------------------------------------------------------------------------------------------------------
# ---------------------------
# Tree Class
# ---------------------------
# Add few function in Tree class
# ---------------------------------------------------------------------------------------------------------

def _iter_subtree(self):
    _queue = queue.Queue()
    _queue.put(self)
    while _queue.qsize() > 0:
        item = _queue.get()
        yield item
        for tree in item.get_children():
            if isinstance(tree, Tree):
                _queue.put(tree)

def _find_pred(self, pred: Callable):
    return filter(pred, self.iter_subtree())


def _find_data(self, data: str):
    return self.find_pred(lambda x: x.get_name() == data)


def _scan_values(self, pred: Callable):
    for child in self.get_children():
        if isinstance(child, Tree):
            for t in child.scan_values(pred):
                yield t
        else:
            if pred(child):
                yield child


def _find_token(self, token_type: str):
    self.scan_values(lambda t: t.get_terminal() == token_type)


def _pretty_print(self, space: str = ""):
    if len(self.get_children()) == 1 and not hasattr(self.get_children()[0], "get_children"):
        print(space, self.get_name(), "  ", end="")
        c = self.get_children()[0]
        if isinstance(c, Token):
            print(c.get_word())
        else:
            print(c)
        return

    print(space, self.get_name())
    space += '  '
    for child in self.get_children():
        if isinstance(child, Token):
            print(space, child.get_word())
        else:
            child.pretty_print(space)

Tree.pretty_print = _pretty_print
Tree.iter_subtree = _iter_subtree
Tree.find_pred = _find_pred
Tree.find_data = _find_data
Tree.scan_values = _scan_values
Tree.find_token = _find_token

# ---------------------------------------------------------------------------------------------------------

class ExceptionTreeType(Exception):
    pass

class Transformer(ABC):
    """
    Transform
    =========

    """
    def __init__(self, token_callback: Dict[str, Callable] | None = None):
        self.token_callback: Dict[str, Callable] = {'INT': int, 'DECIMAL': float}
        if token_callback is not None and isinstance(token_callback, dict):
            self.token_callback.update(token_callback)

    def __call__(self, tree: Tree):
        if not isinstance(tree, Tree):
            raise ExceptionTreeType("argument type is not Tree. It's type is {}".format(type(tree)))
        c = self._transform_tree(tree)
        return c

    def _transform_tree(self, ast):
        children = list(self._transform_children(ast.get_children()))
        try:
            f = getattr(self, ast.get_name())
            return f(children)
        except AttributeError:
            return ast

    def _transform_children(self, children):
        for child in children:
            if isinstance(child, Tree):
                yield self._transform_tree(child)
            elif isinstance(child, Token):
                c = child.get_word()
                if self.token_callback and child.get_terminal() in self.token_callback:
                    c = self.token_callback[child.get_terminal()](c)
                yield c

class Visitor(ABC):
    def __call__(self, tree: Tree):
        if not isinstance(tree, Tree):
            raise ExceptionTreeType("argument type is not Tree. It's type is {}".format(type(tree)))

        self._visit_tree(tree)

    def _visit_tree(self, tree: Tree):
        if not isinstance(tree, Tree):
            return
        try:
            _user_func = tree.get_name()
            getattr(self, _user_func)(tree)
        except AttributeError as e:
            print(e)
            for child in tree.get_children():
                self._visit_tree(child)




__all__ = ["Swiftlet", "Tree", "Token", "pretty_print", "Transformer"]
