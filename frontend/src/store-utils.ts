import { derived, get, writable } from "svelte/store";
import type { Writable } from "svelte/store";

// Local Storage

export function writableLocalStorage<T>(key: string, value: T): Writable<T> {
    const jsonValue = localStorage.getItem(key)
    if (jsonValue) {
        value = JSON.parse(jsonValue) as T
    }

    const store = writable(value)
    const { subscribe, set } =  store

    return {
        subscribe,
        
        set(newValue) {
            localStorage.setItem(key, JSON.stringify(newValue))
            set(value)
        },
        
        update(fn) {
            const newValue = fn(get(store))
            localStorage.setItem(key, JSON.stringify(newValue))
            set(newValue)
        },
    }
}

// Writable derived

export function derivedWritable<T, U>(store: Writable<U>, read: (value: U) => T, update: (old: U, value: T) => U): Writable<T> {
    const newStore = derived(store, read)
    return {
        subscribe: newStore.subscribe,
        set(value) {
            store.update((old) => update(old, value))
        },
        update(fn) {
            store.update((old) => update(old, fn(read(old))))
        }
    }
}

export function derivedWritableProperty<T, U>(store: Writable<U>, read: (this: U) => T, update: (this: U, value: T) => void): Writable<T> {
    const newStore = derived(store, (value) => read.apply(value))
    return {
        subscribe: newStore.subscribe,
        set(value) {
            store.update((old) => update.apply(old, [value]))
        },
        update(fn) {
            store.update((old) => update.apply(old, [fn(read.apply(old))]))
        }
    }
}
