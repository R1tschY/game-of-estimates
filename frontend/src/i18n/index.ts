import type { Translator } from './lib'
import { DefaultTranslator } from './lib'

import de from './de.json'
import en from './en.json'

export const TRANSLATOR: Translator = new DefaultTranslator({ de, en })

export function getText(textId: string): string {
    return TRANSLATOR.getText(textId)
}

export function _(textId: string): string {
    return TRANSLATOR.getText(textId)
}
