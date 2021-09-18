from io import FileIO, TextIOWrapper
from typing import DefaultDict, Dict


frequencies: Dict[str, int] = {}

def is_allowed_word(s: str) -> bool:
    if not s.isascii():
        return False
    return all([c.isalpha() or c == '-' or c == '\'' or c == '_' for c in s])

# skipping _NUM since we don't want numbers anyway
SUFFIXES = ["_NOUN", "_VERB", "_ADJ", "_ADV", "_ADP", "_PRON", "_DET", "_CONJ", "_PRT"]
def trim_part_of_speech(word: str) -> str:
    for suffix in SUFFIXES:
        if word.endswith(suffix):
            return word[:-len(suffix)]
    return word

def parse_line(line: str, frequencies: Dict[str, int]) -> None:
    parts = line.split()
    word = parts[0]
    if not is_allowed_word(word):
        return
    word = trim_part_of_speech(word)
    if "_" in word:
        # ("skipping {} because underscores in weird places", word);
        return
    word = word.lower()
    count = 0
    for entry in parts[1:]:
        # each entry is "<year>,<match count>,<volume count>"
        entry_parts = entry.split(',')
        count += int(entry_parts[1])
    if word not in frequencies:
        frequencies[word] = count
    else:
        frequencies[word] += count


def parse_file(f: TextIOWrapper, frequencies: Dict[str, int]) -> None:
    for line in f:
        parse_line(line, frequencies)

for i in range(14):
    with open(f"../data/raw/1-{i:05}-of-00014", 'r', encoding="utf-8") as f:
        parse_file(f, frequencies)
print (f"got {len(frequencies)} words")
frequencies_sorted = sorted(frequencies.items(), key=lambda x: x[1], reverse=True)
for word, count in frequencies_sorted[:25]:
    print(f"{word}: {count}")
with open("../data/processed/word_frequency.txt", 'w', encoding="utf-8") as f:
    # There are lots of incredibly rare words, as well as a ton of typos (where
    # presumably the OCR was wrong) at low frequencies. On inspection, we're not
    # losing anything valuable if we cut off at 10000, and it reduces false positives
    # and file size.
    for entry in frequencies_sorted:
        if entry[1] >= 10000:
            f.write(f"{entry[0]} {entry[1]}\n")
