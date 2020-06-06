import math
import re
from collections import Counter

WORD = re.compile(r"\w+")


def get_cosine(vec1, vec2):
    intersection = set(vec1.keys()) & set(vec2.keys())
    numerator = sum([vec1[x] * vec2[x] for x in intersection])

    sum1 = sum([vec1[x] ** 2 for x in list(vec1.keys())])
    sum2 = sum([vec2[x] ** 2 for x in list(vec2.keys())])
    denominator = math.sqrt(sum1) * math.sqrt(sum2)

    if not denominator:
        return 0.0
    else:
        return float(numerator) / denominator


def text_to_vector(text):
    # words = WORD.findall(text)
    words = text.split(' ')
    # words = text.split(' ')
    return Counter(words)


text1 = "This is a foo bar sentence ."
text2 = "This sentence is similar to a foo bar sentence ."

vector1 = text_to_vector(text1)
vector2 = text_to_vector(text2)
v1 = vector1.keys()
v2 = vector2.keys()

print(f'text1: {vector1}')
print(f'text2: {vector2}')

print(f'\nv1: {v1}')
print(f'\nv2: {v2}')
print(f'\nset: {set(v1)& set(v2)}')
cosine = get_cosine(vector1, vector2)

print("Cosine:", cosine)
