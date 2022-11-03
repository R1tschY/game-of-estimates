import type { TranslationFile } from './lib'

const TRANSLATIONS: TranslationFile = {
    language: 'en',
    plurals(n: number): number {
        return n === 1 ? 0 : 1
    },
    strings: {
        createRoom: 'Create room',
        deck: 'Deck',
        joinRoom: 'Join existing room',
        or: 'or',
        voter: 'Voter',
        estimates: 'Estimates',
        restart: 'Restart',
        open: 'Open',
        copyRoomLink: 'Copy room link',
        changeName: 'Change name',
        chooseYourEstimate: 'Choose your estimate',
        license: 'The source code is licensed under {0}.',
        byAuthor: '{0} by {1}.',
        summary: 'Plan your sprint with a little game',
        description:
            'Game Of Estimates gives you the chance to do your <a href="https://en.wikipedia.org/wiki/Planning_poker">Planning Poker</a> ' +
            '(also known as Scrum Poker) online and for free. Feel free to contribute, because it is ' +
            '<a href="https://github.com/R1tschY/game-of-estimates">open source</a>!',
    },
}

export default TRANSLATIONS
