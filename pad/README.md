# Galaxy Pad

## Screenshot

![](https://user-images.githubusercontent.com/133952/88479925-a8598980-cf8d-11ea-997d-d791137e12f1.png)

## Features

This project compiles the [interpreter](https://github.com/seikichi/icfpc2020/tree/master/interpreter) to WebAssembly and runs the galaxy protocol in Web browsers!

## Requirements

- [wasm-pack](https://github.com/rustwasm/wasm-pack)
- [node](https://nodejs.org/)

## How to Run

```sh
> cd pad
> wasm-pack build
> cd www
> npm i
> npm start
```

Then, please open `http://localhost:8080/?apiKey={YOUR_API_KEY}`.
