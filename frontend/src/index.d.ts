declare namespace svelteHTML {
    interface HTMLAttributes<T> {
        [x: `g-${string}`]: any
    }
}
