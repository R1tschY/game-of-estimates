import type { Translator } from './lib'
import { DefaultTranslator } from './lib'

import de from './de'
import en from './en'

export const TRANSLATOR: Translator = new DefaultTranslator({ de, en })

export function getText(textId: string): string {
    return TRANSLATOR.getText(textId)
}

export function _(textId: string): string {
    return TRANSLATOR.getText(textId)
}

export function getTextN(
    singularId: string,
    pluralId: string,
    n: number,
): string {
    return TRANSLATOR.getTextN(singularId, pluralId, n)
}
