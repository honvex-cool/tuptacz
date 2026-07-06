export function apiGet(path: string) {
    return fetch(`${window.location.protocol}//${window.location.host}/api${path}`)
}

export function apiPost(path: string, payload: any) {
    return fetch(
        `${window.location.protocol}//${window.location.host}/api${path}`, {
        method: "POST",
        body: JSON.stringify(payload),
        headers: {
            "Content-type": "application/json; charset=UTF-8"
        }
    }
    )
}
