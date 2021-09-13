#!/usr/bin/python3

import os
import shutil
import subprocess

NUM_FILES = 14
for i in range(NUM_FILES):
    filename = f"1-{i:05}-of-{NUM_FILES:05}.gz"
    url = f"http://storage.googleapis.com/books/ngrams/books/20200217/eng-us/{filename}"
    subprocess.run([f"wget {url}"], shell=True, check=True)
    shutil.move(filename, f"raw/{filename}")
    subprocess.run([f"gunzip raw/{filename}"], shell=True, check=True)
    # gunzip removes the .gz file, so no need to do this
    #os.remove(f"raw/{filename}")
