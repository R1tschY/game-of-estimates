<script lang="ts">
    import '../main.sass'

    import type { AvailableLanguageTag } from '$lib/paraglide/runtime'
    import { i18n } from '$lib/i18n'
    import { page } from '$app/stores'
    import { goto } from '$app/navigation'
    import * as m from '$lib/paraglide/messages.js'

    function switchToLanguage(newLanguage: AvailableLanguageTag) {
        const canonicalPath = i18n.route($page.url.pathname)
        const localisedPath = i18n.resolveRoute(canonicalPath, newLanguage)
        goto(localisedPath)
    }
</script>

<section class="hero head">
    <div class="hero-body">
        <div class="container">
            <h1 class="title"><a href="/">Game Of Estimates</a></h1>
            <h2 class="subtitle">Planning Poker</h2>
        </div>
    </div>
</section>

<main>
    <slot />
</main>

<footer class="footer">
    <div class="content has-text-centered">
        <p>{@html m.byAuthor()}</p>
        <p>{@html m.license()}</p>
        <div>
            <a href="/" class="footer-icon">
                <img src="github.svg" alt="GitHub" />
            </a>
        </div>
        <div>
            <button onclick={() => switchToLanguage('en')}>en</button>
            <button onclick={() => switchToLanguage('de')}>de</button>
        </div>
    </div>
</footer>
