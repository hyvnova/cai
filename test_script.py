#!/usr/bin/env python3
import random

def main():
    print("¡Bienvenido al juego de adivina el número!")
    numero = random.randint(1, 20)
    intentos = 3

    for i in range(intentos):
        try:
            guess = int(input(f"Tienes {intentos - i} intento(s). Adivina un número entre 1 y 20: "))
        except ValueError:
            print("Eso no es un número válido. Pierdes un intento.")
            continue

        if guess == numero:
            print("Exacto. ¡Lo lograste!")
            break
        elif guess < numero:
            print("Muy bajo...")
        else:
            print("Muy alto...")

    else:
        print(f"Se te acabaron los intentos. El número era {numero}. Mejor suerte la próxima :)")

if __name__ == '__main__':
    main()
