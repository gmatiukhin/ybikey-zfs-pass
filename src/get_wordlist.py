import requests
import re

url = "https://theworld.com/%7Ereinhold/diceware.wordlist.asc"
with open("diceware_wordlist.txt", "w") as o:
    print("Downloading wordlist...")
    text = requests.get(url).text
    for line in text.splitlines():
        match = re.search(r"\d{5}\s*(.+)", line.strip())
        if not match:
            continue
        print(match.groups()[0], file=o)
