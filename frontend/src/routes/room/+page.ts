import type { PageLoad } from './$types'
import { browser } from '$app/environment'

export const prerender = true
export const ssr = true

export const load: PageLoad = ({ url }) => {
    return {
        id: browser ? url.searchParams.get('id') : undefined,
    }
}
