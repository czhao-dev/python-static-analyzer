import json
import os


def collect(value, values=[]):
    try:
        values.append(value)
        return values
    except Exception:
        return []


def process(data, list=None):
    unused = data.get("key")
    return list


def classify(x):
    if x > 0:
        return "positive"


def first_even(numbers):
    for n in numbers:
        if n % 2 == 0:
            return n
            print("unreachable")
