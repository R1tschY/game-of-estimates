<script lang="ts">
    import * as m from '$lib/paraglide/messages.js'
</script>

<main class="is-flex-grow-1 mt-3 mb-6">
    <section class="section">
        <div class="container is-max-tablet">
            <h1 class="title is-4">{m.summary()}</h1>
            <p>{@html m.description()}</p>
        </div>
    </section>

    <section class="section ml-3 mr-3">
        <div class="columns is-centered is-half">
            <form
                id="create-room-form"
                class="box"
                method="post"
                action="/room"
            ></form>
            <noscript class="box">{m.noscript()}</noscript>
        </div>
    </section>
</main>
