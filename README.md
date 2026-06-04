![Tuptacz](frontend/public/tuptacz-light.png)
# Tuptacz

These repository contains (still WIP) our two projects for the Computationl Problems in public transportation course - they both share the frontend and the backend.

- `TuptaCH` - a visualization of Contraction Hierarchies algorithm
- `JakDotuptam` - a connection finder for public transport in Kraków

## Run instructions

The app has two major components:
- the algorithmic backend
- the visualization frontend (browser-based)

The two components communicate over a REST API and over websockets.

The simplest way is to run the project with docker by running
```bash
./run-prod.sh
```

It exposes the whole app on `localhost:8080`.


Alternatively you can follow the below instructions to run the frontend and the backend manually without Docker.

### Backend

0. Install Rust: [Instructions](https://rust-lang.org/tools/install/).
1. Make sure that port `3000` on `localhost` is free.

    By default, the backend will be served on that port.

    Alternatively, you can edit the source to pick a different port.
2. Run backend:
    ```bash
    cd backend
    cargo run --release
    ```

### Frontend

0. Install npm and NodeJS: [Instructions](https://nodejs.org/en/download).(project was tested with Node 24)
1. Install dependencies:
    ```bash
    cd frontend
    npm i
    ```
2. Make sure that port `5173` on `localhost` is free.

    By default, the fronend will be served on that port.

    Alternatively, you can edit the source to pick a different port.with `npm install`
3. Build the project with `npm run build`. run preview
4. Run `npm run preview` to start the app


## Authors

- Jakub Oskwarek @honvex-cool

- Michał Horodecki @mhorod