<script>
    import { connected, player_id, game } from '../stores.js'
    import Banner from '../components/Banner.svelte'
</script>

<div>
    <Banner />
    <section class="section">
        <div class="container">
            <div class="columns">
                <div class="column is-one-quarter">
                    <input class="input" type="text" placeholder="Game no." />
                </div>
                <div class="column">
                    <button
                        type="submit"
                        class="button is-fullwidth is-primary"
                        on:click={game.create}>Join existing room</button>
                </div>
                <div class="is-divider-vertical" data-content="OR" />
                <div class="column">
                    <button
                        type="button"
                        class="button is-fullwidth is-warning"
                        on:click={game.create}>Create room</button>
                </div>
            </div>
        </div>
    </section>

    <section class="section">
        <div class="container">
            <div>Connected: {$connected}</div>
            <div>Player ID: {$player_id}</div>
            <div>game ID: {$game.id}</div>
            <div>game State: {$game.status}</div>
            <div>game Error: {$game.last_error}</div>
        </div>
    </section>
</div>
