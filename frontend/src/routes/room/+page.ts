import type { PageLoad } from './$types'

export const prerender = true
export const ssr = true

export const load: PageLoad = ({ url }) => {
    return {
        id: url.searchParams.get('id'),
    }
}
