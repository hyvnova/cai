# -*- coding: utf-8 -*-
import random
import datetime

QUOTES = [
    "The only limit to our realization of tomorrow is our doubts of today. - F. D. Roosevelt",
    "In the middle of difficulty lies opportunity. - Albert Einstein",
    "Life is 10% what happens to us and 90% how we react to it. - Charles R. Swindoll",
    "Do not watch the clock. Do what it does. Keep going. - Sam Levenson",
    "The future belongs to those who believe in the beauty of their dreams. - E. Roosevelt"
]

def main():
    now = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    quote = random.choice(QUOTES)
    print(f"[{now}] {quote}")

if __name__ == "__main__":
    main()