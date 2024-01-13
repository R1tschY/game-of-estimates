import type { TranslationFile } from './lib'

const TRANSLATIONS: TranslationFile = {
    language: 'de',
    plurals(n: number): number {
        return n === 1 ? 0 : 1
    },
    strings: {
        createRoom: 'Erstelle Raum',
        deck: 'Deck',
        customDeck: 'Benutzerdefiniertes Deck',
        customDeckField: 'Kartenwerte',
        customDeckPlaceholder: 'Kommaseparierte Liste: 1,2,3,...',
        customDeckHelp: 'Kommaseparierte Liste der Kartenwerte',
        observer: 'Beobachter',
        playerNamePlaceholder: 'Ohne Name',
        estimates: 'Schätzungen',
        restart: 'Neu starten',
        open: 'Öffnen',
        copyRoomLink: 'Kopiere Raum Link',
        changeName: 'Ändere Name',
        chooseYourEstimate: 'Wähle deine Schätzung',
        license: 'Der Quellcode steht unter der {0} Lizenz.',
        byAuthor: '{0} von {1}.',
        summary: 'Plane deinen Sprint mit einem kleinen Spiel',
        description:
            'Game Of Estimates gibt dir die Möglichkeit dein <a href="https://de.wikipedia.org/wiki/Scrum#Planungspoker">Planning Poker</a> ' +
            '(auch bekannt als Scrum Poker) online und kostenlos durchzuführen. Helfe mit diese Software besser zu machen. Sie ist ' +
            '<a href="https://github.com/R1tschY/game-of-estimates">Open Source</a>!',
    },
}

export default TRANSLATIONS
