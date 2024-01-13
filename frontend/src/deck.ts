import { getText } from './i18n'

interface Deck {
    id: string
    name: string
    cards: string[]
}

export const decks: Deck[] = [
    {
        id: 'mod-fibonacci',
        name: 'Modified Fibonacci',
        cards: [
            '0',
            '½',
            '1',
            '2',
            '3',
            '5',
            '8',
            '13',
            '20',
            '40',
            '100',
            '?',
            '☕',
        ],
    },
    {
        id: 'fibonacci',
        name: 'Fibonacci',
        cards: [
            '0',
            '1',
            '2',
            '3',
            '5',
            '8',
            '13',
            '21',
            '34',
            '55',
            '89',
            '?',
            '☕',
        ],
    },
    {
        id: 't-shirt-sizes',
        name: 'T-shirt sizes',
        cards: ['XS', 'S', 'M', 'L', 'XL', 'XXL', '?', '☕'],
    },
    {
        id: 'power-of-2',
        name: 'Powers of 2',
        cards: ['0', '1', '2', '4', '8', '16', '32', '64', '?', '☕'],
    },
    {
        id: 'sequential',
        name: 'Sequential',
        cards: [
            '0',
            '1',
            '2',
            '3',
            '4',
            '5',
            '6',
            '7',
            '8',
            '9',
            '10',
            '?',
            '☕',
        ],
    },
]

export function getDeck(id: string): Deck | undefined {
    if (id.startsWith('custom:')) {
        const cards = id.substring('custom:'.length).split(/\s*,\s*/)
        return { id: 'custom', name: getText('customDeck'), cards }
    }

    for (const elem of decks) {
        if (elem.id == id) {
            return elem
        }
    }
}
