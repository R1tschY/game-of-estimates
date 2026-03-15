<script lang="ts">
    import '../main.sass'

    import { page } from '$app/state'
    import { locales, localizeHref } from '$lib/paraglide/runtime'
    import { m } from '$lib/paraglide/messages.js'
</script>

<section class="hero head">
    <div class="hero-body">
        <div class="container">
            <h1 class="title"><a href="/">{m.title()}</a></h1>
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            <h2 class="subtitle">{@html m.summary()}</h2>
        </div>
    </div>
</section>

<slot />

<footer class="footer">
    <div class="content has-text-centered">
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        <p>{@html m.byAuthor()}</p>
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        <p>{@html m.license()}</p>
        <div>
            <a href="/" class="footer-icon">
                <img src="/github.svg" alt="GitHub" />
            </a>
        </div>
        <nav aria-label="Languages">
            {#each locales as locale}
                <a
                    href={localizeHref(
                        page.url.pathname + '?' + page.url.searchParams,
                        {
                            locale,
                        },
                    )}
                    data-sveltekit-reload
                >
                    {locale}
                </a>
            {/each}
        </nav>
    </div>
</footer>
