export {}

const PLACEHOLDER_RE = /{([0-9]+)}/g

String.prototype.format = function () {
    const args = arguments
    return this.replace(PLACEHOLDER_RE, function (match, index) {
        let arg = args[index]
        return arg === undefined ? '' : arg
    })
}
