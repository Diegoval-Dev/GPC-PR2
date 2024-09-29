# Minecraft Diorama - Ray Tracing Simulation in Rust

Este proyecto es una simulación de un diorama estilo Minecraft utilizando ray tracing en Rust. El proyecto renderiza una escena 3D en tiempo real con bloques de agua, pasto, piedra, madera y glowstone, simula ciclos de día y noche, e incluye movimientos básicos de cámara para explorar la escena.

## Tabla de Contenidos

- [Instalación](#instalación)
- [Configuración del Proyecto](#configuración-del-proyecto)
- [Ejecución](#ejecución)
- [Controles](#controles)
- [Estructura del Código](#estructura-del-código)
- [Explicación de los Materiales](#explicación-de-los-materiales)
- [Ciclo de Día y Noche](#ciclo-de-día-y-noche)
- [Mejoras Futuras](#mejoras-futuras)

## Instalación

Antes de comenzar, asegúrate de tener instalado Rust. Si no lo tienes, puedes instalarlo desde [rust-lang.org](https://www.rust-lang.org/).

```bash
# Instalar Rust mediante Rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Configuración del Proyecto

Para clonar el proyecto y compilarlo, sigue los siguientes pasos:

```bash
# Clonar el repositorio
git clone
```

## Ejecución

Para ejecutar el proyecto, usa el siguiente comando:

```bash
# Compilar y ejecutar el proyecto
cargo run
```

## Controles

-  / S: Rotar la cámara hacia arriba y abajo.
- A / D: Rotar la cámara hacia la izquierda y derecha.
- Scroll del Mouse: Acercar y alejar la cámara del centro de la escena.

## Estructura del Código

El proyecto está dividido en diferentes módulos:

- `camera`: Contiene la estructura de la cámara y las funciones para moverla.
- `chunk`: Contiene la estructura de un chunk que representa una sección de la escena.
- `material`: Contiene las estructuras de los materiales y los shaders.
- `ray`: Contiene la estructura de un rayo y las funciones para lanzarlo.
- `scene`: Contiene la estructura de la escena y las funciones para renderizarla.
- `vector`: Contiene la estructura de un vector y las funciones para operar con ellos.

## Explicación de los Materiales

El proyecto incluye los siguientes materiales:

- Agua: Refleja la luz y distorsiona la imagen.
- Pasto: Textura con colores aleatorios.
- Piedra: Textura con colores grises.
- Madera: Textura con colores marrones.
- Glowstone: Textura con colores amarillos y brilla en la oscuridad.

## Ciclo de Día y Noche

El proyecto simula un ciclo de día y noche en la escena. Durante el día, la luz es más brillante y azulada. Durante la noche, la luz es más tenue y anaranjada.

## Video

https://github.com/user-attachments/assets/a3427317-b62d-490b-9954-0b89c2ea34bb


