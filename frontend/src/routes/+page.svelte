<script lang="ts">
    import { m } from '$lib/paraglide/messages.js'
    import Glasses from '@lucide/svelte/icons/glasses'
    import Bird from '@lucide/svelte/icons/bird'
    import CreateRoomForm from '$/components/organisms/CreateRoomForm.svelte'
</script>

<main class="is-flex-grow-1 pt-3">
    <section class="section pb-6">
        <div class="columns is-centered">
            <div class="column is-half">
                <p class="pb-3">{m.lobbyActionText()}</p>
                <CreateRoomForm></CreateRoomForm>
                <noscript class="box">{m.noscript()}</noscript>
            </div>
        </div>
    </section>

    <section class="hero is-primary is-medium">
        <div class="hero-body">
            <div class="columns is-centered is-half">
                <div class="container is-max-tablet is-size-5">
                    <Glasses class="is-large"></Glasses>
                    <ul>
                        <li>{m.lobbyClaimPrivacy1()}</li>
                        <li>{m.lobbyClaimPrivacy2()}</li>
                    </ul>
                </div>
            </div>
        </div>
    </section>

    <section class="hero is-success is-medium">
        <div class="hero-body">
            <div class="columns is-centered is-half">
                <div class="container is-max-tablet is-size-5">
                    <Bird class="is-large"></Bird>
                    <ul>
                        <li>{m.lobbyClaimFree1()}</li>
                        <li>{m.lobbyClaimFree2()}</li>
                    </ul>
                </div>
            </div>
        </div>
    </section>
</main>
