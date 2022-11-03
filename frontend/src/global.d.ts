export {}

declare global {
    interface String {
        format(...args: string[]): string
    }
}
