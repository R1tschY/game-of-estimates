export interface TranslationFile {
    plurals(n: number): number

    language: string
    strings: Strings
}

export interface Strings {
    [index: string]: string | string[]
}

export interface Translator {
    getText(textId: string): string
    getTextN(singularId: string, pluralId: string, n: number): string
}

export interface TranslationFiles {
    [index: string]: TranslationFile
}

export class DefaultTranslator implements Translator {
    private readonly translationFiles: TranslationFiles
    private readonly preferredLanguages: ReadonlyArray<string>
    private readonly fallback: string

    private currentLanguage: TranslationFile

    constructor(
        translationFiles: TranslationFiles,
        customPreferredLanguages?: ReadonlyArray<string>,
        fallback: string = 'en',
    ) {
        if (translationFiles[fallback] === undefined) {
            throw 'Fallback translation file is missing'
        }

        this.preferredLanguages =
            customPreferredLanguages || window.navigator.languages || []
        this.translationFiles = translationFiles
        this.fallback = fallback
        this.currentLanguage = this.getCurrentTranslation()

        console.debug(
            'Choose language',
            this.currentLanguage.language,
            'from',
            this.preferredLanguages,
        )
    }

    getText(textId: string) {
        let value = this.currentLanguage.strings[textId]
        if (value !== undefined) {
            if (Array.isArray(value)) {
                throw new Error('Expected non-plural text')
            }
            return value
        } else {
            return textId
        }
    }

    getTextN(singularId: string, pluralId: string, n: number) {
        const values = this.currentLanguage.strings[singularId]
        if (values !== undefined) {
            if (!Array.isArray(values)) {
                throw new Error('Expected plural text')
            }
            const plural = values[this.currentLanguage.plurals(n)]
            return plural !== undefined ? plural : pluralId
        } else {
            return n === 1 ? singularId : pluralId
        }
    }

    private getCurrentTranslation(): TranslationFile {
        for (const preferredLanguage of this.preferredLanguages) {
            let file = this.translationFiles[preferredLanguage]
            if (file !== undefined) {
                return file
            }
        }
        return this.translationFiles[this.fallback]
    }
}
