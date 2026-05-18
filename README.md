# Tuptacz

## Run instructions

The app has two major components:
- the algorithmic backend
- the visualization frontend (browser-based)

The two components communicate over a REST API and over websockets.

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
2. Make sure that port `44173` on `localhost` is free.

    By default, the fronend will be served on that port.

    Alternatively, you can edit the source to pick a different port.with `npm install`
3. Build the project with `npm run build`. run preview
4. Run `npm run preview` to start the app