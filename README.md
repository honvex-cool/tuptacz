![Tuptacz](frontend/public/tuptacz-light.png)
# Tuptacz

These repository contains (still WIP) our two projects for the Computationl Problems in public transportation course - they both share the frontend and the backend.

- `TuptaCH` - a route finder for Lesser Poland
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


## AI Usage

AI was mostly used for discussion, learning, prototyping, debugging, and reviewing ideas, rather than vibe-coding the app.

The majority of the code was written directly by hand, with AI helping with minor snippets.

Parts of the frontend for TuptaCH were vibecoded.

The hedgehog logo was drawn by hand (with some inspirations from the internet) :)

## Ideas, experiments, and failures

### TuptaCH

#### Framework

Although the visualization is rather slow (due to the event generation, communication, and rendering overheads), the underlying framework has potential for good performance. Some flexible zero-cost or nearly-zero-cost Rust abstractions were created:
- Immutable vertex and edge views that can be implemented for any underlying graph representation
- Reusable memory pools using the epoch technique - implemented using Rust's `Cell<T>` type which usually has the same hardware representation as the wrapped type but enables interior mutability
- Generic Dijkstra's algorithm, controllable with drivers and heuristics (which are monomorphized and aggressively inlined in most cases)
- "Preproceessing - Pathfinder - Query" pattern combining the flexibility of dynamic dispatch *between* phases with the performance of static dispatch *within* phase

#### Resolved bugs

In implementation of CH, two subtle bugs appeared:
- The priority queue was implemented in a way that enabled it to be generic over whether it supports key updates or not. This is realized via an index tracker, provided by the user. However, during insertion, the index was not correctly updated. It caused some existing values to be reinserted with a different key, even in the version supporting key updates. This caused some vertices to be contracted multiple times, messing up the ranks.
- Some drivers for the bounded Dijkstra's algorithm were supposed to decide whether an edge is available in a given search or not. However, they didn't consult that decision with the inner driver. This caused the bounded search to find a path through the vertex being contracted and no shortcuts were ever added.

Once these two bugs were fixed, the otherwise correct implementation started working. This was truly enlightening (but also frustrating because the bugs were not in the main logic).

#### TODO
- Add more CH heuristics
- Add more graph representations to choose from
- Disable interactive preprocessing for CH on larger graphs - even in the "Run to completion" mode it takes too much time in the browser
- Improve frontend responsiveness
- Troubleshoot frontend race conditions that sometimes appear during queries

### RAPTOR

There were few problems with our implementation along the way:
- First bug: The algorithm allowed "going back in time" which we figured was due to saving wrong info in the wrong place - easy fix was to verify the implementation.
- Redundant journeys:
    - The current implementation does a search from a set of start stops to a set of end stops (because often there are at least two stops with the same name). This causes a quadratic number of "raw" journeys returned - from each start to each end. Of course many of them are redundant and differ only in the walking to first/last stop.
    - What's even funnier is that for bigger hubs the algorithm would return *a longer journey* with *more changes* - with the only difference being last *physical* stop. An example of this would be going one stop further, then immediately going back - however the backwards route would stop in a physically different place and this maneouver would apparently be faster than walking through the hub with the speed we assumed. (The time to the hub is longer, but the time to this particular physical stop would be shorter).
    - The above problems were fixed by implementing deduplication that filters journeys with the same execution, and those that are Pareto-dominated.
- (Unsolved) Change-equivalent stops:
    - Public transit often has a lot of overlapping lines that join, travel together and then split.
    Therefore if we want to change between two such lines there are many (equivalent) options - which one should be chosen? RAPTOR chooses first stop on the second route.
    - Better yet - imagine the network (or its part) has `Y` shape and we want to get from the top left corner to the top right corner - the shortest path is just a `v` going just through the middle.

    But, again, we can go down the "common tail" and change at any other point equivalently (probably up to some point), leading up to longer journey in terms of distance, but taking the same time due to trip frequencies.

    - In both cases we have many options to choose where to change, and it might be non-trivial - e.g. changing at the first possible stop might be inconvenient due to longer distance to walk or due to large amount of people, or unsafe location.

    RAPTOR chooses the first stop on the second route, but it might not be the stop that people would choose.

    Choosing the first stop also means that if we catch the very first bus/tram from the second route we will change as late as possible (on the last common stop) -- you can see that by searching some "Y" shaped path at 00:00 (e.g. Rostworowskiego -> Norymberska - you can walk this in 5 minutes, but we are sent for a 40 minutes travel through Rondo Grunwaldzkie.)

## Authors

- Jakub Oskwarek @honvex-cool

- Michał Horodecki @mhorod
