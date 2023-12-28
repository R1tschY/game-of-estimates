export type SignalHandler<T> = (evt: T) => void

export class Signal<T> {
    listeners: SignalHandler<T>[] = []

    connect(handler: SignalHandler<T>) {
        this.listeners.push(handler)
    }

    disconnect(handler: SignalHandler<T>) {
        const index = this.listeners.indexOf(handler)
        if (index != -1) {
            this.listeners.splice(index, 1)
        }
    }

    emit(payload: T) {
        const l = this.listeners.length
        for (let i = 0; i < l; i++) {
            this.listeners[i](payload)
        }
    }
}
