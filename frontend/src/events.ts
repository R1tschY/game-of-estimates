
export type EventHandler = (evt: any) => void

export class CustomEventTarget {
    listeners: Record<string, EventHandler[]>  = {};

    addEventListener(name: string, handler: EventHandler) {
        if (Object.prototype.hasOwnProperty.call(this.listeners, name))
            this.listeners[name].push(handler)
        else
            this.listeners[name] = [handler]
    }

    removeEventListener(name: string, handler: EventHandler) {
        if (!Object.prototype.hasOwnProperty.call(this.listeners, name))
            return

        const index = this.listeners[name].indexOf(handler)
        if (index != -1)
            this.listeners[name].splice(index, 1)
    }

    dispatchEvent(name: string, payload: any) {
        if (!Object.prototype.hasOwnProperty.call(this.listeners, name))
            return;

        const listeners = this.listeners[name]
        const l = listeners.length
        for (let i = 0; i < l; i++) {
            listeners[i](payload)
        }
    }
}

export type SignalHandler<T> = (evt: T) => void

export class Signal<T> {
    listeners: EventHandler[]  = [];

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