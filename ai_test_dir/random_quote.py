import json
import random
import os

# Ruta al JSON de citas
HERE = os.path.dirname(__file__)
FILE = os.path.join(HERE, 'quotes.json')

def load_quotes(path):
    with open(path, 'r', encoding='utf-8') as f:
        return json.load(f)

def main():
    quotes = load_quotes(FILE)
    if not quotes:
        print("No hay citas disponibles.")
        return
    quote = random.choice(quotes)
    print(f"\n💬 {quote}\n")

if __name__ == '__main__':
    main()