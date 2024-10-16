export interface TranslationFile {
    language: string
    strings: Record<string, string | string[]>
}

export interface Translator {
    getText(textId: string): string
}

export type TranslationFiles = Record<string, TranslationFile>

export class DefaultTranslator implements Translator {
    private readonly translationFiles: TranslationFiles
    private readonly preferredLanguages: readonly string[]
    private readonly fallback: string

    private currentLanguage: TranslationFile

    constructor(
        translationFiles: TranslationFiles,
        customPreferredLanguages?: readonly string[],
        fallback = 'en',
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
        const value = this.currentLanguage.strings[textId]
        if (value !== undefined) {
            if (Array.isArray(value)) {
                throw new Error('Expected non-plural text')
            }
            return value
        } else {
            return textId
        }
    }

    private getCurrentTranslation(): TranslationFile {
        for (const preferredLanguage of this.preferredLanguages) {
            const file = this.translationFiles[preferredLanguage]
            if (file !== undefined) {
                return file
            }
        }
        return this.translationFiles[this.fallback]
    }
}
