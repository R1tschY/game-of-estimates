export {}

const PLACEHOLDER_RE = /{([0-9]+)}/g

String.prototype.format = function (...args: string[]) {
    return this.replace(PLACEHOLDER_RE, (_: string, index: number) => {
        const arg = args[index]
        return arg === undefined ? '' : arg
    })
}
