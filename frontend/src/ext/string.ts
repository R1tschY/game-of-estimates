export {}

const PLACEHOLDER_RE = /{([0-9]+)}/g

String.prototype.format = function (...args) {
    return this.replace(PLACEHOLDER_RE, function (match, index) {
        const arg = args[index]
        return arg === undefined ? '' : arg
    })
}
